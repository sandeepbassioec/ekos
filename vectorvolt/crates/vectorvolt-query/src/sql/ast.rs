#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Select(SelectStmt),
    Insert(InsertStmt),
    Delete(DeleteStmt),
    CreateTable(CreateTableStmt),
    DropTable(DropTableStmt),
    Explain(Box<SelectStmt>),
    Begin,
    Commit,
    Rollback,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectExpr {
    Star,
    Col(String),
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
    pub where_clause: Option<Expr>,
    pub order_by: Option<OrderBy>,
    pub limit: Option<u64>,
    pub distinct: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertStmt {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<Expr>,
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
