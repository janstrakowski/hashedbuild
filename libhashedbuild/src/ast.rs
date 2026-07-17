use std::sync::Arc;
use std::path::PathBuf;

// ------------------------------------------------------------------
// Debug / Location Metadata
// ------------------------------------------------------------------

/// Represents a location in the source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub file: Arc<PathBuf>,
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

/// A wrapper that attaches source location metadata to any AST node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

// ------------------------------------------------------------------
// Core Expressions
// ------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Literals & Atomic Tokens
    Identifier(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Path(String),

    // Structures (record-like literals)
    Structure(Vec<Spanned<StructField>>),

    // Functions and Control Flow
    Function {
        pattern: Spanned<Pattern>,
        body: Box<Spanned<Expression>>,
    },
    If {
        condition: Box<Spanned<Expression>>,
        then_branch: Box<Spanned<Expression>>,
        else_branch: Box<Spanned<Expression>>,
    },
    Cases {
        target: Box<Spanned<Expression>>,
        branches: Vec<Spanned<CaseBranch>>,
        default: Option<Box<Spanned<Expression>>>,
    },

    // Operations
    BinaryOp {
        op: BinaryOperator,
        lhs: Box<Spanned<Expression>>,
        rhs: Box<Spanned<Expression>>,
    },
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Spanned<Expression>>,
    },

    // Access & Invocation
    FieldGet {
        target: Box<Spanned<Expression>>,
        field: String,
    },
    IndexGet {
        target: Box<Spanned<Expression>>,
        index: Box<Spanned<Expression>>,
    },
    Call {
        func: Box<Spanned<Expression>>,
        arg: Box<Spanned<Expression>>,
    },

    // Code-accessible
    Include(Box<Spanned<Expression>>),
}

// ------------------------------------------------------------------
// Sub-components (Structures, Patterns, Cases)
// ------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum StructField {
    /// E.g., `identifier`
    Identifier(Spanned<Expression>),
    /// E.g., `identifier = expression`
    KeyValue(Spanned<Expression>, Spanned<Expression>),
    /// E.g., `= expression`
    ValueOnly(Spanned<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// E.g., `|ident|`
    Identifier(String),
    /// E.g., `|{a, b}|`
    Structural(Vec<Spanned<PatternField>>),
    /// E.g., `|ident: {a, b}|`
    BoundStructural {
        identifier: String,
        structure: Vec<Spanned<PatternField>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternField {
    Identifier(String),
    Structural(Vec<Spanned<PatternField>>),
    BoundStructural {
        identifier: String,
        structure: Vec<Spanned<PatternField>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseBranch {
    pub pattern: Spanned<Pattern>,
    pub guard: Option<Box<Spanned<Expression>>>,
    pub body: Box<Spanned<Expression>>,
}

// ------------------------------------------------------------------
// Operators
// ------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    // Logical & Comparison
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    LessThan,
    LessThanOrEqual,
    And,
    Or,

    // Structural / General
    PutAll,          // Maps to the `|<` operator
    PassAsArgument,  // Maps to the `->` operator
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Negate, // Maps to `-` prefix
    Not,    // Maps to `!` prefix
}
