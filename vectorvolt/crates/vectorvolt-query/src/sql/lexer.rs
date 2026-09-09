use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Select,
    From,
    Where,
    Insert,
    Into,
    Values,
    Delete,
    Create,
    Table,
    Drop,
    Order,
    By,
    Asc,
    Desc,
    Limit,
    Group,
    Having,
    And,
    Or,
    Not,
    As,
    Distinct,
    Explain,
    Begin,
    Commit,
    Rollback,
    // Types
    KwU64,
    KwI64,
    KwF64,
    KwBool,
    KwText,
    KwTimestamp,
    // Literals
    Integer(i64),
    Float(f64),
    Str(String),
    Boolean(bool),
    // Identifiers
    Ident(String),
    // Operators
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    Plus,
    Minus,
    Star,
    Slash,
    // Punctuation
    LParen,
    RParen,
    Comma,
    Semicolon,
    Dot,
    Eof,
}

#[derive(Debug, Error)]
pub enum LexError {
    #[error("unexpected character '{0}' at position {1}")]
    UnexpectedChar(char, usize),
}

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        let bytes = self.input.as_bytes();
        while self.pos < bytes.len() {
            let ch = bytes[self.pos] as char;
            if ch.is_whitespace() {
                self.pos += 1;
                continue;
            }
            if ch == '\'' {
                self.pos += 1;
                let start = self.pos;
                while self.pos < bytes.len() && bytes[self.pos] as char != '\'' {
                    self.pos += 1;
                }
                let s = self.input[start..self.pos].to_owned();
                if self.pos < bytes.len() {
                    self.pos += 1;
                }
                tokens.push(Token::Str(s));
                continue;
            }
            if ch == '-'
                && self.pos + 1 < bytes.len()
                && (bytes[self.pos + 1] as char).is_ascii_digit()
            {
                self.pos += 1;
                let (tok, adv) = Self::lex_number(self.input, self.pos, true)?;
                self.pos += adv;
                tokens.push(tok);
                continue;
            }
            if ch.is_ascii_digit() {
                let (tok, adv) = Self::lex_number(self.input, self.pos, false)?;
                self.pos += adv;
                tokens.push(tok);
                continue;
            }
            if ch.is_ascii_alphabetic() || ch == '_' {
                let start = self.pos;
                while self.pos < bytes.len()
                    && ((bytes[self.pos] as char).is_ascii_alphanumeric()
                        || bytes[self.pos] == b'_')
                {
                    self.pos += 1;
                }
                let word = &self.input[start..self.pos];
                tokens.push(Self::keyword_or_ident(word));
                continue;
            }
            if ch == '<' {
                if self.pos + 1 < bytes.len() && bytes[self.pos + 1] == b'=' {
                    tokens.push(Token::Lte);
                    self.pos += 2;
                    continue;
                }
                if self.pos + 1 < bytes.len() && bytes[self.pos + 1] == b'>' {
                    tokens.push(Token::Neq);
                    self.pos += 2;
                    continue;
                }
                tokens.push(Token::Lt);
                self.pos += 1;
                continue;
            }
            if ch == '>' {
                if self.pos + 1 < bytes.len() && bytes[self.pos + 1] == b'=' {
                    tokens.push(Token::Gte);
                    self.pos += 2;
                    continue;
                }
                tokens.push(Token::Gt);
                self.pos += 1;
                continue;
            }
            if ch == '!' && self.pos + 1 < bytes.len() && bytes[self.pos + 1] == b'=' {
                tokens.push(Token::Neq);
                self.pos += 2;
                continue;
            }
            let tok = match ch {
                '=' => Token::Eq,
                '+' => Token::Plus,
                '-' => Token::Minus,
                '*' => Token::Star,
                '/' => Token::Slash,
                '(' => Token::LParen,
                ')' => Token::RParen,
                ',' => Token::Comma,
                ';' => Token::Semicolon,
                '.' => Token::Dot,
                _ => return Err(LexError::UnexpectedChar(ch, self.pos)),
            };
            tokens.push(tok);
            self.pos += 1;
        }
        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn lex_number(input: &str, pos: usize, negative: bool) -> Result<(Token, usize), LexError> {
        let bytes = input.as_bytes();
        let start = pos;
        let mut end = pos;
        let mut is_float = false;
        while end < bytes.len() && (bytes[end] as char).is_ascii_digit() {
            end += 1;
        }
        if end < bytes.len()
            && bytes[end] == b'.'
            && end + 1 < bytes.len()
            && (bytes[end + 1] as char).is_ascii_digit()
        {
            is_float = true;
            end += 1;
            while end < bytes.len() && (bytes[end] as char).is_ascii_digit() {
                end += 1;
            }
        }
        let s = &input[start..end];
        let adv = end - pos;
        if is_float {
            let v: f64 = s.parse().map_err(|_| LexError::UnexpectedChar('.', pos))?;
            Ok((Token::Float(if negative { -v } else { v }), adv))
        } else {
            let v: i64 = s.parse().map_err(|_| LexError::UnexpectedChar('0', pos))?;
            Ok((Token::Integer(if negative { -v } else { v }), adv))
        }
    }

    fn keyword_or_ident(word: &str) -> Token {
        match word.to_ascii_uppercase().as_str() {
            "SELECT" => Token::Select,
            "FROM" => Token::From,
            "WHERE" => Token::Where,
            "INSERT" => Token::Insert,
            "INTO" => Token::Into,
            "VALUES" => Token::Values,
            "DELETE" => Token::Delete,
            "CREATE" => Token::Create,
            "TABLE" => Token::Table,
            "DROP" => Token::Drop,
            "ORDER" => Token::Order,
            "BY" => Token::By,
            "ASC" => Token::Asc,
            "DESC" => Token::Desc,
            "LIMIT" => Token::Limit,
            "GROUP" => Token::Group,
            "HAVING" => Token::Having,
            "AND" => Token::And,
            "OR" => Token::Or,
            "NOT" => Token::Not,
            "AS" => Token::As,
            "DISTINCT" => Token::Distinct,
            "EXPLAIN" => Token::Explain,
            "BEGIN" => Token::Begin,
            "COMMIT" => Token::Commit,
            "ROLLBACK" | "ABORT" => Token::Rollback,
            "U64" => Token::KwU64,
            "I64" => Token::KwI64,
            "F64" => Token::KwF64,
            "BOOL" => Token::KwBool,
            "TEXT" => Token::KwText,
            "TIMESTAMP" => Token::KwTimestamp,
            "TRUE" => Token::Boolean(true),
            "FALSE" => Token::Boolean(false),
            _ => Token::Ident(word.to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(s: &str) -> Vec<Token> {
        Lexer::new(s).tokenize().unwrap()
    }

    #[test]
    fn test_select_keyword() {
        let t = lex("SELECT");
        assert_eq!(t[0], Token::Select);
    }

    #[test]
    fn test_from_keyword() {
        let t = lex("FROM");
        assert_eq!(t[0], Token::From);
    }

    #[test]
    fn test_where_keyword() {
        let t = lex("WHERE");
        assert_eq!(t[0], Token::Where);
    }

    #[test]
    fn test_integer_literal() {
        let t = lex("42");
        assert_eq!(t[0], Token::Integer(42));
    }

    #[test]
    fn test_float_literal() {
        let t = lex("3.14");
        assert!(matches!(t[0], Token::Float(_)));
    }

    #[test]
    fn test_string_literal() {
        let t = lex("'hello'");
        assert_eq!(t[0], Token::Str("hello".to_owned()));
    }

    #[test]
    fn test_identifier() {
        let t = lex("my_table");
        assert_eq!(t[0], Token::Ident("my_table".to_owned()));
    }

    #[test]
    fn test_operators() {
        let t = lex("= < > <= >= <>");
        assert_eq!(t[0], Token::Eq);
        assert_eq!(t[1], Token::Lt);
        assert_eq!(t[2], Token::Gt);
        assert_eq!(t[3], Token::Lte);
        assert_eq!(t[4], Token::Gte);
        assert_eq!(t[5], Token::Neq);
    }

    #[test]
    fn test_punctuation() {
        let t = lex(", ; ( )");
        assert_eq!(t[0], Token::Comma);
        assert_eq!(t[1], Token::Semicolon);
        assert_eq!(t[2], Token::LParen);
        assert_eq!(t[3], Token::RParen);
    }

    #[test]
    fn test_mixed_select_from() {
        let t = lex("SELECT a FROM t");
        assert_eq!(t[0], Token::Select);
        assert_eq!(t[1], Token::Ident("a".to_owned()));
        assert_eq!(t[2], Token::From);
        assert_eq!(t[3], Token::Ident("t".to_owned()));
    }

    #[test]
    fn test_case_insensitive() {
        let t = lex("select FROM Where");
        assert_eq!(t[0], Token::Select);
        assert_eq!(t[1], Token::From);
        assert_eq!(t[2], Token::Where);
    }

    #[test]
    fn test_unknown_char_error() {
        let r = Lexer::new("@bad").tokenize();
        assert!(r.is_err());
    }

    #[test]
    fn test_empty_input() {
        let t = lex("");
        assert_eq!(t, vec![Token::Eof]);
    }
}
