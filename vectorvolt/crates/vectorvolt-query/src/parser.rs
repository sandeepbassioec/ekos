//! Parse a token stream into a [`Query`] AST.

use crate::error::{QueryError, QueryResult};
use crate::lexer::{tokenize, Keyword, Token};

/// The AST for one query statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Query {
    /// `GET <pk>`
    Get { pk: u64 },

    /// `INSERT <pk> <v1> [v2 ...]`
    Insert { pk: u64, values: Vec<u64> },

    /// `DELETE <pk>`
    Delete { pk: u64 },

    /// `SCAN col=<n> lo=<lo> hi=<hi>`
    Scan { col: usize, lo: u64, hi: u64 },

    /// `SECONDARY col=<n> val=<v>`
    Secondary { col: u32, val: u64 },

    /// `BEGIN` — open a new transaction
    Begin,

    /// `COMMIT <txn_id>`
    Commit { txn_id: u64 },

    /// `ROLLBACK <txn_id>`
    Rollback { txn_id: u64 },

    /// `GET TXN <txn_id> <pk>`
    GetTxn { txn_id: u64, pk: u64 },

    /// `INSERT TXN <txn_id> <pk> <v1> [v2 ...]`
    InsertTxn {
        txn_id: u64,
        pk: u64,
        values: Vec<u64>,
    },

    /// `DELETE TXN <txn_id> <pk>`
    DeleteTxn { txn_id: u64, pk: u64 },

    /// `DESCRIBE` — return the table schema
    Describe,

    /// `STATS` — return engine statistics
    Stats,
}

/// Parse a query string into a [`Query`].
pub fn parse(input: &str) -> QueryResult<Query> {
    let tokens = tokenize(input);
    Parser::new(tokens).parse()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn expect_int(&mut self) -> QueryResult<u64> {
        match self.next() {
            Some(Token::Int(n)) => Ok(n),
            Some(t) => Err(QueryError::Syntax(format!("expected integer, got {t:?}"))),
            None => Err(QueryError::UnexpectedEof),
        }
    }

    fn expect_keyword(&mut self, kw: Keyword) -> QueryResult<()> {
        match self.next() {
            Some(Token::Keyword(k)) if k == kw => Ok(()),
            Some(t) => Err(QueryError::Syntax(format!("expected {kw:?}, got {t:?}"))),
            None => Err(QueryError::UnexpectedEof),
        }
    }

    fn expect_eq(&mut self) -> QueryResult<()> {
        match self.next() {
            Some(Token::Eq) => Ok(()),
            Some(t) => Err(QueryError::Syntax(format!("expected '=', got {t:?}"))),
            None => Err(QueryError::UnexpectedEof),
        }
    }

    fn parse(mut self) -> QueryResult<Query> {
        let kw = match self.next() {
            Some(Token::Keyword(k)) => k,
            Some(t) => return Err(QueryError::Syntax(format!("expected keyword, got {t:?}"))),
            None => return Err(QueryError::UnexpectedEof),
        };

        match kw {
            Keyword::Get => {
                if matches!(self.peek(), Some(Token::Keyword(Keyword::Txn))) {
                    self.next(); // consume TXN
                    let txn_id = self.expect_int()?;
                    let pk = self.expect_int()?;
                    Ok(Query::GetTxn { txn_id, pk })
                } else {
                    let pk = self.expect_int()?;
                    Ok(Query::Get { pk })
                }
            }

            Keyword::Insert => {
                if matches!(self.peek(), Some(Token::Keyword(Keyword::Txn))) {
                    self.next(); // consume TXN
                    let txn_id = self.expect_int()?;
                    let pk = self.expect_int()?;
                    let values = self.collect_ints();
                    Ok(Query::InsertTxn { txn_id, pk, values })
                } else {
                    let pk = self.expect_int()?;
                    let values = self.collect_ints();
                    Ok(Query::Insert { pk, values })
                }
            }

            Keyword::Delete => {
                if matches!(self.peek(), Some(Token::Keyword(Keyword::Txn))) {
                    self.next(); // consume TXN
                    let txn_id = self.expect_int()?;
                    let pk = self.expect_int()?;
                    Ok(Query::DeleteTxn { txn_id, pk })
                } else {
                    let pk = self.expect_int()?;
                    Ok(Query::Delete { pk })
                }
            }

            Keyword::Scan => {
                self.expect_keyword(Keyword::Col)?;
                self.expect_eq()?;
                let col = self.expect_int()? as usize;
                self.expect_keyword(Keyword::Lo)?;
                self.expect_eq()?;
                let lo = self.expect_int()?;
                self.expect_keyword(Keyword::Hi)?;
                self.expect_eq()?;
                let hi = self.expect_int()?;
                Ok(Query::Scan { col, lo, hi })
            }

            Keyword::Secondary => {
                self.expect_keyword(Keyword::Col)?;
                self.expect_eq()?;
                let col = self.expect_int()? as u32;
                self.expect_keyword(Keyword::Val)?;
                self.expect_eq()?;
                let val = self.expect_int()?;
                Ok(Query::Secondary { col, val })
            }

            Keyword::Begin => Ok(Query::Begin),
            Keyword::Commit => {
                let txn_id = self.expect_int()?;
                Ok(Query::Commit { txn_id })
            }
            Keyword::Rollback => {
                let txn_id = self.expect_int()?;
                Ok(Query::Rollback { txn_id })
            }

            Keyword::Describe => Ok(Query::Describe),
            Keyword::Stats => Ok(Query::Stats),

            other => Err(QueryError::Syntax(format!(
                "{other:?} is not a statement keyword"
            ))),
        }
    }

    /// Consume remaining `Int` tokens from the input.
    fn collect_ints(&mut self) -> Vec<u64> {
        let mut out = Vec::new();
        while let Some(Token::Int(n)) = self.peek() {
            out.push(*n);
            self.pos += 1;
        }
        out
    }
}
