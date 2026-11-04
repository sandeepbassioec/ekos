//! E1 — `SET` / `SHOW` of session settings: real settings fall through to
//! their handlers, PostgreSQL client-compat settings are accepted as no-ops,
//! and every other `SET` fails with SQLSTATE 0A000.

use super::*;

/// E1 — standard PostgreSQL client settings that drivers send on connect.
/// `SET` accepts them as documented no-ops (the server does not change
/// behaviour), and `SHOW` echoes the last value set. Names are lower-case.
pub(super) const COMPAT_SETTINGS: &[(&str, &str)] = &[
    ("application_name", ""),
    ("bytea_output", "hex"),
    ("client_encoding", "UTF8"),
    ("client_min_messages", "notice"),
    ("datestyle", "ISO, MDY"),
    ("extra_float_digits", "1"),
    ("intervalstyle", "postgres"),
    ("lc_messages", "C"),
    ("lc_monetary", "C"),
    ("lc_numeric", "C"),
    ("lc_time", "C"),
    ("standard_conforming_strings", "on"),
    ("timezone", "UTC"),
    ("xmloption", "content"),
];

/// E1 — `SET` forms with a real effect on the session or executor. Every
/// other `SET` is rejected with SQLSTATE 0A000. Entries ending in a space or
/// quote match as plain prefixes; the rest must end at a word boundary.
const REAL_SET_PREFIXES: &[&str] = &[
    "SET SESSION TOKEN",
    "SET RATE LIMIT",
    "SET WARNINGS",
    "SET READ ONLY",
    "SET READ WRITE",
    "SET ARCHIVE PATH",
    "SET SLOW_QUERY_THRESHOLD",
    "SET AUTO_EXPLAIN_THRESHOLD",
    "SET STATEMENT_TIMEOUT",
    "SET SEARCH_PATH",
    "SET MAX_CONNECTIONS",
    "SET STREAM CHUNK SIZE",
    "SET ROLE",
    "SET RESULT MASK ON",
    "SET CONNECTION IDLE TIMEOUT",
    "SET QUERY MEMORY LIMIT",
    "SET SESSION VAR '",
    "SET SESSION VAR2 ",
    "SET GLOBAL VAR ",
    "SET CONFIG KEY '",
];

/// True when `up` starts with `prefix` ending on a word boundary.
fn starts_with_word(up: &str, prefix: &str) -> bool {
    match up.strip_prefix(prefix) {
        None => false,
        Some(rest) => {
            prefix.ends_with(['\'', ' '])
                || !rest.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_')
        }
    }
}

/// Strip surrounding single quotes from a `SET` value.
fn unquote_setting(v: &str) -> String {
    let v = v.trim();
    v.strip_prefix('\'')
        .and_then(|x| x.strip_suffix('\''))
        .unwrap_or(v)
        .to_owned()
}

impl SqlExecutor {
    /// E1 — front door for `SET` / `SHOW` of session settings. Returns
    /// `None` to let the normal pipeline handle the statement (real
    /// settings, every non-settings statement).
    ///
    /// * real settings (`REAL_SET_PREFIXES`) fall through;
    /// * PostgreSQL client-compat settings are accepted as no-ops;
    /// * any other `SET` fails with SQLSTATE 0A000 naming the setting.
    pub(super) fn settings_gate(&self, sql: &str) -> Option<Result<QueryResult, ExecError>> {
        let head = sql.trim_start();
        let kw = |n: usize, w: &str| {
            head.is_char_boundary(n) && head.len() > n && head[..n].eq_ignore_ascii_case(w)
        };
        let is_set = kw(4, "SET ");
        let is_show = kw(5, "SHOW ");
        if !is_set && !is_show {
            return None;
        }
        let s = sql.trim().trim_end_matches(';').trim();
        if is_show {
            let name = s[5..].trim().to_ascii_lowercase();
            let name = if name == "time zone" {
                "timezone".to_owned()
            } else {
                name
            };
            let compat = self.compat_settings.lock_safe().get(&name).cloned();
            let value = if let Some(v) = compat {
                v
            } else if name == "search_path" {
                self.search_path.lock_safe().clone()
            } else if name == "statement_timeout" {
                self.statement_timeout_ms
                    .lock_safe()
                    .map_or_else(|| "0".to_owned(), |ms| format!("{ms}ms"))
            } else {
                return None;
            };
            return Some(Ok(QueryResult::Rows {
                columns: vec![name],
                rows: vec![vec![value]],
            }));
        }
        let up = s.to_ascii_uppercase();
        if REAL_SET_PREFIXES.iter().any(|p| starts_with_word(&up, p)) {
            return None;
        }
        // `SET [SESSION | LOCAL] name {= | TO} value`, plus the PostgreSQL
        // spellings `SET TIME ZONE v` and `SET NAMES v`.
        let mut body = s[4..].trim_start();
        for kw in ["SESSION ", "LOCAL "] {
            if body.len() > kw.len()
                && body.is_char_boundary(kw.len())
                && body[..kw.len()].eq_ignore_ascii_case(kw)
            {
                body = body[kw.len()..].trim_start();
            }
        }
        let up_body = body.to_ascii_uppercase();
        let (name, value) = if up_body.starts_with("TIME ZONE ") {
            ("timezone".to_owned(), body["TIME ZONE ".len()..].to_owned())
        } else if up_body.starts_with("NAMES ") {
            (
                "client_encoding".to_owned(),
                body["NAMES ".len()..].to_owned(),
            )
        } else {
            let end = body
                .find(|c: char| c.is_whitespace() || c == '=')
                .unwrap_or(body.len());
            let name = body[..end].to_ascii_lowercase();
            let rest = body[end..].trim_start();
            let rest = if let Some(r) = rest.strip_prefix('=') {
                r
            } else if rest.len() > 3
                && rest.is_char_boundary(3)
                && rest[..3].eq_ignore_ascii_case("TO ")
            {
                &rest[3..]
            } else {
                rest
            };
            (name, rest.to_owned())
        };
        if COMPAT_SETTINGS.iter().any(|(k, _)| *k == name) && !value.trim().is_empty() {
            let v = unquote_setting(&value);
            self.compat_settings.lock_safe().insert(name, v);
            return Some(Ok(QueryResult::Ok("SET".to_owned())));
        }
        // Name the setting as the user wrote it: leading words up to the
        // first value-looking token.
        let shown: Vec<&str> = body
            .split_whitespace()
            .take_while(|w| {
                let u = w.to_ascii_uppercase();
                !(w.starts_with('\'')
                    || w.starts_with(|c: char| c.is_ascii_digit())
                    || w.contains('=')
                    || matches!(u.as_str(), "TO" | "FOR" | "ON" | "OFF"))
            })
            .take(4)
            .collect();
        let shown = if shown.is_empty() {
            name.clone()
        } else {
            shown.join(" ")
        };
        Some(Ok(sql_error(
            "0A000",
            &format!(
                "unsupported setting \"{shown}\": SET {shown} is not implemented and would have no effect"
            ),
        )))
    }
}
