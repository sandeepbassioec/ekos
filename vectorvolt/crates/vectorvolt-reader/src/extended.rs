//! Extended query protocol support (pure logic, no I/O): prepared
//! statements, portals and `$n` parameter binding.
//!
//! The SQL executor has no native bind parameters, so a Bind is resolved by
//! substituting each `$n` with a literal: SQL `NULL`, a validated numeric /
//! boolean literal for parameters declared with a numeric / boolean type,
//! and a single-quoted string otherwise. The executor's lexer has no escape
//! for a quote inside a string literal, so a text value containing `'` (or
//! NUL) is refused with SQLSTATE 22P02 instead of being spliced in — a bound
//! value can never change the statement's structure.

use std::collections::VecDeque;

use vectorvolt_query::sql::executor::QueryResult;

/// Type OIDs the binder understands.
pub mod oid {
    pub const BOOL: u32 = 16;
    pub const BYTEA: u32 = 17;
    pub const NAME: u32 = 19;
    pub const INT8: u32 = 20;
    pub const INT2: u32 = 21;
    pub const INT4: u32 = 23;
    pub const TEXT: u32 = 25;
    pub const OID: u32 = 26;
    pub const FLOAT4: u32 = 700;
    pub const FLOAT8: u32 = 701;
    pub const UNKNOWN: u32 = 705;
    pub const BPCHAR: u32 = 1042;
    pub const VARCHAR: u32 = 1043;
    pub const NUMERIC: u32 = 1700;
}

/// A protocol-level error: SQLSTATE + message, sent as an ErrorResponse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtoError {
    pub code: &'static str,
    pub message: String,
}

impl ProtoError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

/// A parsed (prepared) statement.
#[derive(Debug, Clone)]
pub struct Prepared {
    pub query: String,
    /// One OID per parameter; unspecified (0) ones are reported as `text`.
    pub param_types: Vec<u32>,
}

impl Prepared {
    /// Build from a Parse message. The parameter count is the larger of the
    /// declared types and the highest `$n` in the query.
    pub fn new(query: String, declared: &[u32]) -> Result<Self, ProtoError> {
        let n = max_placeholder(&query).max(declared.len());
        if n > i16::MAX as usize {
            return Err(ProtoError::new("54000", "too many parameters"));
        }
        let param_types = (0..n)
            .map(|i| match declared.get(i).copied().unwrap_or(0) {
                0 => oid::TEXT,
                t => t,
            })
            .collect();
        Ok(Self { query, param_types })
    }
}

/// Where a portal's execution stands.
#[derive(Debug)]
pub enum PortalState {
    /// Not executed yet.
    Pending,
    /// Executed: a row set being drained by (possibly limited) Executes.
    Rows {
        columns: Vec<String>,
        rows: VecDeque<Vec<String>>,
        sent: usize,
    },
    /// Executed: a non-row result not yet reported.
    Executed(QueryResult),
    /// Fully reported; a further Execute repeats this command tag.
    Done(String),
}

/// A bound statement ready to execute.
#[derive(Debug)]
pub struct Portal {
    /// SQL with every `$n` substituted.
    pub sql: String,
    pub result_formats: Vec<i16>,
    pub state: PortalState,
}

/// Is `b` part of an identifier (so `a$1` is not a placeholder)?
fn ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b >= 0x80
}

/// Walk `sql`, calling `on_param(start, end, n)` for each `$n` placeholder
/// outside string literals, quoted identifiers and comments.
fn scan_placeholders(sql: &str, mut on_param: impl FnMut(usize, usize, usize)) {
    let b = sql.as_bytes();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            q @ (b'\'' | b'"') => {
                i += 1;
                while i < b.len() && b[i] != q {
                    i += 1;
                }
                i += 1;
            }
            b'-' if b.get(i + 1) == Some(&b'-') => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                i += 2;
                while i < b.len() && !(b[i] == b'*' && b.get(i + 1) == Some(&b'/')) {
                    i += 1;
                }
                i += 2;
            }
            b'$' if (i == 0 || !ident_char(b[i - 1]))
                && b.get(i + 1).is_some_and(|c| c.is_ascii_digit()) =>
            {
                let start = i;
                i += 1;
                let mut n: usize = 0;
                while i < b.len() && b[i].is_ascii_digit() {
                    n = n
                        .saturating_mul(10)
                        .saturating_add(usize::from(b[i] - b'0'));
                    i += 1;
                }
                on_param(start, i, n);
            }
            _ => i += 1,
        }
    }
}

/// The highest `$n` referenced by `sql` (0 = none).
pub fn max_placeholder(sql: &str) -> usize {
    let mut max = 0;
    scan_placeholders(sql, |_, _, n| max = max.max(n));
    max
}

/// Replace every `$n` in `sql` with `literals[n - 1]`.
pub fn substitute(sql: &str, literals: &[String]) -> Result<String, ProtoError> {
    let mut out = String::with_capacity(sql.len());
    let mut last = 0;
    let mut err = None;
    scan_placeholders(sql, |start, end, n| {
        if err.is_some() {
            return;
        }
        match n.checked_sub(1).and_then(|i| literals.get(i)) {
            Some(lit) => {
                out.push_str(&sql[last..start]);
                out.push_str(lit);
                last = end;
            }
            None => {
                err = Some(ProtoError::new(
                    "42P02",
                    format!("there is no parameter ${n}"),
                ))
            }
        }
    });
    if let Some(e) = err {
        return Err(e);
    }
    out.push_str(&sql[last..]);
    Ok(out)
}

/// The format code that applies to item `i` of `n` (Bind convention).
pub fn format_for(formats: &[i16], i: usize) -> i16 {
    match formats {
        [] => 0,
        [f] => *f,
        fs => fs.get(i).copied().unwrap_or(0),
    }
}

/// A numeric literal safe to splice unquoted.
fn numeric_literal(s: &str) -> Option<String> {
    let t = s.trim();
    let ok = !t.is_empty()
        && t.bytes()
            .all(|c| c.is_ascii_digit() || matches!(c, b'+' | b'-' | b'.' | b'e' | b'E'))
        && t.parse::<f64>().is_ok_and(f64::is_finite);
    ok.then(|| t.to_owned())
}

fn bool_literal(s: &str) -> Option<&'static str> {
    match s.trim().to_ascii_lowercase().as_str() {
        "t" | "true" | "y" | "yes" | "on" | "1" => Some("TRUE"),
        "f" | "false" | "n" | "no" | "off" | "0" => Some("FALSE"),
        _ => None,
    }
}

fn quoted_literal(s: &str, idx: usize) -> Result<String, ProtoError> {
    if s.contains('\'') || s.contains('\0') {
        return Err(ProtoError::new(
            "22P02",
            format!(
                "parameter ${idx}: values containing a single quote or NUL cannot be bound \
                 by this server"
            ),
        ));
    }
    Ok(format!("'{s}'"))
}

fn is_numeric_type(t: u32) -> bool {
    matches!(
        t,
        oid::INT2 | oid::INT4 | oid::INT8 | oid::OID | oid::FLOAT4 | oid::FLOAT8 | oid::NUMERIC
    )
}

/// Render one bound parameter (1-based `idx`) as a SQL literal.
pub fn param_literal(
    idx: usize,
    type_oid: u32,
    format: i16,
    value: Option<&[u8]>,
) -> Result<String, ProtoError> {
    let Some(raw) = value else {
        return Ok("NULL".to_owned());
    };
    let bad = |what: &str| {
        ProtoError::new(
            "22P03",
            format!("parameter ${idx}: invalid binary {what} value"),
        )
    };
    let text: String = match format {
        0 => std::str::from_utf8(raw)
            .map_err(|_| ProtoError::new("22021", format!("parameter ${idx} is not valid UTF-8")))?
            .to_owned(),
        1 => match type_oid {
            oid::BOOL => match raw {
                [0] => "false".into(),
                [1] => "true".into(),
                _ => return Err(bad("bool")),
            },
            oid::INT2 => i16::from_be_bytes(raw.try_into().map_err(|_| bad("int2"))?).to_string(),
            oid::INT4 => i32::from_be_bytes(raw.try_into().map_err(|_| bad("int4"))?).to_string(),
            oid::INT8 => i64::from_be_bytes(raw.try_into().map_err(|_| bad("int8"))?).to_string(),
            oid::OID => u32::from_be_bytes(raw.try_into().map_err(|_| bad("oid"))?).to_string(),
            oid::FLOAT4 => {
                f32::from_be_bytes(raw.try_into().map_err(|_| bad("float4"))?).to_string()
            }
            oid::FLOAT8 => {
                f64::from_be_bytes(raw.try_into().map_err(|_| bad("float8"))?).to_string()
            }
            oid::TEXT | oid::VARCHAR | oid::BPCHAR | oid::NAME | oid::UNKNOWN => {
                std::str::from_utf8(raw)
                    .map_err(|_| {
                        ProtoError::new("22021", format!("parameter ${idx} is not valid UTF-8"))
                    })?
                    .to_owned()
            }
            other => {
                return Err(ProtoError::new(
                    "0A000",
                    format!(
                        "parameter ${idx}: binary format is not supported for type OID {other} \
                         (send it in text format)"
                    ),
                ));
            }
        },
        f => {
            return Err(ProtoError::new(
                "08P01",
                format!("parameter ${idx}: unsupported format code {f}"),
            ));
        }
    };
    if is_numeric_type(type_oid) {
        return numeric_literal(&text).ok_or_else(|| {
            ProtoError::new(
                "22P02",
                format!("parameter ${idx}: invalid numeric value \"{text}\""),
            )
        });
    }
    if type_oid == oid::BOOL {
        return bool_literal(&text).map(str::to_owned).ok_or_else(|| {
            ProtoError::new(
                "22P02",
                format!("parameter ${idx}: invalid boolean value \"{text}\""),
            )
        });
    }
    if type_oid == oid::BYTEA {
        return Err(ProtoError::new(
            "0A000",
            format!("parameter ${idx}: bytea parameters are not supported"),
        ));
    }
    quoted_literal(&text, idx)
}

/// Bind `params` to `stmt`, producing the executable SQL.
pub fn bind(
    stmt: &Prepared,
    param_formats: &[i16],
    params: &[Option<Vec<u8>>],
) -> Result<String, ProtoError> {
    let n = stmt.param_types.len();
    if params.len() != n {
        return Err(ProtoError::new(
            "08P01",
            format!(
                "bind message supplies {} parameters, but prepared statement requires {n}",
                params.len()
            ),
        ));
    }
    if param_formats.len() > 1 && param_formats.len() != n {
        return Err(ProtoError::new(
            "08P01",
            format!(
                "bind message has {} parameter formats but {n} parameters",
                param_formats.len()
            ),
        ));
    }
    let literals = params
        .iter()
        .enumerate()
        .map(|(i, v)| {
            param_literal(
                i + 1,
                stmt.param_types[i],
                format_for(param_formats, i),
                v.as_deref(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    substitute(&stmt.query, &literals)
}

/// Validate Bind result-format codes (0 = text, 1 = binary).
pub fn check_result_formats(formats: &[i16]) -> Result<(), ProtoError> {
    match formats.iter().find(|&&f| f != 0 && f != 1) {
        Some(f) => Err(ProtoError::new(
            "08P01",
            format!("unsupported result format code {f}"),
        )),
        None => Ok(()),
    }
}

/// Does `sql` (by its leading keyword) return rows? Used to decide whether a
/// Describe may run the query to learn its columns.
pub fn is_row_returning(sql: &str) -> bool {
    let mut s = sql.trim_start();
    // Skip leading comments and parentheses.
    loop {
        if let Some(r) = s.strip_prefix("--") {
            s = r.split_once('\n').map(|x| x.1).unwrap_or("").trim_start();
        } else if let Some(r) = s.strip_prefix("/*") {
            s = r.split_once("*/").map(|x| x.1).unwrap_or("").trim_start();
        } else if let Some(r) = s.strip_prefix('(') {
            s = r.trim_start();
        } else {
            break;
        }
    }
    let word: String = s
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_ascii_uppercase();
    matches!(
        word.as_str(),
        "SELECT" | "WITH" | "SHOW" | "VALUES" | "TABLE"
    )
}

/// Strip `WithNotice` wrappers, collecting the notices.
pub fn unwrap_notices(mut r: QueryResult, notices: &mut Vec<String>) -> QueryResult {
    while let QueryResult::WithNotice { result, notice } = r {
        notices.push(notice);
        r = *result;
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholders_outside_literals_only() {
        assert_eq!(max_placeholder("SELECT $1, $12 FROM t"), 12);
        assert_eq!(max_placeholder("SELECT '$5', \"$6\" -- $7\n /* $8 */"), 0);
        assert_eq!(max_placeholder("SELECT a$1 FROM t"), 0);
        assert_eq!(
            substitute("SELECT $1, '$1', $2", &["7".into(), "'x'".into()]).unwrap(),
            "SELECT 7, '$1', 'x'"
        );
        assert_eq!(
            substitute("SELECT $3", &["1".into()]).unwrap_err().code,
            "42P02"
        );
    }

    #[test]
    fn literals() {
        assert_eq!(
            param_literal(1, oid::TEXT, 0, Some(b"abc")).unwrap(),
            "'abc'"
        );
        assert_eq!(param_literal(1, oid::TEXT, 0, None).unwrap(), "NULL");
        assert_eq!(param_literal(1, oid::INT4, 0, Some(b"42")).unwrap(), "42");
        assert!(param_literal(1, oid::INT4, 0, Some(b"42; DROP")).is_err());
        assert_eq!(
            param_literal(1, oid::INT4, 1, Some(&7i32.to_be_bytes())).unwrap(),
            "7"
        );
        assert_eq!(
            param_literal(1, oid::INT8, 1, Some(&(-9i64).to_be_bytes())).unwrap(),
            "-9"
        );
        assert_eq!(param_literal(1, oid::BOOL, 1, Some(&[1])).unwrap(), "TRUE");
        assert_eq!(param_literal(1, oid::TEXT, 1, Some(b"hi")).unwrap(), "'hi'");
        assert_eq!(
            param_literal(1, oid::TEXT, 0, Some(b"x' OR '1'='1"))
                .unwrap_err()
                .code,
            "22P02"
        );
        assert_eq!(
            param_literal(1, 1114, 1, Some(&[0; 8])).unwrap_err().code,
            "0A000"
        );
        assert_eq!(
            param_literal(1, oid::TEXT, 2, Some(b"x")).unwrap_err().code,
            "08P01"
        );
    }

    #[test]
    fn bind_checks_counts() {
        let p = Prepared::new(
            "SELECT * FROM t WHERE a = $1 AND b = $2".into(),
            &[oid::INT4],
        )
        .unwrap();
        assert_eq!(p.param_types, vec![oid::INT4, oid::TEXT]);
        assert_eq!(
            bind(&p, &[], &[Some(b"1".to_vec()), Some(b"x".to_vec())]).unwrap(),
            "SELECT * FROM t WHERE a = 1 AND b = 'x'"
        );
        assert_eq!(bind(&p, &[], &[None]).unwrap_err().code, "08P01");
        assert_eq!(
            bind(&p, &[0, 0, 0], &[None, None]).unwrap_err().code,
            "08P01"
        );
    }

    #[test]
    fn row_returning() {
        assert!(is_row_returning("  select 1"));
        assert!(is_row_returning("-- c\n(SELECT 1)"));
        assert!(is_row_returning("WITH x AS (SELECT 1) SELECT * FROM x"));
        assert!(!is_row_returning("INSERT INTO t VALUES (1)"));
        assert!(!is_row_returning(""));
    }
}
