use crate::data::{self, File, Map, Value};
use crate::ast::{
    Expression, Spanned, BinaryOperator, UnaryOperator,
    Pattern, PatternField, StructField
};
use crate::runtime::Runtime;

/// The environment manages the stack of variable bindings.
pub struct Environment<'a> {
    builtins: &'a Map,
    custom: &'a Map,
    frames: Vec<Map>,
}

impl<'a> Environment<'a> {
    pub fn new(builtins: &'a Map, custom: &'a Map) -> Self {
        Self {
            builtins,
            custom,
            frames: vec![Map::new()], // Start with an empty local frame
        }
    }

    pub fn push_frame(&mut self) {
        self.frames.push(Map::new());
    }

    pub fn pop_frame(&mut self) {
        self.frames.pop();
    }

    pub fn bind(&mut self, name: String, value: Value) {
        if let Some(frame) = self.frames.last_mut() {
            *frame = frame.clone().put(Value::String(name), value);
        }
    }

    pub fn resolve(&self, name: &str) -> Option<Value> {
        let key = Value::String(name.to_string());
        for frame in self.frames.iter().rev() {
            if let Some(val) = frame.get(&key) {
                return Some(val.clone());
            }
        }
        if let Some(val) = self.custom.get(&key) {
            return Some(val.clone());
        }
        if let Some(val) = self.builtins.get(&key) {
            return Some(val.clone());
        }
        None
    }
}

/// Extracts the implicit key string from an expression (e.g., last identifier in a get chain).
fn extract_inferred_key(expr: &Spanned<Expression>) -> Result<String, String> {
    match &expr.node {
        Expression::Identifier(name) => Ok(name.clone()),
        Expression::FieldGet { field, .. } => Ok(field.clone()),
        // If there are other access chain constructs, add them here
        _ => Err("Cannot infer key from expression in value-only structure field".to_string()),
    }
}

/// Recursively binds a Map value against structural pattern fields.
fn bind_pattern_field(env: &mut Environment, field: &PatternField, map: &Map) -> Result<(), String> {
    match field {
        PatternField::Identifier(name) => {
            let val = map.get(&Value::String(name.clone()))
                .ok_or_else(|| format!("Structural match failed: key '{}' not found", name))?;
            env.bind(name.clone(), val.clone());
            Ok(())
        }
        PatternField::Structural(_) => {
            Err("Anonymous nested structural patterns without a key are not supported".to_string())
        }
        PatternField::BoundStructural { identifier, structure } => {
            let val = map.get(&Value::String(identifier.clone()))
                .ok_or_else(|| format!("Structural match failed: key '{}' not found", identifier))?;
            env.bind(identifier.clone(), val.clone());

            if let Value::Map(inner_map) = val {
                for inner_field in structure {
                    bind_pattern_field(env, &inner_field.node, inner_map)?;
                }
                Ok(())
            } else {
                Err(format!("Expected a map for nested structural binding on '{}'", identifier))
            }
        }
    }
}

/// Binds an argument to a pattern within the environment.
fn bind_pattern(env: &mut Environment, pattern: &Pattern, arg: &Value) -> Result<(), String> {
    match pattern {
        Pattern::Identifier(name) => {
            // Sole identifier: bind the value as it is under the identifier
            env.bind(name.clone(), arg.clone());
            Ok(())
        }
        Pattern::Structural(fields) => {
            // Do structural binding
            let Value::Map(map) = arg else {
                return Err("Argument must be a map for structural binding".to_string());
            };
            for field in fields {
                bind_pattern_field(env, &field.node, map)?;
            }
            Ok(())
        }
        Pattern::BoundStructural { identifier, structure } => {
            // Do both: bind the entire value and recursively structural match
            env.bind(identifier.clone(), arg.clone());

            let Value::Map(map) = arg else {
                return Err("Argument must be a map for structural binding".to_string());
            };
            for field in structure {
                bind_pattern_field(env, &field.node, map)?;
            }
            Ok(())
        }
    }
}

fn builtins() -> Map {
    Map::new()
        .put(Value::String("load_file".to_owned()), Value::Function(data::Function::LoadFile))
        .put(Value::String("write_file".to_owned()), Value::Function(data::Function::WriteFile))
}

/// The main evaluation function.
pub fn eval(
    expr: &Spanned<Expression>,
    custom: &Map,
    arg: &Value,
    runtime: &Runtime,
) -> Result<Value, String> {
    let builtins = builtins();
    let mut env = Environment::new(&builtins, custom);
    eval_inner(expr, &mut env, arg, runtime)
}

fn eval_inner(
    expr: &Spanned<Expression>,
    env: &mut Environment,
    current_arg: &Value,
    runtime: &Runtime,
) -> Result<Value, String> {
    match &expr.node {
        Expression::Boolean(b) => Ok(Value::Boolean(*b)),
        Expression::Integer(i) => Ok(Value::Integer(*i)),
        Expression::Float(f) => Ok(Value::Float(f.to_bits())),
        Expression::String(s) => Ok(Value::String(s.clone())),
        Expression::Path(p) => Ok(Value::File(File::new(p))),

        Expression::Identifier(name) => {
            env.resolve(name).ok_or_else(|| format!("Unresolved identifier: {}", name))
        }

        Expression::Structure(fields) => {
            let mut map = Map::new();
            for (index, field) in fields.iter().enumerate() {
                match &field.node {
                    // Tuple semantics: Ordinal integer key
                    StructField::Identifier(inner_expr) => {
                        let val = eval_inner(inner_expr, env, current_arg, runtime)?;
                        map = map.put(Value::Integer(index as i64), val);
                    }
                    // Map semantics: Explicit K = V
                    StructField::KeyValue(k_expr, v_expr) => {
                        let k = eval_inner(k_expr, env, current_arg, runtime)?;
                        let v = eval_inner(v_expr, env, current_arg, runtime)?;
                        map = map.put(k, v);
                    }
                    // Assignment without names: Extracted key from expression
                    StructField::ValueOnly(inner_expr) => {
                        let k_str = extract_inferred_key(inner_expr)?;
                        let val = eval_inner(inner_expr, env, current_arg, runtime)?;
                        map = map.put(Value::String(k_str), val);
                    }
                }
            }
            Ok(Value::Map(map))
        }

        Expression::Function { pattern, body } => {
            env.push_frame();
            bind_pattern(env, &pattern.node, current_arg)?;
            let result = eval_inner(body, env, current_arg, runtime);
            env.pop_frame();
            result
        }

        Expression::If { condition, then_branch, else_branch } => {
            let cond_val = eval_inner(condition, env, current_arg, runtime)?;
            if let Value::Boolean(true) = cond_val {
                eval_inner(then_branch, env, current_arg, runtime)
            } else if let Value::Boolean(false) = cond_val {
                eval_inner(else_branch, env, current_arg, runtime)
            } else {
                Err("If condition must evaluate to a boolean".to_string())
            }
        }

        Expression::Cases { target, branches, default } => {
            let target_val = eval_inner(target, env, current_arg, runtime)?;

            for branch in branches {
                env.push_frame();
                if bind_pattern(env, &branch.node.pattern.node, &target_val).is_ok() {
                    let mut guard_passed = true;
                    if let Some(guard) = &branch.node.guard {
                        let guard_val = eval_inner(guard, env, current_arg, runtime)?;
                        if guard_val != Value::Boolean(true) {
                            guard_passed = false;
                        }
                    }
                    if guard_passed {
                        let result = eval_inner(&branch.node.body, env, current_arg, runtime);
                        env.pop_frame();
                        return result;
                    }
                }
                env.pop_frame();
            }

            if let Some(def) = default {
                eval_inner(def, env, current_arg, runtime)
            } else {
                Err("No case branch matched and no default provided".to_string())
            }
        }

        Expression::BinaryOp { op, lhs, rhs } => {
            let left = eval_inner(lhs, env, current_arg, runtime)?;
            let right = eval_inner(rhs, env, current_arg, runtime)?;

            match (op, left, right) {
                (BinaryOperator::PutAll, Value::Map(m1), Value::Map(m2)) => {
                    Ok(Value::Map(m1.put_all(&m2)))
                }
                (BinaryOperator::Add, Value::Integer(a), Value::Integer(b)) => {
                    Ok(Value::Integer(a + b))
                }
                (BinaryOperator::Equal, a, b) => Ok(Value::Boolean(a == b)),
                _ => Err(format!("Unsupported binary operation {:?} for the given types", op)),
            }
        }

        Expression::UnaryOp { op, expr } => {
            let val = eval_inner(expr, env, current_arg, runtime)?;
            match (op, val) {
                (UnaryOperator::Not, Value::Boolean(b)) => Ok(Value::Boolean(!b)),
                (UnaryOperator::Negate, Value::Integer(i)) => Ok(Value::Integer(-i)),
                _ => Err(format!("Unsupported unary operation {:?} for the given type", op)),
            }
        }

        Expression::FieldGet { target, field } => {
            let target_val = eval_inner(target, env, current_arg, runtime)?;
            if let Value::Map(map) = target_val {
                let key = Value::String(field.clone());
                map.get(&key)
                    .cloned()
                    .ok_or_else(|| format!("Field '{}' not found", field))
            } else {
                Err("FieldGet is only supported on Maps".to_string())
            }
        }

        Expression::IndexGet { target, index } => {
            let target_val = eval_inner(target, env, current_arg, runtime)?;
            let index_val = eval_inner(index, env, current_arg, runtime)?;
            if let Value::Map(map) = target_val {
                map.get(&index_val)
                    .cloned()
                    .ok_or_else(|| "Index not found".to_string())
            } else {
                Err("IndexGet is only supported on Maps".to_string())
            }
        }

        Expression::Call { func, arg } => {
            let called_val = eval_inner(func, env, current_arg, runtime)?;
            let call_arg = eval_inner(arg, env, current_arg, runtime)?;

            if let Value::String(lhs) = called_val {
                if let Value::String(rhs) = call_arg {
                    return Ok(Value::String(format!("{lhs}{rhs}")));
                }
                return Err("Calling a string mean concatinating it, expected the argument to be string".to_owned());
            }

            if let Value::Function(f) = called_val {
                f.call(&call_arg, runtime).map_err(|_| "Function execution error".to_string())
            } else {
                Err("Attempted to call a non-function".to_string())
            }
        }

        Expression::Include(_) => {
            Err("Include is not directly evaluable in this context".to_string())
        }
    }
}
