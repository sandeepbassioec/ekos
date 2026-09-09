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
}

#[derive(Debug, Clone, PartialEq)]
pub enum AggFunc {
    Count, // COUNT(*)
    Sum,
    Min,
    Max,
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertStmt {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<Expr>,
    pub or_replace: bool,
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum LitVal {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
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
