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
            Token::Delete => self.parse_delete(),
            Token::Create => self.parse_create_table(),
            Token::Drop => self.parse_drop_table(),
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
            match self.peek().clone() {
                Token::Star => {
                    self.advance();
                    columns.push(SelectExpr::Star);
                }
                Token::Ident(name) => {
                    self.advance();
                    columns.push(SelectExpr::Col(name));
                }
                t => return Err(ParseError::UnexpectedToken(t)),
            }
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
        let mut order_by = None;
        if self.peek() == &Token::Order {
            self.advance(); // ORDER
            self.expect(&Token::By)?;
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
            order_by = Some((col, asc));
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
            order_by,
            limit,
            distinct,
        }))
    }

    fn parse_insert(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // INSERT
        self.expect(&Token::Into)?;
        let table = self.expect_ident()?;
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
            Token::Ident(s) => Ok(Expr::Col(s)),
            Token::Integer(n) => Ok(Expr::Lit(LitVal::Int(n))),
            Token::Float(f) => Ok(Expr::Lit(LitVal::Float(f))),
            Token::Str(s) => Ok(Expr::Lit(LitVal::Str(s))),
            Token::Boolean(b) => Ok(Expr::Lit(LitVal::Bool(b))),
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
            Stmt::Select(s) => assert_eq!(s.order_by, Some(("pk".to_owned(), false))),
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
