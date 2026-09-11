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
    // S53 — savepoints
    Savepoint(String),
    ReleaseSavepoint(String),
    RollbackTo(String),
    // S56 — EXPLAIN ANALYZE
    ExplainAnalyze(Box<SelectStmt>),
    // S57 — prepared statements
    Prepare {
        name: String,
        stmt: Box<Stmt>,
    },
    ExecuteStmt {
        name: String,
        params: Vec<Expr>,
    },
    // S61 — isolation levels
    BeginIsolation {
        level: IsolationLevel,
    },
    // S65 — UDFs
    CreateFunction {
        name: String,
        param: String,
        body: String,
    },
    DropFunction {
        name: String,
    },
    // S66 — triggers
    CreateTrigger(TriggerDef),
    DropTrigger {
        name: String,
    },
    // S71
    ResetQueryStats,
    // S74
    CopyFrom {
        table: String,
        path: String,
        has_header: bool,
    },
    // S75
    CreateMatView {
        name: String,
        select: Box<SelectStmt>,
    },
    RefreshMatView {
        name: String,
    },
    DropMatView {
        name: String,
    },
    // S78 — Row-level security
    CreatePolicy {
        name: String,
        table: String,
        using_expr: Expr,
    },
    DropPolicy {
        name: String,
        table: String,
    },
    // S79 — Column masking
    CreateMask {
        name: String,
        table: String,
        col: String,
        mask_val: String,
    },
    DropMask {
        name: String,
    },
    // S80 — Audit log truncate
    TruncateAuditLog,
    // S81 — Token-based auth
    SetSessionToken {
        token: String,
    },
    RequireToken {
        table: String,
        token: String,
    },
    DropRequireToken {
        table: String,
    },
    // S82 — Rate limiting
    SetRateLimit {
        table: String,
        n: u64,
    },
    DropRateLimit {
        table: String,
    },
    // S83 — IP allowlist
    AddAllowedIp {
        ip: String,
    },
    RemoveAllowedIp {
        ip: String,
    },
    ShowAllowedIps,
    // S84 — TLS cert rotation
    RotateTlsCert {
        cert_path: String,
        key_path: String,
    },
    // S87 — Warnings toggle
    SetWarnings(bool),
    // S88 — replication slots
    CreateReplicationSlot {
        name: String,
        temporary: bool,
    },
    DropReplicationSlot {
        name: String,
    },
    StartReplication {
        slot: String,
        from_lsn: String,
    },
    // S90 — read-only mode
    SetReadOnly,
    SetReadWrite,
    // S91 — failover
    PromoteStandby,
    DemotePrimary,
    // S92 — WAL archiving
    SetArchivePath {
        path: String,
    },
    ArchiveNow,
    ShowArchiveStatus,
    // S93 — slow query log
    SetSlowQueryThreshold {
        ms: u64,
    },
    TruncateSlowLog,
    // S96 — auto-explain
    SetAutoExplainThreshold {
        ms: u64,
    },
    // S97 — wait events
    ResetWaitEvents,
    // S98 — flamegraph export
    ExplainFlamegraph(Box<SelectStmt>),
    // S100 — EXPLAIN BUFFERS
    ExplainBuffers(Box<SelectStmt>),
    // S101/S102 — psql/pgcli meta-commands
    MetaCmd {
        cmd: String,
        arg: Option<String>,
    },
    // S103 — cursor protocol
    DeclareCursor {
        name: String,
        select: Box<SelectStmt>,
    },
    FetchCursor {
        name: String,
        count: u64,
    },
    CloseCursor {
        name: String,
    },
    // S106 — statement timeout
    SetStatementTimeout {
        ms: u64,
    },
    // S108 — systemd integration
    SdNotify {
        state: String,
    },
    SystemdReady,
    SystemdStopping,
    // S109 — graceful/immediate shutdown
    ShutdownGraceful,
    ShutdownImmediate,
    // S110 — multi-database
    CreateDatabase {
        name: String,
    },
    DropDatabase {
        name: String,
    },
    UseDatabase {
        name: String,
    },
    ShowDatabases,
    // S111 — schema namespacing
    CreateSchema {
        name: String,
    },
    DropSchema {
        name: String,
    },
    SetSearchPath {
        schema: String,
    },
    ShowSchemas,
    // S112 — table partitioning
    CreatePartition {
        name: String,
        parent: String,
        range_from: String,
        range_to: String,
    },
    // S113 — online schema change (AlterTable with ONLINE is handled via AlterOp)
    // S114 — vacuum
    Vacuum {
        table: String,
        analyze: bool,
    },
    // S115 — ANALYZE (standalone)
    Analyze {
        table: String,
    },
    // S116 — connection limits
    SetMaxConnections {
        n: u64,
    },
    ShowMaxConnections,
    // S117 — resource groups
    CreateResourceGroup {
        name: String,
        cpu_pct: u64,
        memory_mb: u64,
    },
    DropResourceGroup {
        name: String,
    },
    AlterResourceGroup {
        name: String,
        cpu_pct: Option<u64>,
        memory_mb: Option<u64>,
    },
    AssignUserToGroup {
        user: String,
        group: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum AggFunc {
    Count, // COUNT(*)
    Sum,
    Min,
    Max,
}

/// Scalar function applied to a column or expression value.
#[derive(Debug, Clone, PartialEq)]
pub enum ScalarFunc {
    Upper,
    Lower,
    Length,
    Abs,
    Round,
    Ceil,
    Floor,
    Trim,
    Now,
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
    // S52
    DenseRank,
    Ntile(u64),
    PercentRank,
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
            WindowFunc::DenseRank => "dense_rank".to_owned(),
            WindowFunc::Ntile(n) => format!("ntile({})", n),
            WindowFunc::PercentRank => "percent_rank".to_owned(),
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
        // S77
        frame: Option<WindowFrame>,
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
    // S71 — CROSS JOIN (no ON condition)
    Cross,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JoinClause {
    pub kind: JoinKind,
    pub table: String,
    /// `ON left_table.left_col = right_table.right_col` (equality only)
    pub left_col: String,
    pub right_col: String,
}

// S69
#[derive(Debug, Clone, PartialEq)]
pub struct LateralJoin {
    pub alias: String,
    pub subquery: Box<SelectStmt>,
    pub correlation_col: String,
}

/// ORDER BY clause — either a plain column sort or a vector distance ranking.
#[derive(Debug, Clone, PartialEq)]
pub enum OrderBy {
    /// `ORDER BY col ASC|DESC`
    Col { name: String, asc: bool },
    /// `ORDER BY col1 ASC, col2 DESC, ...`
    Multi(Vec<(String, bool)>),
    /// `ORDER BY vec_distance(col, '[...]') [ASC]`
    VecDistance { col: String, query: Vec<f32> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectStmt {
    pub columns: Vec<SelectExpr>,
    pub table: String,
    pub joins: Vec<JoinClause>,
    pub where_clause: Option<Expr>,
    pub group_by: Vec<String>,
    pub having: Option<Expr>,
    pub order_by: Option<OrderBy>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub distinct: bool,
    // S40 — set operations
    pub set_op: Option<SetOp>,
    // S41 — CTEs (name, body, is_recursive)
    pub ctes: Vec<(String, Box<SelectStmt>, bool)>,
    // S68 — recursive CTE second parts
    pub recursive_parts: Vec<(String, Box<SelectStmt>)>,
    // S69 — lateral joins
    pub lateral_joins: Vec<LateralJoin>,
    // S67 — index hint
    pub index_hint: Option<String>,
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
    // S73 — bulk INSERT extra rows
    pub extra_rows: Vec<Vec<Expr>>,
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
    // S59
    pub default: Option<LitVal>,
    pub check: Option<String>,
}

// S60 — foreign key
#[derive(Debug, Clone, PartialEq)]
pub struct ForeignKey {
    pub col: String,
    pub ref_table: String,
    pub ref_col: String,
}

// S112 — partition definition
#[derive(Debug, Clone, PartialEq)]
pub enum PartitionStrategy {
    Range,
    List,
    Hash,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PartitionDef {
    pub strategy: PartitionStrategy,
    pub col: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateTableStmt {
    pub table: String,
    pub columns: Vec<ColTypeDef>,
    // S60
    pub foreign_keys: Vec<ForeignKey>,
    // S112
    pub partition_by: Option<PartitionDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DropTableStmt {
    pub table: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlterOp {
    AddColumn(ColTypeDef),
    // S113 — ONLINE flag for add column
    AddColumnOnline(ColTypeDef),
    DropColumn(String),
    // S58
    RenameColumn { from: String, to: String },
    ChangeType { col: String, new_type: String },
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

// S61 — isolation levels
#[derive(Debug, Clone, PartialEq)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

// S66 — triggers
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerTiming {
    Before,
    After,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TriggerEvent {
    Insert,
    Update,
    Delete,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TriggerDef {
    pub name: String,
    pub timing: TriggerTiming,
    pub event: TriggerEvent,
    pub table: String,
    pub func: String,
}

// S77 — window frame
#[derive(Debug, Clone, PartialEq)]
pub enum FrameUnit {
    Rows,
    Range,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FrameBound {
    UnboundedPreceding,
    CurrentRow,
    UnboundedFollowing,
    Preceding(u64),
    Following(u64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowFrame {
    pub unit: FrameUnit,
    pub start: FrameBound,
    pub end: FrameBound,
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
    /// `col LIKE 'pattern'` / `col NOT LIKE 'pattern'` / `col ILIKE 'pattern'`
    Like {
        col: String,
        pattern: String,
        negated: bool,
        // S70 — case-insensitive variant
        case_insensitive: bool,
    },
    // S39 — arithmetic
    Arith {
        op: ArithOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// `UPPER(col)`, `LOWER(col)`, `LENGTH(col)`, `ABS(expr)`, `ROUND(expr)`, etc.
    ScalarFn {
        func: ScalarFunc,
        arg: Box<Expr>,
    },
    // S43 — CAST
    Cast {
        expr: Box<Expr>,
        target: String,
    },
    // S49 — string functions
    StrFunc {
        name: String,
        args: Vec<Expr>,
    },
    // S50 — date/time
    DateExtract {
        part: String,
        expr: Box<Expr>,
    },
    // S54 — scalar subquery in expression context
    ScalarSubquery {
        subquery: Box<SelectStmt>,
    },
    // S57 — prepared statement placeholder
    Placeholder(usize),
    // S62 — match score
    MatchScore {
        col: String,
        term: String,
    },
    // S63 — JSON operators
    JsonGet {
        expr: Box<Expr>,
        key: String,
    },
    JsonGetText {
        expr: Box<Expr>,
        key: String,
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
