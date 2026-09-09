use thiserror::Error;

use crate::sql::ast::*;
use crate::sql::lexer::Token;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unexpected token: {0:?}")]
    UnexpectedToken(Token),
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("{0}")]
    Other(String),
}

pub struct SqlParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl SqlParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> &Token {
        let t = self.tokens.get(self.pos).unwrap_or(&Token::Eof);
        self.pos += 1;
        t
    }

    fn expect(&mut self, expected: &Token) -> Result<(), ParseError> {
        let t = self.advance().clone();
        if &t == expected {
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(t))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.advance().clone() {
            Token::Ident(s) => Ok(s),
            t => Err(ParseError::UnexpectedToken(t)),
        }
    }

    pub fn parse(&mut self) -> Result<Stmt, ParseError> {
        match self.peek().clone() {
            Token::Select => self.parse_select(),
            Token::Insert => self.parse_insert(),
            Token::Update => self.parse_update(),
            Token::Delete => self.parse_delete(),
            Token::Create => self.parse_create_table(),
            Token::Drop => self.parse_drop_table(),
            Token::Alter => self.parse_alter_table(),
            Token::Truncate => self.parse_truncate(),
            Token::Rename => self.parse_rename_table(),
            Token::Explain => {
                self.advance(); // EXPLAIN
                match self.parse_select()? {
                    Stmt::Select(s) => Ok(Stmt::Explain(Box::new(s))),
                    _ => Err(ParseError::UnexpectedToken(Token::Explain)),
                }
            }
            Token::Begin => {
                self.advance();
                Ok(Stmt::Begin)
            }
            Token::Commit => {
                self.advance();
                Ok(Stmt::Commit)
            }
            Token::Rollback => {
                self.advance();
                Ok(Stmt::Rollback)
            }
            t => Err(ParseError::UnexpectedToken(t)),
        }
    }

    fn parse_select(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // SELECT
        let distinct = if self.peek() == &Token::Distinct {
            self.advance();
            true
        } else {
            false
        };
        let mut columns = Vec::new();
        loop {
            let expr = match self.peek().clone() {
                Token::Star => {
                    self.advance();
                    SelectExpr::Star
                }
                Token::Count => {
                    self.advance();
                    self.expect(&Token::LParen)?;
                    self.expect(&Token::Star)?;
                    self.expect(&Token::RParen)?;
                    SelectExpr::Agg {
                        func: AggFunc::Count,
                        col: None,
                    }
                }
                Token::Sum | Token::Min | Token::Max => {
                    let func = match self.advance().clone() {
                        Token::Sum => AggFunc::Sum,
                        Token::Min => AggFunc::Min,
                        Token::Max => AggFunc::Max,
                        _ => unreachable!(),
                    };
                    self.expect(&Token::LParen)?;
                    let col = self.expect_ident()?;
                    self.expect(&Token::RParen)?;
                    SelectExpr::Agg {
                        func,
                        col: Some(col),
                    }
                }
                Token::Ident(name) => {
                    self.advance();
                    SelectExpr::Col(name)
                }
                t => return Err(ParseError::UnexpectedToken(t)),
            };
            columns.push(expr);
            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&Token::From)?;
        let table = self.expect_ident()?;
        let mut where_clause = None;
        if self.peek() == &Token::Where {
            self.advance();
            where_clause = Some(self.parse_expr()?);
        }
        let mut group_by = None;
        if self.peek() == &Token::Group {
            self.advance(); // GROUP
            self.expect(&Token::By)?;
            group_by = Some(self.expect_ident()?);
        }
        let mut having = None;
        if self.peek() == &Token::Having {
            self.advance();
            having = Some(self.parse_expr()?);
        }
        let mut order_by = None;
        if self.peek() == &Token::Order {
            self.advance(); // ORDER
            self.expect(&Token::By)?;
            if self.peek() == &Token::VecDistance {
                // ORDER BY vec_distance(col, '[1.0,2.0,...]')
                self.advance(); // VEC_DISTANCE
                self.expect(&Token::LParen)?;
                let col = self.expect_ident()?;
                self.expect(&Token::Comma)?;
                let vec_str = match self.advance().clone() {
                    Token::Str(s) => s,
                    t => return Err(ParseError::UnexpectedToken(t)),
                };
                self.expect(&Token::RParen)?;
                let query =
                    crate::sql::vec_store::parse_vec_literal(&vec_str).ok_or_else(|| {
                        ParseError::Other(format!("invalid vector literal: '{vec_str}'"))
                    })?;
                order_by = Some(OrderBy::VecDistance { col, query });
            } else {
                let col = self.expect_ident()?;
                let asc = match self.peek() {
                    Token::Desc => {
                        self.advance();
                        false
                    }
                    Token::Asc => {
                        self.advance();
                        true
                    }
                    _ => true,
                };
                order_by = Some(OrderBy::Col { name: col, asc });
            }
        }
        let mut limit = None;
        if self.peek() == &Token::Limit {
            self.advance();
            match self.advance().clone() {
                Token::Integer(n) => limit = Some(n as u64),
                t => return Err(ParseError::UnexpectedToken(t)),
            }
        }
        Ok(Stmt::Select(SelectStmt {
            columns,
            table,
            where_clause,
            group_by,
            having,
            order_by,
            limit,
            distinct,
        }))
    }

    fn parse_insert(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // INSERT
        let or_replace = if self.peek() == &Token::Or {
            self.advance();
            self.expect(&Token::Replace)?;
            true
        } else {
            false
        };
        self.expect(&Token::Into)?;
        let table = self.expect_ident()?;
        // INSERT INTO t SELECT ...
        if self.peek() == &Token::Select {
            let sel = self.parse_select()?;
            if let Stmt::Select(s) = sel {
                return Ok(Stmt::InsertSelect(InsertSelectStmt {
                    table,
                    columns: vec![],
                    select: Box::new(s),
                }));
            }
        }
        self.expect(&Token::LParen)?;
        let mut columns = Vec::new();
        loop {
            columns.push(self.expect_ident()?);
            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&Token::RParen)?;
        self.expect(&Token::Values)?;
        self.expect(&Token::LParen)?;
        let mut values = Vec::new();
        loop {
            values.push(self.parse_primary()?);
            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&Token::RParen)?;
        Ok(Stmt::Insert(InsertStmt {
            table,
            columns,
            values,
            or_replace,
        }))
    }

    fn parse_alter_table(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // ALTER
        self.expect(&Token::Table)?;
        let table = self.expect_ident()?;
        let op = match self.peek().clone() {
            Token::Add => {
                self.advance();
                self.expect(&Token::Column)?;
                let name = self.expect_ident()?;
                let col_type = match self.advance().clone() {
                    Token::KwU64 => "u64".to_owned(),
                    Token::KwI64 => "i64".to_owned(),
                    Token::KwF64 => "f64".to_owned(),
                    Token::KwBool => "bool".to_owned(),
                    Token::KwText => "text".to_owned(),
                    Token::KwTimestamp => "timestamp".to_owned(),
                    Token::Ident(s) => s,
                    t => return Err(ParseError::UnexpectedToken(t)),
                };
                AlterOp::AddColumn(ColTypeDef { name, col_type })
            }
            Token::Drop => {
                self.advance();
                self.expect(&Token::Column)?;
                let name = self.expect_ident()?;
                AlterOp::DropColumn(name)
            }
            t => return Err(ParseError::UnexpectedToken(t)),
        };
        Ok(Stmt::AlterTable(AlterTableStmt { table, op }))
    }

    fn parse_truncate(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // TRUNCATE
        self.expect(&Token::Table)?;
        let table = self.expect_ident()?;
        Ok(Stmt::Truncate(TruncateStmt { table }))
    }

    fn parse_rename_table(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // RENAME
        self.expect(&Token::Table)?;
        let old_name = self.expect_ident()?;
        self.expect(&Token::To)?;
        let new_name = self.expect_ident()?;
        Ok(Stmt::RenameTable(RenameTableStmt { old_name, new_name }))
    }

    fn parse_update(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // UPDATE
        let table = self.expect_ident()?;
        self.expect(&Token::Set)?;
        let mut assignments = Vec::new();
        loop {
            let col = self.expect_ident()?;
            self.expect(&Token::Eq)?;
            let val = self.parse_primary()?;
            assignments.push((col, val));
            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        let where_clause = if self.peek() == &Token::Where {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };
        Ok(Stmt::Update(UpdateStmt {
            table,
            assignments,
            where_clause,
        }))
    }

    fn parse_delete(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // DELETE
        self.expect(&Token::From)?;
        let table = self.expect_ident()?;
        let where_clause = if self.peek() == &Token::Where {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };
        Ok(Stmt::Delete(DeleteStmt {
            table,
            where_clause,
        }))
    }

    fn parse_create_table(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // CREATE
        self.expect(&Token::Table)?;
        let table = self.expect_ident()?;
        self.expect(&Token::LParen)?;
        let mut columns = Vec::new();
        loop {
            let name = self.expect_ident()?;
            let col_type = match self.advance().clone() {
                Token::KwU64 => "u64".to_owned(),
                Token::KwI64 => "i64".to_owned(),
                Token::KwF64 => "f64".to_owned(),
                Token::KwBool => "bool".to_owned(),
                Token::KwText => "text".to_owned(),
                Token::KwTimestamp => "timestamp".to_owned(),
                Token::Ident(s) => {
                    // Handle parameterized types like vector(N)
                    if self.peek() == &Token::LParen {
                        self.advance(); // consume '('
                        let dim = match self.advance().clone() {
                            Token::Integer(n) => n,
                            t => return Err(ParseError::UnexpectedToken(t)),
                        };
                        self.expect(&Token::RParen)?;
                        format!("{s}({dim})")
                    } else {
                        s
                    }
                }
                t => return Err(ParseError::UnexpectedToken(t)),
            };
            columns.push(ColTypeDef { name, col_type });
            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&Token::RParen)?;
        Ok(Stmt::CreateTable(CreateTableStmt { table, columns }))
    }

    fn parse_drop_table(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // DROP
        self.expect(&Token::Table)?;
        let table = self.expect_ident()?;
        Ok(Stmt::DropTable(DropTableStmt { table }))
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_comparison()?;
        if self.peek() == &Token::And {
            self.advance();
            let right = self.parse_comparison()?;
            return Ok(Expr::BinOp {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        if self.peek() == &Token::Or {
            self.advance();
            let right = self.parse_comparison()?;
            return Ok(Expr::BinOp {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_primary()?;
        if self.peek() == &Token::In {
            self.advance();
            self.expect(&Token::LParen)?;
            let mut values = Vec::new();
            loop {
                let expr = self.parse_primary()?;
                match expr {
                    Expr::Lit(v) => values.push(v),
                    _ => return Err(ParseError::Other("IN values must be literals".to_owned())),
                }
                if self.peek() == &Token::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(&Token::RParen)?;
            let col = match left {
                Expr::Col(c) => c,
                _ => {
                    return Err(ParseError::Other(
                        "IN left side must be a column".to_owned(),
                    ));
                }
            };
            return Ok(Expr::In { col, values });
        }
        if self.peek() == &Token::Between {
            self.advance();
            let lo_expr = self.parse_primary()?;
            let lo = match lo_expr {
                Expr::Lit(v) => v,
                _ => return Err(ParseError::Other("BETWEEN lo must be a literal".to_owned())),
            };
            self.expect(&Token::And)?;
            let hi_expr = self.parse_primary()?;
            let hi = match hi_expr {
                Expr::Lit(v) => v,
                _ => return Err(ParseError::Other("BETWEEN hi must be a literal".to_owned())),
            };
            let col = match left {
                Expr::Col(c) => c,
                _ => {
                    return Err(ParseError::Other(
                        "BETWEEN left side must be a column".to_owned(),
                    ));
                }
            };
            return Ok(Expr::Between { col, lo, hi });
        }
        let op = match self.peek().clone() {
            Token::Eq => BinOp::Eq,
            Token::Neq => BinOp::Neq,
            Token::Lt => BinOp::Lt,
            Token::Lte => BinOp::Lte,
            Token::Gt => BinOp::Gt,
            Token::Gte => BinOp::Gte,
            _ => return Ok(left),
        };
        self.advance();
        let right = self.parse_primary()?;
        Ok(Expr::BinOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.advance().clone() {
            Token::Match => {
                // MATCH(col, 'term')
                self.expect(&Token::LParen)?;
                let col = self.expect_ident()?;
                self.expect(&Token::Comma)?;
                let term = match self.advance().clone() {
                    Token::Str(s) => s,
                    t => return Err(ParseError::UnexpectedToken(t)),
                };
                self.expect(&Token::RParen)?;
                Ok(Expr::Match { col, term })
            }
            Token::Ident(s) => Ok(Expr::Col(s)),
            Token::Integer(n) => Ok(Expr::Lit(LitVal::Int(n))),
            Token::Float(f) => Ok(Expr::Lit(LitVal::Float(f))),
            Token::Str(s) => Ok(Expr::Lit(LitVal::Str(s))),
            Token::Boolean(b) => Ok(Expr::Lit(LitVal::Bool(b))),
            // Aggregate keywords may appear as column names in HAVING clauses.
            Token::Count => Ok(Expr::Col("count".to_owned())),
            Token::Sum => Ok(Expr::Col("sum".to_owned())),
            Token::Min => Ok(Expr::Col("min".to_owned())),
            Token::Max => Ok(Expr::Col("max".to_owned())),
            t => Err(ParseError::UnexpectedToken(t)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::lexer::Lexer;

    fn parse(sql: &str) -> Stmt {
        let mut lex = Lexer::new(sql);
        let tokens = lex.tokenize().unwrap();
        let mut p = SqlParser::new(tokens);
        p.parse().unwrap()
    }

    #[test]
    fn test_select_star() {
        let s = parse("SELECT * FROM t");
        assert!(matches!(s, Stmt::Select(_)));
    }

    #[test]
    fn test_select_cols() {
        match parse("SELECT a, b FROM t") {
            Stmt::Select(s) => assert_eq!(s.columns.len(), 2),
            _ => panic!(),
        }
    }

    #[test]
    fn test_select_where() {
        match parse("SELECT * FROM t WHERE pk = 1") {
            Stmt::Select(s) => assert!(s.where_clause.is_some()),
            _ => panic!(),
        }
    }

    #[test]
    fn test_select_order_desc() {
        match parse("SELECT * FROM t ORDER BY pk DESC") {
            Stmt::Select(s) => assert_eq!(
                s.order_by,
                Some(OrderBy::Col {
                    name: "pk".to_owned(),
                    asc: false
                })
            ),
            _ => panic!(),
        }
    }

    #[test]
    fn test_select_limit() {
        match parse("SELECT * FROM t LIMIT 10") {
            Stmt::Select(s) => assert_eq!(s.limit, Some(10)),
            _ => panic!(),
        }
    }

    #[test]
    fn test_insert() {
        match parse("INSERT INTO t (pk, val) VALUES (1, 42)") {
            Stmt::Insert(i) => {
                assert_eq!(i.table, "t");
                assert_eq!(i.columns.len(), 2);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_delete_where() {
        match parse("DELETE FROM t WHERE pk = 1") {
            Stmt::Delete(d) => {
                assert_eq!(d.table, "t");
                assert!(d.where_clause.is_some());
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_create_table() {
        match parse("CREATE TABLE t (pk U64, val U64)") {
            Stmt::CreateTable(c) => {
                assert_eq!(c.table, "t");
                assert_eq!(c.columns.len(), 2);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_drop_table() {
        match parse("DROP TABLE t") {
            Stmt::DropTable(d) => assert_eq!(d.table, "t"),
            _ => panic!(),
        }
    }

    #[test]
    fn test_where_and() {
        match parse("SELECT * FROM t WHERE pk > 1 AND val < 100") {
            Stmt::Select(s) => assert!(matches!(
                s.where_clause,
                Some(Expr::BinOp { op: BinOp::And, .. })
            )),
            _ => panic!(),
        }
    }

    #[test]
    fn test_distinct() {
        match parse("SELECT DISTINCT a FROM t") {
            Stmt::Select(s) => assert!(s.distinct),
            _ => panic!(),
        }
    }

    #[test]
    fn test_parse_error_invalid_sql() {
        let mut lex = Lexer::new("BLAH BLAH");
        let tokens = lex.tokenize().unwrap();
        let mut p = SqlParser::new(tokens);
        assert!(p.parse().is_err());
    }

    #[test]
    fn test_where_string_literal() {
        match parse("SELECT * FROM t WHERE name = 'alice'") {
            Stmt::Select(s) => assert!(matches!(s.where_clause, Some(Expr::BinOp { .. }))),
            _ => panic!(),
        }
    }
}
