#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Select(SelectStmt),
    Insert(InsertStmt),
    InsertSelect(InsertSelectStmt),
    Update(UpdateStmt),
    Delete(DeleteStmt),
    CreateTable(CreateTableStmt),
    DropTable(DropTableStmt),
    AlterTable(AlterTableStmt),
    Truncate(TruncateStmt),
    RenameTable(RenameTableStmt),
    Explain(Box<SelectStmt>),
    Begin,
    Commit,
    Rollback,
    // S42
    CreateIndex(CreateIndexStmt),
    DropIndex(DropIndexStmt),
    // S45
    CreateView(CreateViewStmt),
    DropView(DropViewStmt),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AggFunc {
    Count, // COUNT(*)
    Sum,
    Min,
    Max,
}

/// Window function kind for `func() OVER (...)` expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum WindowFunc {
    RowNumber,
    Rank,
    /// `SUM(col) OVER ()`
    WinSum(String),
    // S47
    Lead {
        col: String,
        offset: u64,
    },
    Lag {
        col: String,
        offset: u64,
    },
    FirstValue(String),
    LastValue(String),
}

impl WindowFunc {
    pub fn col_name(&self) -> String {
        match self {
            WindowFunc::RowNumber => "row_number".to_owned(),
            WindowFunc::Rank => "rank".to_owned(),
            WindowFunc::WinSum(col) => format!("win_sum({})", col),
            WindowFunc::Lead { col, offset } => format!("lead({},{})", col, offset),
            WindowFunc::Lag { col, offset } => format!("lag({},{})", col, offset),
            WindowFunc::FirstValue(col) => format!("first_value({})", col),
            WindowFunc::LastValue(col) => format!("last_value({})", col),
        }
    }
}

/// S39 arithmetic operator
#[derive(Debug, Clone, PartialEq)]
pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

/// S39 scalar function
#[derive(Debug, Clone, PartialEq)]
pub enum ScalarFunc {
    Abs,
    Round,
    Ceil,
    Floor,
}

/// S40 set operation
#[derive(Debug, Clone, PartialEq)]
pub enum SetOp {
    UnionAll(Box<SelectStmt>),
    Intersect(Box<SelectStmt>),
    Except(Box<SelectStmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectExpr {
    Star,
    Col(String),
    /// `t.col` — table-qualified column reference
    QualifiedCol {
        table: String,
        col: String,
    },
    /// `COUNT(*)`, `SUM(col)`, `MIN(col)`, `MAX(col)`
    Agg {
        func: AggFunc,
        col: Option<String>,
    },
    /// `(SELECT ...) AS alias` — scalar subquery in SELECT list
    Subquery {
        subquery: Box<SelectStmt>,
        alias: String,
    },
    /// `ROW_NUMBER() OVER ()`, `RANK() OVER (ORDER BY col)`, `SUM(col) OVER ()`
    WindowFunc {
        func: WindowFunc,
        over_order: Option<String>,
        over_partition: Option<String>,
        alias: Option<String>,
    },
    /// `CASE WHEN ... END AS alias`, `COALESCE(...) AS alias` — per-row computed expression
    Computed {
        expr: Expr,
        alias: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum JoinKind {
    Inner,
    Left,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JoinClause {
    pub kind: JoinKind,
    pub table: String,
    /// `ON left_table.left_col = right_table.right_col` (equality only)
    pub left_col: String,
    pub right_col: String,
}

/// ORDER BY clause — either a plain column sort or a vector distance ranking.
#[derive(Debug, Clone, PartialEq)]
pub enum OrderBy {
    /// `ORDER BY col ASC|DESC`
    Col { name: String, asc: bool },
    /// `ORDER BY vec_distance(col, '[...]') [ASC]`
    VecDistance { col: String, query: Vec<f32> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectStmt {
    pub columns: Vec<SelectExpr>,
    pub table: String,
    pub joins: Vec<JoinClause>,
    pub where_clause: Option<Expr>,
    pub group_by: Option<String>,
    pub having: Option<Expr>,
    pub order_by: Option<OrderBy>,
    pub limit: Option<u64>,
    pub distinct: bool,
    // S40 — set operations
    pub set_op: Option<SetOp>,
    // S41 — CTEs
    pub ctes: Vec<(String, Box<SelectStmt>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertStmt {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<Expr>,
    pub or_replace: bool,
    // S44 — UPSERT
    pub on_conflict_update: Option<Vec<(String, Expr)>>,
    pub on_conflict_ignore: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertSelectStmt {
    pub table: String,
    pub columns: Vec<String>,
    pub select: Box<SelectStmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateStmt {
    pub table: String,
    /// `(col_name, new_value_expr)` pairs from the SET clause.
    pub assignments: Vec<(String, Expr)>,
    pub where_clause: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeleteStmt {
    pub table: String,
    pub where_clause: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColTypeDef {
    pub name: String,
    pub col_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateTableStmt {
    pub table: String,
    pub columns: Vec<ColTypeDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DropTableStmt {
    pub table: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlterOp {
    AddColumn(ColTypeDef),
    DropColumn(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlterTableStmt {
    pub table: String,
    pub op: AlterOp,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TruncateStmt {
    pub table: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenameTableStmt {
    pub old_name: String,
    pub new_name: String,
}

// S42
#[derive(Debug, Clone, PartialEq)]
pub struct CreateIndexStmt {
    pub index_name: String,
    pub table: String,
    pub col: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DropIndexStmt {
    pub index_name: String,
}

// S45
#[derive(Debug, Clone, PartialEq)]
pub struct CreateViewStmt {
    pub name: String,
    pub select: Box<SelectStmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DropViewStmt {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Col(String),
    Lit(LitVal),
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// `MATCH(col, 'term')` — full-text search predicate.
    Match {
        col: String,
        term: String,
    },
    /// `t.col` — table-qualified column reference in expressions.
    QualifiedCol {
        table: String,
        col: String,
    },
    /// `col IN (v1, v2, ...)`
    In {
        col: String,
        values: Vec<LitVal>,
    },
    /// `col BETWEEN lo AND hi`
    Between {
        col: String,
        lo: LitVal,
        hi: LitVal,
    },
    /// `col IN (SELECT col FROM table WHERE ...)`
    InSubquery {
        col: String,
        subquery: Box<SelectStmt>,
    },
    /// `EXISTS (SELECT 1 FROM table WHERE ...)`
    Exists {
        subquery: Box<SelectStmt>,
    },
    /// `col IS NULL` / `col IS NOT NULL`
    IsNull {
        col: String,
        negated: bool,
    },
    /// `COALESCE(expr, expr, ...)`
    Coalesce {
        args: Vec<Expr>,
    },
    /// `CASE WHEN cond THEN val ... ELSE val END`
    Case {
        branches: Vec<(Expr, Expr)>,
        else_val: Option<Box<Expr>>,
    },
    // S39 — arithmetic
    Arith {
        op: ArithOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    // S39 — scalar functions
    ScalarFn {
        func: ScalarFunc,
        arg: Box<Expr>,
    },
    // S43 — CAST
    Cast {
        expr: Box<Expr>,
        target: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum LitVal {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    And,
    Or,
}
