use thiserror::Error;

use crate::sql::ast::*;
#[allow(unused_imports)]
use crate::sql::ast::{ArithOp, ScalarFunc, SetOp};
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

    #[allow(clippy::needless_return)]
    pub fn parse(&mut self) -> Result<Stmt, ParseError> {
        match self.peek().clone() {
            Token::Select => self.parse_select(),
            Token::With => self.parse_with_select(),
            Token::Insert => self.parse_insert(),
            Token::Update => self.parse_update(),
            Token::Delete => self.parse_delete(),
            Token::Create => self.parse_create(),
            Token::Drop => self.parse_drop(),
            Token::Alter => self.parse_alter_table(),
            Token::Rename => self.parse_rename_table(),
            Token::Explain => {
                self.advance(); // EXPLAIN
                if self.peek() == &Token::Analyze {
                    self.advance(); // ANALYZE
                    match self.parse_select()? {
                        Stmt::Select(s) => Ok(Stmt::ExplainAnalyze(Box::new(s))),
                        _ => Err(ParseError::UnexpectedToken(Token::Explain)),
                    }
                } else {
                    match self.parse_select()? {
                        Stmt::Select(s) => Ok(Stmt::Explain(Box::new(s))),
                        _ => Err(ParseError::UnexpectedToken(Token::Explain)),
                    }
                }
            }
            Token::Begin => {
                self.advance();
                // S61 — BEGIN ISOLATION LEVEL ...
                if self.peek() == &Token::Isolation {
                    self.advance(); // ISOLATION
                    self.expect(&Token::Level)?;
                    let level = match self.advance().clone() {
                        Token::KwRead => match self.advance().clone() {
                            Token::Committed => IsolationLevel::ReadCommitted,
                            Token::Uncommitted => IsolationLevel::ReadUncommitted,
                            t => return Err(ParseError::UnexpectedToken(t)),
                        },
                        Token::Repeatable => {
                            self.advance(); // READ
                            IsolationLevel::RepeatableRead
                        }
                        Token::Serializable => IsolationLevel::Serializable,
                        t => return Err(ParseError::UnexpectedToken(t)),
                    };
                    return Ok(Stmt::BeginIsolation { level });
                }
                Ok(Stmt::Begin)
            }
            Token::Commit => {
                self.advance();
                Ok(Stmt::Commit)
            }
            Token::Rollback => {
                self.advance();
                if self.peek() == &Token::To {
                    self.advance(); // TO
                    if self.peek() == &Token::Savepoint {
                        self.advance(); // SAVEPOINT (optional keyword)
                    }
                    let name = self.expect_ident()?;
                    Ok(Stmt::RollbackTo(name))
                } else {
                    Ok(Stmt::Rollback)
                }
            }
            Token::Savepoint => {
                self.advance();
                let name = self.expect_ident()?;
                Ok(Stmt::Savepoint(name))
            }
            Token::Release => {
                self.advance();
                if self.peek() == &Token::Savepoint {
                    self.advance();
                }
                let name = self.expect_ident()?;
                Ok(Stmt::ReleaseSavepoint(name))
            }
            Token::Prepare => {
                self.advance();
                let name = self.expect_ident()?;
                self.expect(&Token::As)?;
                let inner = self.parse()?;
                Ok(Stmt::Prepare {
                    name,
                    stmt: Box::new(inner),
                })
            }
            Token::Execute => {
                self.advance();
                let name = self.expect_ident()?;
                let mut params = Vec::new();
                if self.peek() == &Token::LParen {
                    self.advance();
                    if self.peek() != &Token::RParen {
                        loop {
                            params.push(self.parse_primary()?);
                            if self.peek() == &Token::Comma {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(&Token::RParen)?;
                }
                Ok(Stmt::ExecuteStmt { name, params })
            }
            Token::Reset => {
                self.advance(); // RESET
                match self.advance().clone() {
                    Token::Ident(s) if s.eq_ignore_ascii_case("QUERY") => {}
                    t => return Err(ParseError::UnexpectedToken(t)),
                }
                match self.advance().clone() {
                    Token::Ident(s) if s.eq_ignore_ascii_case("STATS") => {}
                    t => return Err(ParseError::UnexpectedToken(t)),
                }
                Ok(Stmt::ResetQueryStats)
            }
            Token::Refresh => {
                self.advance(); // REFRESH
                self.expect(&Token::Materialized)?;
                self.expect(&Token::View)?;
                let name = self.expect_ident()?;
                Ok(Stmt::RefreshMatView { name })
            }
            Token::Copy => {
                self.advance(); // COPY
                let table = self.expect_ident()?;
                self.expect(&Token::From)?;
                let path = match self.advance().clone() {
                    Token::Str(s) => s,
                    t => return Err(ParseError::UnexpectedToken(t)),
                };
                let mut has_header = false;
                if self.peek() == &Token::With
                    || matches!(self.peek(), Token::Ident(s) if s.eq_ignore_ascii_case("WITH"))
                {
                    self.advance(); // WITH
                    self.expect(&Token::LParen)?;
                    loop {
                        match self.peek().clone() {
                            Token::RParen => break,
                            Token::Format => {
                                self.advance();
                                self.advance(); // CSV
                            }
                            Token::Header => {
                                self.advance();
                                match self.advance().clone() {
                                    Token::Boolean(b) => has_header = b,
                                    Token::Ident(s) if s.eq_ignore_ascii_case("TRUE") => {
                                        has_header = true
                                    }
                                    Token::Ident(s) if s.eq_ignore_ascii_case("FALSE") => {
                                        has_header = false
                                    }
                                    _ => {}
                                }
                            }
                            _ => {
                                self.advance();
                            }
                        }
                        if self.peek() == &Token::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect(&Token::RParen)?;
                }
                Ok(Stmt::CopyFrom {
                    table,
                    path,
                    has_header,
                })
            }
            // S80 — TRUNCATE AUDIT LOG
            Token::Truncate => {
                self.advance(); // TRUNCATE
                match self.peek().clone() {
                    Token::Ident(s) if s.eq_ignore_ascii_case("AUDIT") => {
                        self.advance(); // AUDIT
                        match self.advance().clone() {
                            Token::Ident(s2) if s2.eq_ignore_ascii_case("LOG") => {}
                            t => return Err(ParseError::UnexpectedToken(t)),
                        }
                        return Ok(Stmt::TruncateAuditLog);
                    }
                    _ => {
                        // normal TRUNCATE TABLE
                        self.expect(&Token::Table)?;
                        let table = self.expect_ident()?;
                        return Ok(Stmt::Truncate(TruncateStmt { table }));
                    }
                }
            }
            // S81 — SET SESSION TOKEN 'val'
            Token::Set => {
                self.advance(); // SET
                match self.peek().clone() {
                    Token::Session => {
                        self.advance(); // SESSION
                        match self.advance().clone() {
                            Token::KwToken => {}
                            t => return Err(ParseError::UnexpectedToken(t)),
                        }
                        let token = match self.advance().clone() {
                            Token::Str(s) => s,
                            t => return Err(ParseError::UnexpectedToken(t)),
                        };
                        return Ok(Stmt::SetSessionToken { token });
                    }
                    Token::Rate => {
                        // SET RATE LIMIT ON table TO n PER SECOND
                        self.advance(); // RATE
                        match self.advance().clone() {
                            Token::Limit | Token::Ident(_) => {}
                            t => return Err(ParseError::UnexpectedToken(t)),
                        }
                        self.expect(&Token::On)?;
                        let table = self.expect_ident()?;
                        self.expect(&Token::To)?;
                        let n = match self.advance().clone() {
                            Token::Integer(n) => n as u64,
                            t => return Err(ParseError::UnexpectedToken(t)),
                        };
                        // consume optional PER SECOND
                        if self.peek() == &Token::Per {
                            self.advance();
                            if self.peek() == &Token::KwSecond {
                                self.advance();
                            }
                        }
                        return Ok(Stmt::SetRateLimit { table, n });
                    }
                    Token::Warnings => {
                        self.advance(); // WARNINGS
                        let on = match self.advance().clone() {
                            Token::On => true,
                            Token::Off => false,
                            Token::Ident(s) if s.eq_ignore_ascii_case("ON") => true,
                            Token::Ident(s) if s.eq_ignore_ascii_case("OFF") => false,
                            t => return Err(ParseError::UnexpectedToken(t)),
                        };
                        return Ok(Stmt::SetWarnings(on));
                    }
                    _ => {
                        return Err(ParseError::UnexpectedToken(self.peek().clone()));
                    }
                }
            }
            // S81 — REQUIRE TOKEN 'val' ON table
            Token::Require => {
                self.advance(); // REQUIRE
                match self.advance().clone() {
                    Token::KwToken => {}
                    t => return Err(ParseError::UnexpectedToken(t)),
                }
                let token = match self.advance().clone() {
                    Token::Str(s) => s,
                    t => return Err(ParseError::UnexpectedToken(t)),
                };
                self.expect(&Token::On)?;
                let table = self.expect_ident()?;
                return Ok(Stmt::RequireToken { table, token });
            }
            // S83 — ADD ALLOWED IP 'x.x.x.x'
            Token::Add => {
                self.advance(); // ADD
                match self.peek().clone() {
                    Token::Allowed => {
                        self.advance(); // ALLOWED
                        match self.advance().clone() {
                            Token::Ip => {}
                            t => return Err(ParseError::UnexpectedToken(t)),
                        }
                        let ip = match self.advance().clone() {
                            Token::Str(s) => s,
                            t => return Err(ParseError::UnexpectedToken(t)),
                        };
                        return Ok(Stmt::AddAllowedIp { ip });
                    }
                    _ => return Err(ParseError::UnexpectedToken(self.peek().clone())),
                }
            }
            // S83 — REMOVE ALLOWED IP 'x.x.x.x'
            Token::Ident(ref s) if s.eq_ignore_ascii_case("REMOVE") => {
                self.advance(); // REMOVE
                match self.advance().clone() {
                    Token::Allowed => {}
                    t => return Err(ParseError::UnexpectedToken(t)),
                }
                match self.advance().clone() {
                    Token::Ip => {}
                    t => return Err(ParseError::UnexpectedToken(t)),
                }
                let ip = match self.advance().clone() {
                    Token::Str(s) => s,
                    t => return Err(ParseError::UnexpectedToken(t)),
                };
                return Ok(Stmt::RemoveAllowedIp { ip });
            }
            // S83 — SHOW ALLOWED IPS
            Token::Ident(ref s) if s.eq_ignore_ascii_case("SHOW") => {
                self.advance(); // SHOW
                match self.advance().clone() {
                    Token::Allowed => {}
                    t => return Err(ParseError::UnexpectedToken(t)),
                }
                match self.advance().clone() {
                    Token::Ident(s) if s.eq_ignore_ascii_case("IPS") => {}
                    t => return Err(ParseError::UnexpectedToken(t)),
                }
                return Ok(Stmt::ShowAllowedIps);
            }
            // S84 — ROTATE TLS CERT 'path' KEY 'path'
            Token::Rotate => {
                self.advance(); // ROTATE
                match self.advance().clone() {
                    Token::Ident(s) if s.eq_ignore_ascii_case("TLS") => {}
                    t => return Err(ParseError::UnexpectedToken(t)),
                }
                match self.advance().clone() {
                    Token::Cert => {}
                    t => return Err(ParseError::UnexpectedToken(t)),
                }
                let cert_path = match self.advance().clone() {
                    Token::Str(s) => s,
                    t => return Err(ParseError::UnexpectedToken(t)),
                };
                match self.advance().clone() {
                    Token::Key => {}
                    t => return Err(ParseError::UnexpectedToken(t)),
                }
                let key_path = match self.advance().clone() {
                    Token::Str(s) => s,
                    t => return Err(ParseError::UnexpectedToken(t)),
                };
                return Ok(Stmt::RotateTlsCert {
                    cert_path,
                    key_path,
                });
            }
            t => Err(ParseError::UnexpectedToken(t)),
        }
    }

    /// S41/S68 — `WITH [RECURSIVE] name AS (SELECT ...) SELECT ...`
    fn parse_with_select(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // WITH
        let is_recursive = if self.peek() == &Token::Recursive {
            self.advance();
            true
        } else {
            false
        };
        let mut ctes = Vec::new();
        let mut recursive_parts = Vec::new();
        loop {
            let name = self.expect_ident()?;
            self.expect(&Token::As)?;
            self.expect(&Token::LParen)?;
            if is_recursive {
                // parse_select handles UNION ALL internally, so parse the whole CTE body
                let whole = self.parse_select()?;
                let whole_stmt = match whole {
                    Stmt::Select(s) => s,
                    _ => {
                        return Err(ParseError::Other(
                            "Recursive CTE body must be SELECT".to_owned(),
                        ));
                    }
                };
                self.expect(&Token::RParen)?;
                // Extract seed and recursive part from UNION ALL
                let (seed_stmt, rec_stmt) = match whole_stmt.set_op.clone() {
                    Some(SetOp::UnionAll(rec)) => {
                        let seed = SelectStmt {
                            set_op: None,
                            ..whole_stmt.clone()
                        };
                        (seed, *rec)
                    }
                    _ => (whole_stmt.clone(), whole_stmt),
                };
                ctes.push((name.clone(), Box::new(seed_stmt), true));
                recursive_parts.push((name, Box::new(rec_stmt)));
            } else {
                let inner = self.parse_select()?;
                let inner_stmt = match inner {
                    Stmt::Select(s) => s,
                    _ => return Err(ParseError::Other("CTE body must be SELECT".to_owned())),
                };
                self.expect(&Token::RParen)?;
                ctes.push((name, Box::new(inner_stmt), false));
            }
            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        let outer = self.parse_select()?;
        match outer {
            Stmt::Select(mut s) => {
                s.ctes = ctes;
                s.recursive_parts = recursive_parts;
                Ok(Stmt::Select(s))
            }
            _ => Err(ParseError::Other(
                "WITH must be followed by SELECT".to_owned(),
            )),
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
                Token::RowNumber => {
                    self.advance();
                    self.expect(&Token::LParen)?;
                    self.expect(&Token::RParen)?;
                    self.expect(&Token::Over)?;
                    self.expect(&Token::LParen)?;
                    let over_partition = if self.peek() == &Token::Partition {
                        self.advance(); // PARTITION
                        self.expect(&Token::By)?;
                        Some(self.expect_ident()?)
                    } else {
                        None
                    };
                    let over_order = if self.peek() == &Token::Order {
                        self.advance();
                        self.expect(&Token::By)?;
                        Some(self.expect_ident()?)
                    } else {
                        None
                    };
                    self.expect(&Token::RParen)?;
                    let alias = if self.peek() == &Token::As {
                        self.advance();
                        Some(self.expect_ident()?)
                    } else {
                        None
                    };
                    SelectExpr::WindowFunc {
                        func: crate::sql::ast::WindowFunc::RowNumber,
                        over_order,
                        over_partition,
                        alias,
                        frame: None,
                    }
                }
                Token::Rank => {
                    self.advance();
                    self.expect(&Token::LParen)?;
                    self.expect(&Token::RParen)?;
                    self.expect(&Token::Over)?;
                    self.expect(&Token::LParen)?;
                    let over_partition = if self.peek() == &Token::Partition {
                        self.advance();
                        self.expect(&Token::By)?;
                        Some(self.expect_ident()?)
                    } else {
                        None
                    };
                    // Optional ORDER BY inside OVER
                    let over_order = if self.peek() == &Token::Order {
                        self.advance(); // ORDER
                        self.expect(&Token::By)?;
                        Some(self.expect_ident()?)
                    } else {
                        None
                    };
                    self.expect(&Token::RParen)?;
                    let alias = if self.peek() == &Token::As {
                        self.advance();
                        Some(self.expect_ident()?)
                    } else {
                        None
                    };
                    SelectExpr::WindowFunc {
                        func: crate::sql::ast::WindowFunc::Rank,
                        over_order,
                        over_partition,
                        alias,
                        frame: None,
                    }
                }
                Token::Sum | Token::Min | Token::Max => {
                    let func_tok = self.advance().clone();
                    self.expect(&Token::LParen)?;
                    let col = self.expect_ident()?;
                    self.expect(&Token::RParen)?;
                    // Check for OVER — window function variant of SUM
                    if self.peek() == &Token::Over && matches!(func_tok, Token::Sum) {
                        self.advance(); // OVER
                        self.expect(&Token::LParen)?;
                        let over_partition = if self.peek() == &Token::Partition {
                            self.advance();
                            self.expect(&Token::By)?;
                            Some(self.expect_ident()?)
                        } else {
                            None
                        };
                        let over_order = if self.peek() == &Token::Order {
                            self.advance();
                            self.expect(&Token::By)?;
                            Some(self.expect_ident()?)
                        } else {
                            None
                        };
                        // S77 — optional ROWS/RANGE BETWEEN ... AND ...
                        let frame = self.try_parse_frame()?;
                        self.expect(&Token::RParen)?;
                        let alias = if self.peek() == &Token::As {
                            self.advance();
                            Some(self.expect_ident()?)
                        } else {
                            None
                        };
                        SelectExpr::WindowFunc {
                            func: crate::sql::ast::WindowFunc::WinSum(col),
                            over_order,
                            over_partition,
                            alias,
                            frame,
                        }
                    } else {
                        let func = match func_tok {
                            Token::Sum => AggFunc::Sum,
                            Token::Min => AggFunc::Min,
                            Token::Max => AggFunc::Max,
                            _ => unreachable!(),
                        };
                        SelectExpr::Agg {
                            func,
                            col: Some(col),
                        }
                    }
                }
                Token::LParen => {
                    // (SELECT ...) AS alias — scalar subquery in SELECT list
                    self.advance();
                    let inner = self.parse_select()?;
                    let subquery = match inner {
                        Stmt::Select(s) => s,
                        _ => return Err(ParseError::Other("subquery must be SELECT".to_owned())),
                    };
                    self.expect(&Token::RParen)?;
                    self.expect(&Token::As)?;
                    let alias = self.expect_ident()?;
                    SelectExpr::Subquery {
                        subquery: Box::new(subquery),
                        alias,
                    }
                }
                Token::Ident(name) => {
                    self.advance();
                    // S64 / UDF — function call in SELECT list: NAME(...) AS alias
                    if self.peek() == &Token::LParen {
                        // Back up the token position to let parse_primary re-read it
                        self.pos -= 1;
                        let expr = self.parse_primary()?;
                        let alias = if self.peek() == &Token::As {
                            self.advance();
                            self.expect_ident()?
                        } else {
                            "expr".to_owned()
                        };
                        SelectExpr::Computed { expr, alias }
                    } else if self.peek() == &Token::Dot {
                        self.advance(); // consume '.'
                        let col = self.expect_ident()?;
                        SelectExpr::QualifiedCol { table: name, col }
                    } else {
                        // Check for arithmetic operator → Computed
                        let arith_op = match self.peek() {
                            Token::Plus => Some(ArithOp::Add),
                            Token::Minus => Some(ArithOp::Sub),
                            Token::Star => Some(ArithOp::Mul),
                            Token::Slash => Some(ArithOp::Div),
                            Token::Percent => Some(ArithOp::Mod),
                            _ => None,
                        };
                        if let Some(op) = arith_op {
                            self.advance(); // consume operator
                            let right = self.parse_primary()?;
                            let expr = Expr::Arith {
                                op,
                                left: Box::new(Expr::Col(name.clone())),
                                right: Box::new(right),
                            };
                            let alias = if self.peek() == &Token::As {
                                self.advance();
                                self.expect_ident()?
                            } else {
                                name
                            };
                            SelectExpr::Computed { expr, alias }
                        } else {
                            SelectExpr::Col(name)
                        }
                    }
                }
                Token::Case
                | Token::Coalesce
                | Token::Null
                | Token::Upper
                | Token::Lower
                | Token::Length
                | Token::Abs
                | Token::Round
                | Token::Ceil
                | Token::Floor
                | Token::Cast
                | Token::Trim
                | Token::Now
                | Token::Substr
                | Token::Concat
                | Token::Instr
                | Token::Replace
                | Token::DateAdd
                | Token::DateDiff
                | Token::Extract
                | Token::MatchScore => {
                    // Computed expression in SELECT list — peek stays, parse_primary advances.
                    let expr = self.parse_primary()?;
                    let alias = if self.peek() == &Token::As {
                        self.advance();
                        self.expect_ident()?
                    } else {
                        "expr".to_owned()
                    };
                    SelectExpr::Computed { expr, alias }
                }
                Token::DenseRank => {
                    self.advance();
                    self.expect(&Token::LParen)?;
                    self.expect(&Token::RParen)?;
                    self.expect(&Token::Over)?;
                    self.expect(&Token::LParen)?;
                    let over_order = if self.peek() == &Token::Order {
                        self.advance();
                        self.expect(&Token::By)?;
                        Some(self.expect_ident()?)
                    } else {
                        None
                    };
                    self.expect(&Token::RParen)?;
                    let alias = if self.peek() == &Token::As {
                        self.advance();
                        Some(self.expect_ident()?)
                    } else {
                        None
                    };
                    SelectExpr::WindowFunc {
                        func: crate::sql::ast::WindowFunc::DenseRank,
                        over_order,
                        over_partition: None,
                        alias,
                        frame: None,
                    }
                }
                Token::Ntile => {
                    self.advance();
                    self.expect(&Token::LParen)?;
                    let n = match self.advance().clone() {
                        Token::Integer(n) => n as u64,
                        t => return Err(ParseError::UnexpectedToken(t)),
                    };
                    self.expect(&Token::RParen)?;
                    self.expect(&Token::Over)?;
                    self.expect(&Token::LParen)?;
                    self.expect(&Token::RParen)?;
                    let alias = if self.peek() == &Token::As {
                        self.advance();
                        Some(self.expect_ident()?)
                    } else {
                        None
                    };
                    SelectExpr::WindowFunc {
                        func: crate::sql::ast::WindowFunc::Ntile(n),
                        over_order: None,
                        over_partition: None,
                        alias,
                        frame: None,
                    }
                }
                Token::PercentRank => {
                    self.advance();
                    self.expect(&Token::LParen)?;
                    self.expect(&Token::RParen)?;
                    self.expect(&Token::Over)?;
                    self.expect(&Token::LParen)?;
                    let over_order = if self.peek() == &Token::Order {
                        self.advance();
                        self.expect(&Token::By)?;
                        Some(self.expect_ident()?)
                    } else {
                        None
                    };
                    self.expect(&Token::RParen)?;
                    let alias = if self.peek() == &Token::As {
                        self.advance();
                        Some(self.expect_ident()?)
                    } else {
                        None
                    };
                    SelectExpr::WindowFunc {
                        func: crate::sql::ast::WindowFunc::PercentRank,
                        over_order,
                        over_partition: None,
                        alias,
                        frame: None,
                    }
                }
                Token::Lead | Token::Lag => {
                    let func_tok = self.advance().clone();
                    self.expect(&Token::LParen)?;
                    let col = self.expect_ident()?;
                    self.expect(&Token::Comma)?;
                    let offset = match self.advance().clone() {
                        Token::Integer(n) => n as u64,
                        t => return Err(ParseError::UnexpectedToken(t)),
                    };
                    self.expect(&Token::RParen)?;
                    self.expect(&Token::Over)?;
                    self.expect(&Token::LParen)?;
                    self.expect(&Token::RParen)?;
                    let alias = if self.peek() == &Token::As {
                        self.advance();
                        Some(self.expect_ident()?)
                    } else {
                        None
                    };
                    let func = if matches!(func_tok, Token::Lead) {
                        crate::sql::ast::WindowFunc::Lead { col, offset }
                    } else {
                        crate::sql::ast::WindowFunc::Lag { col, offset }
                    };
                    SelectExpr::WindowFunc {
                        func,
                        over_order: None,
                        over_partition: None,
                        alias,
                        frame: None,
                    }
                }
                Token::FirstValue | Token::LastValue => {
                    let func_tok = self.advance().clone();
                    self.expect(&Token::LParen)?;
                    let col = self.expect_ident()?;
                    self.expect(&Token::RParen)?;
                    self.expect(&Token::Over)?;
                    self.expect(&Token::LParen)?;
                    self.expect(&Token::RParen)?;
                    let alias = if self.peek() == &Token::As {
                        self.advance();
                        Some(self.expect_ident()?)
                    } else {
                        None
                    };
                    let func = if matches!(func_tok, Token::FirstValue) {
                        crate::sql::ast::WindowFunc::FirstValue(col)
                    } else {
                        crate::sql::ast::WindowFunc::LastValue(col)
                    };
                    SelectExpr::WindowFunc {
                        func,
                        over_order: None,
                        over_partition: None,
                        alias,
                        frame: None,
                    }
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
        // S67 — USE INDEX(name) hint
        let mut index_hint = None;
        if self.peek() == &Token::Use {
            self.advance(); // USE
            self.expect(&Token::Index)?;
            self.expect(&Token::LParen)?;
            let hint = self.expect_ident()?;
            self.expect(&Token::RParen)?;
            index_hint = Some(hint);
        }
        let mut joins = Vec::new();
        loop {
            let kind = match self.peek().clone() {
                Token::Inner => {
                    self.advance();
                    self.expect(&Token::Join)?;
                    JoinKind::Inner
                }
                Token::Left => {
                    self.advance();
                    let _ = if self.peek() == &Token::Outer {
                        self.advance();
                        true
                    } else {
                        false
                    };
                    self.expect(&Token::Join)?;
                    JoinKind::Left
                }
                Token::Join => {
                    self.advance();
                    JoinKind::Inner
                }
                // S71 — CROSS JOIN (no ON condition)
                Token::Cross => {
                    self.advance();
                    self.expect(&Token::Join)?;
                    let join_table = self.expect_ident()?;
                    joins.push(JoinClause {
                        kind: JoinKind::Cross,
                        table: join_table,
                        left_col: String::new(),
                        right_col: String::new(),
                    });
                    continue;
                }
                _ => break,
            };
            let join_table = self.expect_ident()?;
            self.expect(&Token::On)?;
            // ON left_table.left_col = right_table.right_col
            // We accept any two qualified or unqualified col refs around '='
            let (_, left_col) = self.parse_qualified_col()?;
            self.expect(&Token::Eq)?;
            let (_, right_col) = self.parse_qualified_col()?;
            joins.push(JoinClause {
                kind,
                table: join_table,
                left_col,
                right_col,
            });
        }
        // S69 — LATERAL joins: , LATERAL (SELECT ...) AS alias
        let mut lateral_joins = Vec::new();
        while self.peek() == &Token::Comma {
            if self.pos + 1 < self.tokens.len() && self.tokens[self.pos + 1] == Token::Lateral {
                self.advance(); // consume comma
                self.advance(); // LATERAL
                self.expect(&Token::LParen)?;
                let sub = self.parse_select()?;
                let subquery = match sub {
                    Stmt::Select(s) => s,
                    _ => {
                        return Err(ParseError::Other(
                            "LATERAL subquery must be SELECT".to_owned(),
                        ));
                    }
                };
                self.expect(&Token::RParen)?;
                self.expect(&Token::As)?;
                let alias = self.expect_ident()?;
                lateral_joins.push(LateralJoin {
                    alias,
                    subquery: Box::new(subquery),
                    correlation_col: String::new(),
                });
            } else {
                break;
            }
        }
        let mut where_clause = None;
        if self.peek() == &Token::Where {
            self.advance();
            where_clause = Some(self.parse_expr()?);
        }
        let mut group_by: Vec<String> = Vec::new();
        if self.peek() == &Token::Group {
            self.advance(); // GROUP
            self.expect(&Token::By)?;
            loop {
                group_by.push(self.expect_ident()?);
                if self.peek() == &Token::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
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
                if self.peek() == &Token::Comma {
                    // Multi-column ORDER BY
                    let mut cols = vec![(col, asc)];
                    while self.peek() == &Token::Comma {
                        self.advance();
                        let c = self.expect_ident()?;
                        let a = match self.peek() {
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
                        cols.push((c, a));
                    }
                    order_by = Some(OrderBy::Multi(cols));
                } else {
                    order_by = Some(OrderBy::Col { name: col, asc });
                }
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
        let mut offset = None;
        if self.peek() == &Token::Offset {
            self.advance();
            match self.advance().clone() {
                Token::Integer(n) => offset = Some(n as u64),
                t => return Err(ParseError::UnexpectedToken(t)),
            }
        }
        let mut stmt = SelectStmt {
            columns,
            table,
            joins,
            where_clause,
            group_by,
            having,
            order_by,
            limit,
            offset,
            distinct,
            set_op: None,
            ctes: vec![],
            recursive_parts: vec![],
            lateral_joins,
            index_hint,
        };

        // S40 — check for UNION ALL / INTERSECT / EXCEPT
        match self.peek().clone() {
            Token::Union => {
                self.advance();
                self.expect(&Token::All)?;
                let right = match self.parse_select()? {
                    Stmt::Select(s) => s,
                    _ => return Err(ParseError::Other("UNION requires SELECT".to_owned())),
                };
                stmt.set_op = Some(SetOp::UnionAll(Box::new(right)));
            }
            Token::Intersect => {
                self.advance();
                let right = match self.parse_select()? {
                    Stmt::Select(s) => s,
                    _ => return Err(ParseError::Other("INTERSECT requires SELECT".to_owned())),
                };
                stmt.set_op = Some(SetOp::Intersect(Box::new(right)));
            }
            Token::Except => {
                self.advance();
                let right = match self.parse_select()? {
                    Stmt::Select(s) => s,
                    _ => return Err(ParseError::Other("EXCEPT requires SELECT".to_owned())),
                };
                stmt.set_op = Some(SetOp::Except(Box::new(right)));
            }
            _ => {}
        }

        Ok(Stmt::Select(stmt))
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

        // S73 — multi-row VALUES: (v1,...), (v2,...), ...
        let mut extra_rows: Vec<Vec<Expr>> = Vec::new();
        while self.peek() == &Token::Comma {
            if self.pos + 1 < self.tokens.len() && self.tokens[self.pos + 1] == Token::LParen {
                self.advance(); // comma
                self.expect(&Token::LParen)?;
                let mut row = Vec::new();
                loop {
                    row.push(self.parse_primary()?);
                    if self.peek() == &Token::Comma {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect(&Token::RParen)?;
                extra_rows.push(row);
            } else {
                break;
            }
        }

        // S44 — ON CONFLICT handling
        let mut on_conflict_update = None;
        let mut on_conflict_ignore = false;
        if self.peek() == &Token::On {
            self.advance(); // ON
            self.expect(&Token::Conflict)?;
            self.expect(&Token::Do)?;
            match self.peek().clone() {
                Token::Nothing => {
                    self.advance();
                    on_conflict_ignore = true;
                }
                Token::Update => {
                    self.advance();
                    self.expect(&Token::Set)?;
                    let mut updates = Vec::new();
                    loop {
                        let col = self.expect_ident()?;
                        self.expect(&Token::Eq)?;
                        let val = self.parse_primary()?;
                        updates.push((col, val));
                        if self.peek() == &Token::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    on_conflict_update = Some(updates);
                }
                t => return Err(ParseError::UnexpectedToken(t)),
            }
        }

        Ok(Stmt::Insert(InsertStmt {
            table,
            columns,
            values,
            or_replace,
            on_conflict_update,
            on_conflict_ignore,
            extra_rows,
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
                AlterOp::AddColumn(ColTypeDef {
                    name,
                    col_type,
                    default: None,
                    check: None,
                })
            }
            Token::Drop => {
                self.advance();
                self.expect(&Token::Column)?;
                let name = self.expect_ident()?;
                AlterOp::DropColumn(name)
            }
            // S58 — RENAME COLUMN old TO new
            Token::Rename => {
                self.advance();
                self.expect(&Token::Column)?;
                let from = self.expect_ident()?;
                self.expect(&Token::To)?;
                let to = self.expect_ident()?;
                AlterOp::RenameColumn { from, to }
            }
            // S58 — ALTER COLUMN col TYPE new_type
            Token::Alter => {
                self.advance();
                self.expect(&Token::Column)?;
                let col = self.expect_ident()?;
                // Expect TYPE keyword (stored as KwType or as Ident "TYPE")
                match self.advance().clone() {
                    Token::KwType => {}
                    Token::Ident(s) if s.eq_ignore_ascii_case("TYPE") => {}
                    t => return Err(ParseError::UnexpectedToken(t)),
                }
                let new_type = match self.advance().clone() {
                    Token::KwU64 => "u64".to_owned(),
                    Token::KwI64 => "i64".to_owned(),
                    Token::KwF64 => "f64".to_owned(),
                    Token::KwBool => "bool".to_owned(),
                    Token::KwText => "text".to_owned(),
                    Token::KwTimestamp => "timestamp".to_owned(),
                    Token::Ident(s) => s,
                    t => return Err(ParseError::UnexpectedToken(t)),
                };
                AlterOp::ChangeType { col, new_type }
            }
            t => return Err(ParseError::UnexpectedToken(t)),
        };
        Ok(Stmt::AlterTable(AlterTableStmt { table, op }))
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

    fn parse_create(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // CREATE
        match self.peek().clone() {
            // S65 — CREATE FUNCTION
            Token::Function => {
                self.advance();
                let name = self.expect_ident()?;
                self.expect(&Token::LParen)?;
                // param may be any identifier-like token
                let param = match self.advance().clone() {
                    Token::Ident(s) => s,
                    Token::Row => "row".to_owned(),
                    Token::Rows => "rows".to_owned(),
                    t => return Err(ParseError::UnexpectedToken(t)),
                };
                self.expect(&Token::RParen)?;
                self.expect(&Token::Returns)?;
                self.advance(); // skip return type token
                self.expect(&Token::As)?;
                let body = match self.advance().clone() {
                    Token::Str(s) => s,
                    t => return Err(ParseError::UnexpectedToken(t)),
                };
                return Ok(Stmt::CreateFunction { name, param, body });
            }
            // S66 — CREATE TRIGGER
            Token::Trigger => {
                self.advance();
                let name = self.expect_ident()?;
                let timing = match self.advance().clone() {
                    Token::Before => TriggerTiming::Before,
                    Token::After => TriggerTiming::After,
                    t => return Err(ParseError::UnexpectedToken(t)),
                };
                let event = match self.advance().clone() {
                    Token::Insert => TriggerEvent::Insert,
                    Token::Update => TriggerEvent::Update,
                    Token::Delete => TriggerEvent::Delete,
                    t => return Err(ParseError::UnexpectedToken(t)),
                };
                self.expect(&Token::On)?;
                let table = self.expect_ident()?;
                // FOR keyword — consume as ident or skip if not present
                if matches!(self.peek(), Token::Ident(s) if s.eq_ignore_ascii_case("FOR")) {
                    self.advance();
                }
                self.expect(&Token::Each)?;
                self.expect(&Token::Row)?;
                self.expect(&Token::Execute)?;
                self.expect(&Token::Function)?;
                let func = self.expect_ident()?;
                self.expect(&Token::LParen)?;
                self.expect(&Token::RParen)?;
                return Ok(Stmt::CreateTrigger(TriggerDef {
                    name,
                    timing,
                    event,
                    table,
                    func,
                }));
            }
            Token::Index => {
                self.advance(); // INDEX
                let index_name = self.expect_ident()?;
                self.expect(&Token::On)?;
                let table = self.expect_ident()?;
                self.expect(&Token::LParen)?;
                let col = self.expect_ident()?;
                self.expect(&Token::RParen)?;
                return Ok(Stmt::CreateIndex(CreateIndexStmt {
                    index_name,
                    table,
                    col,
                }));
            }
            Token::View => {
                self.advance(); // VIEW
                let name = self.expect_ident()?;
                self.expect(&Token::As)?;
                let sel = self.parse_select()?;
                let select_stmt = match sel {
                    Stmt::Select(s) => s,
                    _ => return Err(ParseError::Other("VIEW body must be SELECT".to_owned())),
                };
                return Ok(Stmt::CreateView(CreateViewStmt {
                    name,
                    select: Box::new(select_stmt),
                }));
            }
            // S78 — CREATE POLICY name ON table USING (expr)
            Token::Policy => {
                self.advance();
                let name = self.expect_ident()?;
                self.expect(&Token::On)?;
                let table = self.expect_ident()?;
                match self.advance().clone() {
                    Token::Using => {}
                    t => return Err(ParseError::UnexpectedToken(t)),
                }
                self.expect(&Token::LParen)?;
                let using_expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                return Ok(Stmt::CreatePolicy {
                    name,
                    table,
                    using_expr,
                });
            }
            // S79 — CREATE MASK name ON table.col USING 'mask_value'
            Token::Mask => {
                self.advance();
                let name = self.expect_ident()?;
                self.expect(&Token::On)?;
                let table = self.expect_ident()?;
                self.expect(&Token::Dot)?;
                let col = self.expect_ident()?;
                match self.advance().clone() {
                    Token::Using => {}
                    t => return Err(ParseError::UnexpectedToken(t)),
                }
                let mask_val = match self.advance().clone() {
                    Token::Str(s) => s,
                    t => return Err(ParseError::UnexpectedToken(t)),
                };
                return Ok(Stmt::CreateMask {
                    name,
                    table,
                    col,
                    mask_val,
                });
            }
            Token::Materialized => {
                self.advance(); // MATERIALIZED
                self.expect(&Token::View)?;
                let name = self.expect_ident()?;
                self.expect(&Token::As)?;
                let sel = self.parse_select()?;
                let select_stmt = match sel {
                    Stmt::Select(s) => s,
                    _ => {
                        return Err(ParseError::Other(
                            "MATERIALIZED VIEW body must be SELECT".to_owned(),
                        ));
                    }
                };
                return Ok(Stmt::CreateMatView {
                    name,
                    select: Box::new(select_stmt),
                });
            }
            _ => {}
        }
        self.expect(&Token::Table)?;
        let table = self.expect_ident()?;
        self.expect(&Token::LParen)?;
        let mut columns = Vec::new();
        let mut foreign_keys = Vec::new();
        loop {
            // S60 — FOREIGN KEY clause
            if self.peek() == &Token::Foreign {
                self.advance(); // FOREIGN
                self.expect(&Token::Key)?;
                self.expect(&Token::LParen)?;
                let col = self.expect_ident()?;
                self.expect(&Token::RParen)?;
                self.expect(&Token::References)?;
                let ref_table = self.expect_ident()?;
                self.expect(&Token::LParen)?;
                let ref_col = self.expect_ident()?;
                self.expect(&Token::RParen)?;
                foreign_keys.push(ForeignKey {
                    col,
                    ref_table,
                    ref_col,
                });
                if self.peek() == &Token::Comma {
                    self.advance();
                    continue;
                } else {
                    break;
                }
            }
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
            // S59 — DEFAULT and CHECK
            let mut default = None;
            let mut check = None;
            loop {
                match self.peek().clone() {
                    Token::Default => {
                        self.advance();
                        let lit = match self.advance().clone() {
                            Token::Integer(n) => LitVal::Int(n),
                            Token::Float(f) => LitVal::Float(f),
                            Token::Str(s) => LitVal::Str(s),
                            Token::Boolean(b) => LitVal::Bool(b),
                            Token::Null => LitVal::Null,
                            t => return Err(ParseError::UnexpectedToken(t)),
                        };
                        default = Some(lit);
                    }
                    Token::Check => {
                        self.advance();
                        self.expect(&Token::LParen)?;
                        // Collect tokens until matching RParen as a string
                        let mut depth = 1usize;
                        let mut check_str = String::new();
                        loop {
                            match self.advance().clone() {
                                Token::LParen => {
                                    depth += 1;
                                    check_str.push('(');
                                }
                                Token::RParen => {
                                    depth -= 1;
                                    if depth == 0 {
                                        break;
                                    }
                                    check_str.push(')');
                                }
                                Token::Ident(s) => {
                                    check_str.push_str(&s);
                                    check_str.push(' ');
                                }
                                Token::Integer(n) => {
                                    check_str.push_str(&n.to_string());
                                    check_str.push(' ');
                                }
                                Token::Float(f) => {
                                    check_str.push_str(&f.to_string());
                                    check_str.push(' ');
                                }
                                Token::Str(s) => {
                                    check_str.push('\'');
                                    check_str.push_str(&s);
                                    check_str.push('\'');
                                    check_str.push(' ');
                                }
                                Token::Gt => check_str.push_str("> "),
                                Token::Lt => check_str.push_str("< "),
                                Token::Gte => check_str.push_str(">= "),
                                Token::Lte => check_str.push_str("<= "),
                                Token::Eq => check_str.push_str("= "),
                                Token::Neq => check_str.push_str("<> "),
                                Token::Eof => break,
                                _ => {}
                            }
                        }
                        check = Some(check_str.trim().to_owned());
                    }
                    _ => break,
                }
            }
            columns.push(ColTypeDef {
                name,
                col_type,
                default,
                check,
            });
            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&Token::RParen)?;
        Ok(Stmt::CreateTable(CreateTableStmt {
            table,
            columns,
            foreign_keys,
        }))
    }

    /// Parse `table.col` or bare `col`, returning `(table_hint, col)`.
    fn parse_qualified_col(&mut self) -> Result<(Option<String>, String), ParseError> {
        let first = self.expect_ident()?;
        if self.peek() == &Token::Dot {
            self.advance();
            let col = self.expect_ident()?;
            Ok((Some(first), col))
        } else {
            Ok((None, first))
        }
    }

    fn parse_drop(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // DROP
        match self.peek().clone() {
            Token::Index => {
                self.advance();
                let index_name = self.expect_ident()?;
                return Ok(Stmt::DropIndex(DropIndexStmt { index_name }));
            }
            Token::View => {
                self.advance();
                let name = self.expect_ident()?;
                return Ok(Stmt::DropView(DropViewStmt { name }));
            }
            Token::Function => {
                self.advance();
                let name = self.expect_ident()?;
                return Ok(Stmt::DropFunction { name });
            }
            Token::Trigger => {
                self.advance();
                let name = self.expect_ident()?;
                return Ok(Stmt::DropTrigger { name });
            }
            Token::Materialized => {
                self.advance();
                self.expect(&Token::View)?;
                let name = self.expect_ident()?;
                return Ok(Stmt::DropMatView { name });
            }
            // S78 — DROP POLICY name ON table
            Token::Policy => {
                self.advance();
                let name = self.expect_ident()?;
                self.expect(&Token::On)?;
                let table = self.expect_ident()?;
                return Ok(Stmt::DropPolicy { name, table });
            }
            // S79 — DROP MASK name
            Token::Mask => {
                self.advance();
                let name = self.expect_ident()?;
                return Ok(Stmt::DropMask { name });
            }
            // S81 — DROP REQUIRE TOKEN ON table
            Token::Require => {
                self.advance(); // REQUIRE
                match self.advance().clone() {
                    Token::KwToken => {}
                    t => return Err(ParseError::UnexpectedToken(t)),
                }
                self.expect(&Token::On)?;
                let table = self.expect_ident()?;
                return Ok(Stmt::DropRequireToken { table });
            }
            // S82 — DROP RATE LIMIT ON table
            Token::Rate => {
                self.advance(); // RATE
                match self.advance().clone() {
                    Token::Limit | Token::Ident(_) => {}
                    t => return Err(ParseError::UnexpectedToken(t)),
                }
                self.expect(&Token::On)?;
                let table = self.expect_ident()?;
                return Ok(Stmt::DropRateLimit { table });
            }
            _ => {}
        }
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
        // EXISTS (SELECT ...)
        if self.peek() == &Token::Exists {
            self.advance();
            self.expect(&Token::LParen)?;
            let inner = self.parse_select()?;
            let subquery = match inner {
                Stmt::Select(s) => s,
                _ => return Err(ParseError::Other("EXISTS requires SELECT".to_owned())),
            };
            self.expect(&Token::RParen)?;
            return Ok(Expr::Exists {
                subquery: Box::new(subquery),
            });
        }
        let left = self.parse_primary()?;
        // col LIKE 'pattern' / col NOT LIKE 'pattern'
        if self.peek() == &Token::Not {
            self.advance(); // consume NOT
            self.expect(&Token::Like)?;
            let pattern = match self.advance().clone() {
                Token::Str(s) => s,
                t => return Err(ParseError::UnexpectedToken(t)),
            };
            let col = match left {
                Expr::Col(c) => c,
                _ => {
                    return Err(ParseError::Other(
                        "NOT LIKE left side must be a column".to_owned(),
                    ));
                }
            };
            return Ok(Expr::Like {
                col,
                pattern,
                negated: true,
                case_insensitive: false,
            });
        }
        if self.peek() == &Token::Like {
            self.advance();
            let pattern = match self.advance().clone() {
                Token::Str(s) => s,
                t => return Err(ParseError::UnexpectedToken(t)),
            };
            let col = match left {
                Expr::Col(c) => c,
                _ => {
                    return Err(ParseError::Other(
                        "LIKE left side must be a column".to_owned(),
                    ));
                }
            };
            return Ok(Expr::Like {
                col,
                pattern,
                negated: false,
                case_insensitive: false,
            });
        }
        // S70 — ILIKE (case-insensitive LIKE)
        if self.peek() == &Token::ILike {
            self.advance();
            let pattern = match self.advance().clone() {
                Token::Str(s) => s,
                t => return Err(ParseError::UnexpectedToken(t)),
            };
            let col = match left {
                Expr::Col(c) => c,
                _ => {
                    return Err(ParseError::Other(
                        "ILIKE left side must be a column".to_owned(),
                    ));
                }
            };
            return Ok(Expr::Like {
                col,
                pattern,
                negated: false,
                case_insensitive: true,
            });
        }
        // IS NULL / IS NOT NULL
        if self.peek() == &Token::Is {
            self.advance(); // IS
            let negated = if self.peek() == &Token::Not {
                self.advance();
                true
            } else {
                false
            };
            self.expect(&Token::Null)?;
            let col = match left {
                Expr::Col(c) => c,
                _ => return Err(ParseError::Other("IS NULL requires a column".to_owned())),
            };
            return Ok(Expr::IsNull { col, negated });
        }
        if self.peek() == &Token::In {
            self.advance();
            self.expect(&Token::LParen)?;
            // Check if this is IN (SELECT ...) or IN (literal, ...)
            if self.peek() == &Token::Select {
                let inner = self.parse_select()?;
                let subquery = match inner {
                    Stmt::Select(s) => s,
                    _ => return Err(ParseError::Other("IN subquery must be SELECT".to_owned())),
                };
                self.expect(&Token::RParen)?;
                let col = match left {
                    Expr::Col(c) => c,
                    _ => {
                        return Err(ParseError::Other(
                            "IN left side must be a column".to_owned(),
                        ));
                    }
                };
                return Ok(Expr::InSubquery {
                    col,
                    subquery: Box::new(subquery),
                });
            }
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
            Token::Abs | Token::Round | Token::Ceil | Token::Floor => {
                let func_tok = self.tokens[self.pos - 1].clone();
                self.expect(&Token::LParen)?;
                let arg = self.parse_primary()?;
                self.expect(&Token::RParen)?;
                let func = match func_tok {
                    Token::Abs => ScalarFunc::Abs,
                    Token::Round => ScalarFunc::Round,
                    Token::Ceil => ScalarFunc::Ceil,
                    Token::Floor => ScalarFunc::Floor,
                    _ => unreachable!(),
                };
                Ok(Expr::ScalarFn {
                    func,
                    arg: Box::new(arg),
                })
            }
            Token::Cast => {
                self.expect(&Token::LParen)?;
                let inner = self.parse_primary()?;
                self.expect(&Token::As)?;
                let target = match self.advance().clone() {
                    Token::KwU64 => "u64".to_owned(),
                    Token::KwI64 => "i64".to_owned(),
                    Token::KwF64 => "f64".to_owned(),
                    Token::KwBool => "bool".to_owned(),
                    Token::KwText => "text".to_owned(),
                    Token::KwTimestamp => "timestamp".to_owned(),
                    Token::Ident(s) => s,
                    t => return Err(ParseError::UnexpectedToken(t)),
                };
                self.expect(&Token::RParen)?;
                Ok(Expr::Cast {
                    expr: Box::new(inner),
                    target,
                })
            }
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
            // S62 — MATCH_SCORE(col, 'term')
            Token::MatchScore => {
                self.expect(&Token::LParen)?;
                let col = self.expect_ident()?;
                self.expect(&Token::Comma)?;
                let term = match self.advance().clone() {
                    Token::Str(s) => s,
                    t => return Err(ParseError::UnexpectedToken(t)),
                };
                self.expect(&Token::RParen)?;
                Ok(Expr::MatchScore { col, term })
            }
            Token::Ident(s) if s.eq_ignore_ascii_case("DATE_TRUNC") => {
                self.expect(&Token::LParen)?;
                let part = match self.advance().clone() {
                    Token::Str(s2) => s2,
                    Token::Ident(s2) => s2,
                    t => return Err(ParseError::UnexpectedToken(t)),
                };
                self.expect(&Token::Comma)?;
                let col = self.parse_primary()?;
                self.expect(&Token::RParen)?;
                Ok(Expr::StrFunc {
                    name: "DATE_TRUNC".to_owned(),
                    args: vec![Expr::Lit(LitVal::Str(part)), col],
                })
            }
            Token::Ident(s) if s.eq_ignore_ascii_case("TIME_BUCKET") => {
                self.expect(&Token::LParen)?;
                let interval = self.parse_primary()?;
                self.expect(&Token::Comma)?;
                let col = self.parse_primary()?;
                self.expect(&Token::RParen)?;
                Ok(Expr::StrFunc {
                    name: "TIME_BUCKET".to_owned(),
                    args: vec![interval, col],
                })
            }
            Token::Ident(s) => {
                // S65 — UDF/generic function call: ident(args...)
                if self.peek() == &Token::LParen {
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    if self.peek() != &Token::RParen {
                        loop {
                            args.push(self.parse_primary()?);
                            if self.peek() == &Token::Comma {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(&Token::RParen)?;
                    return Ok(Expr::StrFunc {
                        name: s.to_ascii_uppercase(),
                        args,
                    });
                }
                // S63 — JSON operators col->'key' and col->>'key'
                let base = if self.peek() == &Token::Dot {
                    self.advance(); // consume '.'
                    let col = self.expect_ident()?;
                    Expr::QualifiedCol { table: s, col }
                } else {
                    Expr::Col(s)
                };
                match self.peek().clone() {
                    Token::Arrow => {
                        self.advance();
                        let key = match self.advance().clone() {
                            Token::Str(k) => k,
                            Token::Ident(k) => k,
                            t => return Err(ParseError::UnexpectedToken(t)),
                        };
                        Ok(Expr::JsonGet {
                            expr: Box::new(base),
                            key,
                        })
                    }
                    Token::ArrowText => {
                        self.advance();
                        let key = match self.advance().clone() {
                            Token::Str(k) => k,
                            Token::Ident(k) => k,
                            t => return Err(ParseError::UnexpectedToken(t)),
                        };
                        Ok(Expr::JsonGetText {
                            expr: Box::new(base),
                            key,
                        })
                    }
                    _ => Ok(base),
                }
            }
            Token::Integer(n) => Ok(Expr::Lit(LitVal::Int(n))),
            Token::Float(f) => Ok(Expr::Lit(LitVal::Float(f))),
            Token::Str(s) => Ok(Expr::Lit(LitVal::Str(s))),
            Token::Boolean(b) => Ok(Expr::Lit(LitVal::Bool(b))),
            // Aggregate keywords may appear as column names in HAVING clauses.
            Token::Count => Ok(Expr::Col("count".to_owned())),
            Token::Sum => Ok(Expr::Col("sum".to_owned())),
            Token::Min => Ok(Expr::Col("min".to_owned())),
            Token::Max => Ok(Expr::Col("max".to_owned())),
            Token::Null => Ok(Expr::Lit(LitVal::Null)),
            Token::Coalesce => {
                self.expect(&Token::LParen)?;
                let mut args = Vec::new();
                loop {
                    args.push(self.parse_primary()?);
                    if self.peek() == &Token::Comma {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect(&Token::RParen)?;
                Ok(Expr::Coalesce { args })
            }
            tok @ (Token::Upper | Token::Lower | Token::Length) => {
                let func = match tok {
                    Token::Upper => crate::sql::ast::ScalarFunc::Upper,
                    Token::Lower => crate::sql::ast::ScalarFunc::Lower,
                    Token::Length => crate::sql::ast::ScalarFunc::Length,
                    _ => unreachable!(),
                };
                self.expect(&Token::LParen)?;
                let col = self.expect_ident()?;
                self.expect(&Token::RParen)?;
                Ok(Expr::ScalarFn {
                    func,
                    arg: Box::new(Expr::Col(col)),
                })
            }
            Token::Case => {
                let mut branches = Vec::new();
                while self.peek() == &Token::When {
                    self.advance(); // WHEN
                    let cond = self.parse_comparison()?;
                    self.expect(&Token::Then)?;
                    let then_val = self.parse_primary()?;
                    branches.push((cond, then_val));
                }
                let else_val = if self.peek() == &Token::Else {
                    self.advance();
                    Some(Box::new(self.parse_primary()?))
                } else {
                    None
                };
                self.expect(&Token::End)?;
                Ok(Expr::Case { branches, else_val })
            }
            Token::Trim => {
                self.expect(&Token::LParen)?;
                let arg = self.parse_primary()?;
                self.expect(&Token::RParen)?;
                Ok(Expr::ScalarFn {
                    func: ScalarFunc::Trim,
                    arg: Box::new(arg),
                })
            }
            Token::Now => {
                self.expect(&Token::LParen)?;
                self.expect(&Token::RParen)?;
                Ok(Expr::ScalarFn {
                    func: ScalarFunc::Now,
                    arg: Box::new(Expr::Lit(LitVal::Null)),
                })
            }
            Token::Substr => {
                self.expect(&Token::LParen)?;
                let s = self.parse_primary()?;
                self.expect(&Token::Comma)?;
                let start = self.parse_primary()?;
                let mut args = vec![s, start];
                if self.peek() == &Token::Comma {
                    self.advance();
                    args.push(self.parse_primary()?);
                }
                self.expect(&Token::RParen)?;
                Ok(Expr::StrFunc {
                    name: "SUBSTR".to_owned(),
                    args,
                })
            }
            Token::Concat => {
                self.expect(&Token::LParen)?;
                let mut args = Vec::new();
                loop {
                    args.push(self.parse_primary()?);
                    if self.peek() == &Token::Comma {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect(&Token::RParen)?;
                Ok(Expr::StrFunc {
                    name: "CONCAT".to_owned(),
                    args,
                })
            }
            Token::Instr => {
                self.expect(&Token::LParen)?;
                let s = self.parse_primary()?;
                self.expect(&Token::Comma)?;
                let sub = self.parse_primary()?;
                self.expect(&Token::RParen)?;
                Ok(Expr::StrFunc {
                    name: "INSTR".to_owned(),
                    args: vec![s, sub],
                })
            }
            Token::Replace => {
                self.expect(&Token::LParen)?;
                let s = self.parse_primary()?;
                self.expect(&Token::Comma)?;
                let from = self.parse_primary()?;
                self.expect(&Token::Comma)?;
                let to = self.parse_primary()?;
                self.expect(&Token::RParen)?;
                Ok(Expr::StrFunc {
                    name: "REPLACE".to_owned(),
                    args: vec![s, from, to],
                })
            }
            Token::DateAdd => {
                self.expect(&Token::LParen)?;
                let date = self.parse_primary()?;
                self.expect(&Token::Comma)?;
                let n = self.parse_primary()?;
                self.expect(&Token::RParen)?;
                Ok(Expr::StrFunc {
                    name: "DATE_ADD".to_owned(),
                    args: vec![date, n],
                })
            }
            Token::DateDiff => {
                self.expect(&Token::LParen)?;
                let d1 = self.parse_primary()?;
                self.expect(&Token::Comma)?;
                let d2 = self.parse_primary()?;
                self.expect(&Token::RParen)?;
                Ok(Expr::StrFunc {
                    name: "DATE_DIFF".to_owned(),
                    args: vec![d1, d2],
                })
            }
            Token::Extract => {
                self.expect(&Token::LParen)?;
                let part = self.expect_ident()?;
                self.expect(&Token::From)?;
                let expr = self.parse_primary()?;
                self.expect(&Token::RParen)?;
                Ok(Expr::DateExtract {
                    part,
                    expr: Box::new(expr),
                })
            }
            Token::Placeholder(n) => Ok(Expr::Placeholder(n)),
            t => Err(ParseError::UnexpectedToken(t)),
        }
    }

    fn try_parse_frame(&mut self) -> Result<Option<WindowFrame>, ParseError> {
        let unit = match self.peek() {
            Token::Rows => {
                self.advance();
                FrameUnit::Rows
            }
            Token::Range => {
                self.advance();
                FrameUnit::Range
            }
            _ => return Ok(None),
        };
        self.expect(&Token::Between)?;
        let start = self.parse_frame_bound()?;
        self.expect(&Token::And)?;
        let end = self.parse_frame_bound()?;
        Ok(Some(WindowFrame { unit, start, end }))
    }

    fn parse_frame_bound(&mut self) -> Result<FrameBound, ParseError> {
        match self.peek().clone() {
            Token::Unbounded => {
                self.advance();
                match self.advance().clone() {
                    Token::Preceding => Ok(FrameBound::UnboundedPreceding),
                    Token::Following => Ok(FrameBound::UnboundedFollowing),
                    t => Err(ParseError::UnexpectedToken(t)),
                }
            }
            Token::Current => {
                self.advance();
                self.expect(&Token::Row)?;
                Ok(FrameBound::CurrentRow)
            }
            Token::Integer(n) => {
                let n = n as u64;
                self.advance();
                match self.advance().clone() {
                    Token::Preceding => Ok(FrameBound::Preceding(n)),
                    Token::Following => Ok(FrameBound::Following(n)),
                    t => Err(ParseError::UnexpectedToken(t)),
                }
            }
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
