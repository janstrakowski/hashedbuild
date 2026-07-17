use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::fs;
use crate::ast::*;

pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Spanned<Expression>, String> {
    let src_buf: Vec<u8> = fs::read(path.as_ref())
        .map_err(|io_err| format!("Could not read the source: {io_err}."))?;
    parse_raw(&src_buf, path.as_ref().to_string_lossy())
}

pub fn parse_raw(source: &[u8], src_name: impl Into<String>) -> Result<Spanned<Expression>, String> {
    let arc_path = Arc::new(PathBuf::from(src_name.into()));
    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_hashbuild::language();
    parser.set_language(&language).expect("Loading hashedbuild tree-sitter language failed.");
    let tree = parser.parse(source, None).unwrap();
    Translator::new(source, arc_path).translate(&tree)
}

pub struct Translator<'a> {
    source: &'a [u8],
    file: Arc<PathBuf>,
}

impl<'a> Translator<'a> {
    pub fn new(source: &'a [u8], file: Arc<PathBuf>) -> Self {
        Self { source, file }
    }

    /// Entry point for translation.
    pub fn translate(&self, tree: &tree_sitter::Tree) -> Result<Spanned<Expression>, String> {
        let root = tree.root_node();
        // The root node of a file will wrap the source_file rule.
        // If it's a direct expression, parse it.
        self.parse_expr(root)
    }

    /// Extracts the precise span from a tree-sitter node.
    fn span(&self, node: tree_sitter::Node) -> Span {
        let start = node.start_position();
        let end = node.end_position();
        Span {
            file: Arc::clone(&self.file),
            start_line: start.row,
            start_col: start.column,
            end_line: end.row,
            end_col: end.column,
        }
    }

    /// Helper to grab the raw text of a node.
    fn text(&self, node: tree_sitter::Node) -> String {
        node.utf8_text(self.source).unwrap_or("").to_owned()
    }

    /// Core expression translation.
    pub fn parse_expr(&self, node: tree_sitter::Node) -> Result<Spanned<Expression>, String> {
        let span = self.span(node);

        let expr = match node.kind() {
            // Literals
            "identifier" => Expression::Identifier(self.text(node)),
            "bool" => Expression::Boolean(self.text(node) == "true"),
            "integer" => Expression::Integer(
                self.text(node).parse().map_err(|_| format!("Invalid integer at {:?}", span))?
            ),
            "float" => {
                let text = self.text(node);
                // Handle tree-sitter's optional infinity/NaN matching if necessary
                let val = if text == "inf" || text == "infinity" {
                    f64::INFINITY
                } else if text == "NaN" {
                    f64::NAN
                } else {
                    text.parse().map_err(|_| format!("Invalid float at {:?}", span))?
                };
                Expression::Float(val)
            },
            "string" => {
                // Remove the outer quotes from the string literal[cite: 1]
                let raw = self.text(node);
                let content = if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
                    raw[1..raw.len() - 1].to_string()
                } else {
                    raw
                };
                let unescaped = unescaper::unescape(&content)
                    .map_err(|unesc_err| format!("Malformed string: {unesc_err}."))?;
                Expression::String(unescaped)
            },
            "path" => Expression::Path(self.text(node)),

            // Structures
            "structure" => {
                let mut fields = Vec::new();
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    if child.kind() == "structure_field" {
                        fields.push(self.parse_structure_field(child)?);
                    }
                }
                Expression::Structure(fields)
            },

            // Functions
            "function" => {
                let children: Vec<_> = node.named_children(&mut node.walk()).collect();
                let (pattern, consumed) = self.parse_pattern(&children)?;
                let body = Box::new(self.parse_expr(children[consumed])?);

                Expression::Function { pattern, body }
            },

            // Control Flow
            "if" => {
                let condition = Box::new(self.parse_expr(node.named_child(0).unwrap())?);
                let then_branch = Box::new(self.parse_expr(node.named_child(1).unwrap())?);
                let else_branch = Box::new(self.parse_expr(node.named_child(2).unwrap())?);
                Expression::If { condition, then_branch, else_branch }
            },
            "cases" => {
                let target = Box::new(self.parse_expr(node.named_child(0).unwrap())?);
                let mut branches = Vec::new();
                let mut default = None;

                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    if child.kind() == "case" {
                        branches.push(self.parse_case(child)?);
                    } else if child.kind() == "default" {
                        default = Some(Box::new(self.parse_expr(child.named_child(0).unwrap())?));
                    }
                }

                Expression::Cases { target, branches, default }
            },

            // Operations & Access
            "field_get" => {
                let target = Box::new(self.parse_expr(node.named_child(0).unwrap())?);
                let field = self.text(node.named_child(1).unwrap());
                Expression::FieldGet { target, field }
            },
            "get" => {
                let target = Box::new(self.parse_expr(node.named_child(0).unwrap())?);
                let index = Box::new(self.parse_expr(node.named_child(1).unwrap())?);
                Expression::IndexGet { target, index }
            },
            "call" => {
                let func = Box::new(self.parse_expr(node.named_child(0).unwrap())?);
                let arg = Box::new(self.parse_expr(node.named_child(1).unwrap())?);
                Expression::Call { func, arg }
            },
            "include" => {
                let target = Box::new(self.parse_expr(node.named_child(0).unwrap())?);
                Expression::Include(target)
            },

            // Binary Operators
            "add" | "subtract" | "multiply" | "divide" | "modulo" |
            "greater_than" | "greater_than_or_equal" | "equal" |
            "less_than" | "less_than_or_equal" | "and" | "or" |
            "set_all" | "pass_as_argument" => {
                let lhs = Box::new(self.parse_expr(node.named_child(0).unwrap())?);
                let rhs = Box::new(self.parse_expr(node.named_child(1).unwrap())?);
                let op = match node.kind() {
                    "add" => BinaryOperator::Add,
                    "subtract" => BinaryOperator::Subtract,
                    "multiply" => BinaryOperator::Multiply,
                    "divide" => BinaryOperator::Divide,
                    "modulo" => BinaryOperator::Modulo,
                    "greater_than" => BinaryOperator::GreaterThan,
                    "greater_than_or_equal" => BinaryOperator::GreaterThanOrEqual,
                    "equal" => BinaryOperator::Equal,
                    "less_than" => BinaryOperator::LessThan,
                    "less_than_or_equal" => BinaryOperator::LessThanOrEqual,
                    "and" => BinaryOperator::And,
                    "or" => BinaryOperator::Or,
                    "set_all" => BinaryOperator::PutAll,
                    "pass_as_argument" => BinaryOperator::PassAsArgument,
                    _ => unreachable!(),
                };
                Expression::BinaryOp { op, lhs, rhs }
            },

            // Unary Operators
            "negate" | "not" => {
                let expr = Box::new(self.parse_expr(node.named_child(0).unwrap())?);
                let op = match node.kind() {
                    "negate" => UnaryOperator::Negate,
                    "not" => UnaryOperator::Not,
                    _ => unreachable!(),
                };
                Expression::UnaryOp { op, expr }
            },

            // Transparent wrappers (like source_file)
            _ => {
                if node.named_child_count() == 1 {
                    return self.parse_expr(node.named_child(0).unwrap());
                }
                return Err(format!("Unrecognized syntax node: {}", node.kind()));
            }
        };

        Ok(Spanned::new(expr, span))
    }

    /// Parses a structure field[cite: 1]
    fn parse_structure_field(&self, node: tree_sitter::Node) -> Result<Spanned<StructField>, String> {
        let span = self.span(node);
        let named_count = node.named_child_count();

        let field = if named_count == 2 {
            // seq($._expression, "=", $._expression)[cite: 1]
            let key = self.parse_expr(node.named_child(0).unwrap())?;
            let value = self.parse_expr(node.named_child(1).unwrap())?;
            StructField::KeyValue(key, value)
        } else {
            // Check if there's an unnamed "=" child to differentiate ValueOnly vs Identifier
            let has_eq = node.children(&mut node.walk()).any(|c| c.kind() == "=");
            let expr = self.parse_expr(node.named_child(0).unwrap())?;

            if has_eq {
                StructField::ValueOnly(expr)
            } else {
                StructField::Identifier(expr)
            }
        };

        Ok(Spanned::new(field, span))
    }

    /// Parses a structural pattern. Because `_pattern` is hidden, its children bubble up[cite: 1].
    fn parse_pattern(&self, nodes: &[tree_sitter::Node]) -> Result<(Spanned<Pattern>, usize), String> {
        if nodes.is_empty() {
            return Err("Expected pattern, found nothing".into());
        }

        let first = nodes[0];
        let span = self.span(first);

        if first.kind() == "identifier" {
            if nodes.len() > 1 && nodes[1].kind() == "structural_pattern" {
                // BoundStructural: |ident: {a, b}|[cite: 1]
                let ident = self.text(first);
                let struct_span = self.span(nodes[1]);
                let fields = self.parse_structural_pattern(nodes[1])?;

                let combined_span = Span { end_line: struct_span.end_line, end_col: struct_span.end_col, ..span };
                let pat = Pattern::BoundStructural { identifier: ident, structure: fields };

                Ok((Spanned::new(pat, combined_span), 2))
            } else {
                // Simple Identifier: |ident|[cite: 1]
                Ok((Spanned::new(Pattern::Identifier(self.text(first)), span), 1))
            }
        } else if first.kind() == "structural_pattern" {
            // Structural: |{a, b}|[cite: 1]
            let fields = self.parse_structural_pattern(first)?;
            Ok((Spanned::new(Pattern::Structural(fields), span), 1))
        } else {
            Err(format!("Unexpected pattern starting node: {}", first.kind()))
        }
    }

    /// Parses the inner fields of a structural pattern[cite: 1].
    fn parse_structural_pattern(&self, node: tree_sitter::Node) -> Result<Vec<Spanned<PatternField>>, String> {
        let mut fields = Vec::new();
        let mut cursor = node.walk();

        for child in node.named_children(&mut cursor) {
            if child.kind() == "pattern_field" {
                let children: Vec<_> = child.named_children(&mut child.walk()).collect();

                let field_span = self.span(child);
                let field = if children.len() == 2 {
                    PatternField::BoundStructural {
                        identifier: self.text(children[0]),
                        structure: self.parse_structural_pattern(children[1])?,
                    }
                } else if children[0].kind() == "structural_pattern" {
                    PatternField::Structural(self.parse_structural_pattern(children[0])?)
                } else {
                    PatternField::Identifier(self.text(children[0]))
                };

                fields.push(Spanned::new(field, field_span));
            }
        }

        Ok(fields)
    }

    /// Parses a single `case` branch[cite: 1].
    fn parse_case(&self, node: tree_sitter::Node) -> Result<Spanned<CaseBranch>, String> {
        let span = self.span(node);
        let children: Vec<_> = node.named_children(&mut node.walk()).collect();

        // A case has a pattern, an optional guard, and a body[cite: 1].
        let (pattern, consumed) = self.parse_pattern(&children)?;

        let remaining = children.len() - consumed;
        let (guard, body_node) = if remaining == 2 {
            // Has guard and body
            let guard_expr = self.parse_expr(children[consumed])?;
            (Some(Box::new(guard_expr)), children[consumed + 1])
        } else {
            // Only body
            (None, children[consumed])
        };

        let body = Box::new(self.parse_expr(body_node)?);

        Ok(Spanned::new(CaseBranch { pattern, guard, body }, span))
    }
}
