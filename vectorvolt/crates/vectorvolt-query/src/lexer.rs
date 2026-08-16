/// Tokens produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(Keyword),
    Int(u64),
    Ident(String),
    Eq,
}

/// Reserved keywords (case-insensitive).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Get,
    Insert,
    Delete,
    Scan,
    Secondary,
    Begin,
    Commit,
    Rollback,
    Txn,
    Describe,
    Stats,
    Col,
    Lo,
    Hi,
    Val,
    // Layer 43 additions
    Count,
    Sum,
    Min,
    Max,
    Avg,
    Sorted,
    Page,
    Sort,
    Asc,
    After,
    Limit,
    Upsert,
    Update,
    Field,
    Range,
    // Layer 50 additions
    Histogram,
    Percentile,
    Buckets,
    P,
}

impl Keyword {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_uppercase().as_str() {
            "GET" => Some(Self::Get),
            "INSERT" => Some(Self::Insert),
            "DELETE" => Some(Self::Delete),
            "SCAN" => Some(Self::Scan),
            "SECONDARY" => Some(Self::Secondary),
            "BEGIN" => Some(Self::Begin),
            "COMMIT" => Some(Self::Commit),
            "ROLLBACK" => Some(Self::Rollback),
            "TXN" => Some(Self::Txn),
            "DESCRIBE" => Some(Self::Describe),
            "STATS" => Some(Self::Stats),
            "COL" => Some(Self::Col),
            "LO" => Some(Self::Lo),
            "HI" => Some(Self::Hi),
            "VAL" => Some(Self::Val),
            "COUNT" => Some(Self::Count),
            "SUM" => Some(Self::Sum),
            "MIN" => Some(Self::Min),
            "MAX" => Some(Self::Max),
            "AVG" => Some(Self::Avg),
            "SORTED" => Some(Self::Sorted),
            "PAGE" => Some(Self::Page),
            "SORT" => Some(Self::Sort),
            "ASC" => Some(Self::Asc),
            "AFTER" => Some(Self::After),
            "LIMIT" => Some(Self::Limit),
            "UPSERT" => Some(Self::Upsert),
            "UPDATE" => Some(Self::Update),
            "FIELD" => Some(Self::Field),
            "RANGE" => Some(Self::Range),
            "HISTOGRAM" => Some(Self::Histogram),
            "PERCENTILE" => Some(Self::Percentile),
            "BUCKETS" => Some(Self::Buckets),
            "P" => Some(Self::P),
            _ => None,
        }
    }
}

/// Tokenize `input` into a `Vec<Token>`.  Whitespace is skipped; `=` is a
/// dedicated token; everything else is either an integer literal or an
/// identifier / keyword.
pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        if ch == '=' {
            chars.next();
            tokens.push(Token::Eq);
            continue;
        }
        if ch.is_ascii_digit() {
            let mut num = String::new();
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    num.push(d);
                    chars.next();
                } else {
                    break;
                }
            }
            if let Ok(n) = num.parse::<u64>() {
                tokens.push(Token::Int(n));
            }
            continue;
        }
        if ch.is_ascii_alphanumeric() || ch == '_' {
            let mut word = String::new();
            while let Some(&w) = chars.peek() {
                if w.is_ascii_alphanumeric() || w == '_' {
                    word.push(w);
                    chars.next();
                } else {
                    break;
                }
            }
            if let Some(kw) = Keyword::from_str(&word) {
                tokens.push(Token::Keyword(kw));
            } else {
                tokens.push(Token::Ident(word));
            }
            continue;
        }
        // skip unrecognised chars (e.g. punctuation)
        chars.next();
    }

    tokens
}
