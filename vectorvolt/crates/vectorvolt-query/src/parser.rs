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

    // ── Layer 43: analytical ──────────────────────────────────────────────────
    /// `COUNT col=<n> lo=<lo> hi=<hi>`
    Count { col: usize, lo: u64, hi: u64 },

    /// `SUM col=<n> lo=<lo> hi=<hi>`
    Sum { col: usize, lo: u64, hi: u64 },

    /// `MIN col=<n> lo=<lo> hi=<hi>`
    Min { col: usize, lo: u64, hi: u64 },

    /// `MAX col=<n> lo=<lo> hi=<hi>`
    Max { col: usize, lo: u64, hi: u64 },

    /// `AVG col=<n> lo=<lo> hi=<hi>`
    Avg { col: usize, lo: u64, hi: u64 },

    /// `SCAN SORTED col=<n> lo=<lo> hi=<hi> sort=<s> asc=<0|1> limit=<l>`
    ScanSorted {
        col: usize,
        lo: u64,
        hi: u64,
        sort: usize,
        asc: bool,
        limit: usize,
    },

    /// `SCAN PAGE col=<n> lo=<lo> hi=<hi> after=<pk> limit=<l>`
    ScanPage {
        col: usize,
        lo: u64,
        hi: u64,
        after: u64,
        limit: usize,
    },

    // ── Layer 43: write/mutate ────────────────────────────────────────────────
    /// `UPSERT <pk> <v1> [v2 ...]`
    Upsert { pk: u64, values: Vec<u64> },

    /// `UPDATE FIELD <pk> col=<n> val=<v>`
    UpdateField { pk: u64, col: usize, val: u64 },

    /// `DELETE RANGE col=<n> lo=<lo> hi=<hi>`
    DeleteRange { col: usize, lo: u64, hi: u64 },

    // ── Layer 50: histogram + percentile ─────────────────────────────────────
    /// `HISTOGRAM col=<n> lo=<lo> hi=<hi> buckets=<b>`
    Histogram {
        col: usize,
        lo: u64,
        hi: u64,
        buckets: usize,
    },

    /// `PERCENTILE col=<n> lo=<lo> hi=<hi> p=<0-100>`
    Percentile {
        col: usize,
        lo: u64,
        hi: u64,
        p: u64,
    },

    // ── Layer 51: group-by aggregates ─────────────────────────────────────────
    /// `GROUP COUNT filter_col=<n> lo=<lo> hi=<hi> group_col=<g>`
    GroupCount {
        filter_col: usize,
        lo: u64,
        hi: u64,
        group_col: usize,
    },

    /// `GROUP SUM filter_col=<n> lo=<lo> hi=<hi> group_col=<g> agg_col=<a>`
    GroupSum {
        filter_col: usize,
        lo: u64,
        hi: u64,
        group_col: usize,
        agg_col: usize,
    },

    /// `GROUP MIN filter_col=<n> lo=<lo> hi=<hi> group_col=<g> agg_col=<a>`
    GroupMin {
        filter_col: usize,
        lo: u64,
        hi: u64,
        group_col: usize,
        agg_col: usize,
    },

    /// `GROUP MAX filter_col=<n> lo=<lo> hi=<hi> group_col=<g> agg_col=<a>`
    GroupMax {
        filter_col: usize,
        lo: u64,
        hi: u64,
        group_col: usize,
        agg_col: usize,
    },

    // ── Layer 52: window functions ────────────────────────────────────────────
    /// `WINDOW RUNNING_SUM filter_col=<n> lo=<lo> hi=<hi> agg_col=<a>`
    WindowRunningSum {
        filter_col: usize,
        lo: u64,
        hi: u64,
        agg_col: usize,
    },

    /// `WINDOW RUNNING_COUNT filter_col=<n> lo=<lo> hi=<hi>`
    WindowRunningCount { filter_col: usize, lo: u64, hi: u64 },

    /// `WINDOW RANK filter_col=<n> lo=<lo> hi=<hi> sort_col=<s> asc=<0|1>`
    WindowRank {
        filter_col: usize,
        lo: u64,
        hi: u64,
        sort_col: usize,
        asc: bool,
    },

    /// `WINDOW DENSE_RANK filter_col=<n> lo=<lo> hi=<hi> sort_col=<s> asc=<0|1>`
    WindowDenseRank {
        filter_col: usize,
        lo: u64,
        hi: u64,
        sort_col: usize,
        asc: bool,
    },
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

            Keyword::Scan => {
                if matches!(self.peek(), Some(Token::Keyword(Keyword::Sorted))) {
                    self.next(); // consume SORTED
                    let (col, lo, hi) = self.parse_col_lo_hi()?;
                    self.expect_keyword(Keyword::Sort)?;
                    self.expect_eq()?;
                    let sort = self.expect_int()? as usize;
                    self.expect_keyword(Keyword::Asc)?;
                    self.expect_eq()?;
                    let asc = self.expect_int()? != 0;
                    self.expect_keyword(Keyword::Limit)?;
                    self.expect_eq()?;
                    let limit = self.expect_int()? as usize;
                    Ok(Query::ScanSorted {
                        col,
                        lo,
                        hi,
                        sort,
                        asc,
                        limit,
                    })
                } else if matches!(self.peek(), Some(Token::Keyword(Keyword::Page))) {
                    self.next(); // consume PAGE
                    let (col, lo, hi) = self.parse_col_lo_hi()?;
                    self.expect_keyword(Keyword::After)?;
                    self.expect_eq()?;
                    let after = self.expect_int()?;
                    self.expect_keyword(Keyword::Limit)?;
                    self.expect_eq()?;
                    let limit = self.expect_int()? as usize;
                    Ok(Query::ScanPage {
                        col,
                        lo,
                        hi,
                        after,
                        limit,
                    })
                } else {
                    let (col, lo, hi) = self.parse_col_lo_hi()?;
                    Ok(Query::Scan { col, lo, hi })
                }
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

            Keyword::Count => {
                let (col, lo, hi) = self.parse_col_lo_hi()?;
                Ok(Query::Count { col, lo, hi })
            }
            Keyword::Sum => {
                let (col, lo, hi) = self.parse_col_lo_hi()?;
                Ok(Query::Sum { col, lo, hi })
            }
            Keyword::Min => {
                let (col, lo, hi) = self.parse_col_lo_hi()?;
                Ok(Query::Min { col, lo, hi })
            }
            Keyword::Max => {
                let (col, lo, hi) = self.parse_col_lo_hi()?;
                Ok(Query::Max { col, lo, hi })
            }
            Keyword::Avg => {
                let (col, lo, hi) = self.parse_col_lo_hi()?;
                Ok(Query::Avg { col, lo, hi })
            }

            Keyword::Upsert => {
                let pk = self.expect_int()?;
                let values = self.collect_ints();
                Ok(Query::Upsert { pk, values })
            }

            Keyword::Update => {
                self.expect_keyword(Keyword::Field)?;
                let pk = self.expect_int()?;
                self.expect_keyword(Keyword::Col)?;
                self.expect_eq()?;
                let col = self.expect_int()? as usize;
                self.expect_keyword(Keyword::Val)?;
                self.expect_eq()?;
                let val = self.expect_int()?;
                Ok(Query::UpdateField { pk, col, val })
            }

            Keyword::Delete => {
                if matches!(self.peek(), Some(Token::Keyword(Keyword::Range))) {
                    self.next(); // consume RANGE
                    let (col, lo, hi) = self.parse_col_lo_hi()?;
                    Ok(Query::DeleteRange { col, lo, hi })
                } else if matches!(self.peek(), Some(Token::Keyword(Keyword::Txn))) {
                    self.next(); // consume TXN
                    let txn_id = self.expect_int()?;
                    let pk = self.expect_int()?;
                    Ok(Query::DeleteTxn { txn_id, pk })
                } else {
                    let pk = self.expect_int()?;
                    Ok(Query::Delete { pk })
                }
            }

            Keyword::Histogram => {
                let (col, lo, hi) = self.parse_col_lo_hi()?;
                self.expect_keyword(Keyword::Buckets)?;
                self.expect_eq()?;
                let buckets = self.expect_int()? as usize;
                Ok(Query::Histogram {
                    col,
                    lo,
                    hi,
                    buckets,
                })
            }

            Keyword::Percentile => {
                let (col, lo, hi) = self.parse_col_lo_hi()?;
                self.expect_keyword(Keyword::P)?;
                self.expect_eq()?;
                let p = self.expect_int()?;
                Ok(Query::Percentile { col, lo, hi, p })
            }

            // ── Layer 51: group-by aggregates ─────────────────────────────────
            Keyword::Group => {
                let agg_kw = match self.next() {
                    Some(Token::Keyword(k)) => k,
                    Some(t) => {
                        return Err(QueryError::Syntax(format!(
                            "expected COUNT/SUM/MIN/MAX after GROUP, got {t:?}"
                        )))
                    }
                    None => return Err(QueryError::UnexpectedEof),
                };
                let (filter_col, lo, hi) = self.parse_filter_col_lo_hi()?;
                self.expect_keyword(Keyword::GroupCol)?;
                self.expect_eq()?;
                let group_col = self.expect_int()? as usize;
                match agg_kw {
                    Keyword::Count => Ok(Query::GroupCount {
                        filter_col,
                        lo,
                        hi,
                        group_col,
                    }),
                    Keyword::Sum => {
                        self.expect_keyword(Keyword::AggCol)?;
                        self.expect_eq()?;
                        let agg_col = self.expect_int()? as usize;
                        Ok(Query::GroupSum {
                            filter_col,
                            lo,
                            hi,
                            group_col,
                            agg_col,
                        })
                    }
                    Keyword::Min => {
                        self.expect_keyword(Keyword::AggCol)?;
                        self.expect_eq()?;
                        let agg_col = self.expect_int()? as usize;
                        Ok(Query::GroupMin {
                            filter_col,
                            lo,
                            hi,
                            group_col,
                            agg_col,
                        })
                    }
                    Keyword::Max => {
                        self.expect_keyword(Keyword::AggCol)?;
                        self.expect_eq()?;
                        let agg_col = self.expect_int()? as usize;
                        Ok(Query::GroupMax {
                            filter_col,
                            lo,
                            hi,
                            group_col,
                            agg_col,
                        })
                    }
                    other => Err(QueryError::Syntax(format!(
                        "expected COUNT/SUM/MIN/MAX after GROUP, got {other:?}"
                    ))),
                }
            }

            // ── Layer 52: window functions ─────────────────────────────────
            Keyword::Window => {
                let fn_kw = match self.next() {
                    Some(Token::Keyword(k)) => k,
                    Some(t) => {
                        return Err(QueryError::Syntax(format!(
                            "expected RUNNING_SUM/RUNNING_COUNT/RANK/DENSE_RANK after WINDOW, got {t:?}"
                        )))
                    }
                    None => return Err(QueryError::UnexpectedEof),
                };
                let (filter_col, lo, hi) = self.parse_filter_col_lo_hi()?;
                match fn_kw {
                    Keyword::RunningSUM => {
                        self.expect_keyword(Keyword::AggCol)?;
                        self.expect_eq()?;
                        let agg_col = self.expect_int()? as usize;
                        Ok(Query::WindowRunningSum {
                            filter_col,
                            lo,
                            hi,
                            agg_col,
                        })
                    }
                    Keyword::RunningCount => Ok(Query::WindowRunningCount {
                        filter_col,
                        lo,
                        hi,
                    }),
                    Keyword::Rank => {
                        self.expect_keyword(Keyword::SortCol)?;
                        self.expect_eq()?;
                        let sort_col = self.expect_int()? as usize;
                        self.expect_keyword(Keyword::Asc)?;
                        self.expect_eq()?;
                        let asc = self.expect_int()? != 0;
                        Ok(Query::WindowRank {
                            filter_col,
                            lo,
                            hi,
                            sort_col,
                            asc,
                        })
                    }
                    Keyword::DenseRank => {
                        self.expect_keyword(Keyword::SortCol)?;
                        self.expect_eq()?;
                        let sort_col = self.expect_int()? as usize;
                        self.expect_keyword(Keyword::Asc)?;
                        self.expect_eq()?;
                        let asc = self.expect_int()? != 0;
                        Ok(Query::WindowDenseRank {
                            filter_col,
                            lo,
                            hi,
                            sort_col,
                            asc,
                        })
                    }
                    other => Err(QueryError::Syntax(format!(
                        "expected RUNNING_SUM/RUNNING_COUNT/RANK/DENSE_RANK after WINDOW, got {other:?}"
                    ))),
                }
            }

            other => Err(QueryError::Syntax(format!(
                "{other:?} is not a statement keyword"
            ))),
        }
    }

    /// Parse `col=<n> lo=<lo> hi=<hi>` — shared by COUNT/SUM/MIN/MAX/AVG/SCAN.
    fn parse_col_lo_hi(&mut self) -> QueryResult<(usize, u64, u64)> {
        self.expect_keyword(Keyword::Col)?;
        self.expect_eq()?;
        let col = self.expect_int()? as usize;
        self.expect_keyword(Keyword::Lo)?;
        self.expect_eq()?;
        let lo = self.expect_int()?;
        self.expect_keyword(Keyword::Hi)?;
        self.expect_eq()?;
        let hi = self.expect_int()?;
        Ok((col, lo, hi))
    }

    /// Parse `filter_col=<n> lo=<lo> hi=<hi>` — used by GROUP-BY aggregates.
    fn parse_filter_col_lo_hi(&mut self) -> QueryResult<(usize, u64, u64)> {
        self.expect_keyword(Keyword::FilterCol)?;
        self.expect_eq()?;
        let col = self.expect_int()? as usize;
        self.expect_keyword(Keyword::Lo)?;
        self.expect_eq()?;
        let lo = self.expect_int()?;
        self.expect_keyword(Keyword::Hi)?;
        self.expect_eq()?;
        let hi = self.expect_int()?;
        Ok((col, lo, hi))
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
