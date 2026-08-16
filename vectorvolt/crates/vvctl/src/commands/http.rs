//! `vvctl http` — HTTP-gateway client for analytics and write operations.
//!
//! Covers features from Layers 44-60 that are only reachable via the HTTP
//! gateway (not the TCP wire protocol):
//!
//! ```text
//! # Raw DSL — works for everything, including Layers 50-60 ops
//! vvctl http --addr http://localhost:3030 --table t query "HISTOGRAM col=0 lo=0 hi=100 BUCKETS=10"
//!
//! # Dedicated sub-commands
//! vvctl http --addr http://localhost:3030 --table t histogram --col 0 --lo 0 --hi 100 --buckets 10
//! vvctl http --addr http://localhost:3030 --table t percentile --col 0 --p 95
//! vvctl http --addr http://localhost:3030 --table t group-count --key-col 0 --val-col 1
//! vvctl http --addr http://localhost:3030 --table t window-rank --col 1
//! vvctl http --addr http://localhost:3030 --table t distinct --col 0
//! vvctl http --addr http://localhost:3030 --table t value-counts --col 0
//! vvctl http --addr http://localhost:3030 --table t update-where --col 0 --op eq --val 5 --target-col 1 --new-val 99
//! vvctl http --addr http://localhost:3030 --table t increment-where --col 0 --op lt --val 10 --target-col 1 --delta 1
//! vvctl http --addr http://localhost:3030 --table t decrement-where --col 0 --op gt --val 50 --target-col 1 --delta 1
//! vvctl http --addr http://localhost:3030 --table t scan-where --pred col:0,op:eq,val:7 --pred col:1,op:gt,val:0
//! vvctl http --addr http://localhost:3030 --table t scan-project --cols 0,2,3
//! vvctl http --addr http://localhost:3030 --table t if-absent --pk 42 --values 42,100,200
//! vvctl http --addr http://localhost:3030 --table t cas --pk 42 --expected 42,100,200 --new-values 42,101,201
//! vvctl http --addr http://localhost:3030 --table t batch-insert --row 1,10,20 --row 2,30,40
//! vvctl http --addr http://localhost:3030 --table t insert-ttl --pk 9 --values 9,77 --expire-at 9999999999
//! vvctl http --addr http://localhost:3030 --table t get-ttl --pk 9 --now 0
//! vvctl http --addr http://localhost:3030 --table t scan-ttl --now 0
//! vvctl http --addr http://localhost:3030 --table t purge-expired --now 9999999999
//! ```

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

// ── top-level args ────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct HttpArgs {
    /// Base URL of the VectorVolt HTTP gateway (e.g. http://localhost:3030).
    #[arg(long, default_value = "http://localhost:3030")]
    pub addr: String,

    /// Table name to operate on.
    #[arg(long, short = 't')]
    pub table: String,

    /// Bearer token for authenticated gateways (matches --api-token on the server).
    #[arg(long)]
    pub token: Option<String>,

    #[command(subcommand)]
    pub command: HttpCommand,
}

#[derive(Subcommand)]
pub enum HttpCommand {
    /// Send a raw DSL statement to POST /tables/{t}/query.
    Query {
        /// DSL statement string (e.g. "HISTOGRAM col=0 lo=0 hi=100 BUCKETS=10").
        dsl: String,
    },

    /// Compute a frequency histogram over one column.
    Histogram {
        /// Column index.
        #[arg(long)]
        col: u64,
        /// Bucket lower bound (inclusive).
        #[arg(long)]
        lo: u64,
        /// Bucket upper bound (inclusive).
        #[arg(long)]
        hi: u64,
        /// Number of buckets.
        #[arg(long)]
        buckets: u64,
    },

    /// Compute an approximate percentile over one column.
    Percentile {
        /// Column index.
        #[arg(long)]
        col: u64,
        /// Percentile to compute (0-100).
        #[arg(long)]
        p: u64,
    },

    /// Group-by count: count rows per unique key-column value.
    GroupCount {
        /// Key column index.
        #[arg(long)]
        key_col: u64,
    },

    /// Group-by sum: sum a value column per unique key-column value.
    GroupSum {
        #[arg(long)]
        key_col: u64,
        #[arg(long)]
        val_col: u64,
    },

    /// Group-by min: minimum of a value column per unique key-column value.
    GroupMin {
        #[arg(long)]
        key_col: u64,
        #[arg(long)]
        val_col: u64,
    },

    /// Group-by max: maximum of a value column per unique key-column value.
    GroupMax {
        #[arg(long)]
        key_col: u64,
        #[arg(long)]
        val_col: u64,
    },

    /// Window function: running sum over a column.
    WindowRunningSum {
        #[arg(long)]
        col: u64,
    },

    /// Window function: running count (rank-1 based) over a column.
    WindowRunningCount {
        #[arg(long)]
        col: u64,
    },

    /// Window function: rank over a column.
    WindowRank {
        #[arg(long)]
        col: u64,
    },

    /// Window function: dense rank over a column.
    WindowDenseRank {
        #[arg(long)]
        col: u64,
    },

    /// Return all distinct values for a column.
    Distinct {
        #[arg(long)]
        col: u64,
    },

    /// Return value → count map for a column.
    ValueCounts {
        #[arg(long)]
        col: u64,
    },

    /// Update a field on every row matching a predicate.
    UpdateWhere {
        /// Column index to filter on.
        #[arg(long)]
        col: u64,
        /// Comparison operator (eq, ne, lt, le, gt, ge).
        #[arg(long)]
        op: String,
        /// Value to compare against.
        #[arg(long)]
        val: u64,
        /// Column index to update.
        #[arg(long)]
        target_col: u64,
        /// New value to write.
        #[arg(long)]
        new_val: u64,
    },

    /// Increment a field on every row matching a predicate.
    IncrementWhere {
        #[arg(long)]
        col: u64,
        #[arg(long)]
        op: String,
        #[arg(long)]
        val: u64,
        #[arg(long)]
        target_col: u64,
        #[arg(long)]
        delta: u64,
    },

    /// Decrement a field on every row matching a predicate.
    DecrementWhere {
        #[arg(long)]
        col: u64,
        #[arg(long)]
        op: String,
        #[arg(long)]
        val: u64,
        #[arg(long)]
        target_col: u64,
        #[arg(long)]
        delta: u64,
    },

    /// Scan rows matching multiple column predicates.
    /// Each predicate is a comma-separated triple: col:<N>,op:<op>,val:<V>
    /// (op is one of: eq, ne, lt, le, gt, ge).
    ScanWhere {
        /// Predicates (repeatable).  e.g. --pred col:0,op:eq,val:5
        #[arg(long = "pred", num_args = 1)]
        preds: Vec<String>,
        /// Maximum rows to return.
        #[arg(long)]
        limit: Option<u64>,
    },

    /// Scan rows with column projection (return only selected columns).
    ScanProject {
        /// Comma-separated column indices to return (e.g. 0,2,3).
        #[arg(long)]
        cols: String,
        /// Maximum rows to return.
        #[arg(long)]
        limit: Option<u64>,
    },

    /// Insert a row only if the primary key does not already exist.
    IfAbsent {
        #[arg(long)]
        pk: u64,
        /// Comma-separated column values (e.g. 42,100,200).
        #[arg(long)]
        values: String,
    },

    /// Compare-and-swap: atomically replace a row when its current values match.
    Cas {
        #[arg(long)]
        pk: u64,
        /// Comma-separated expected current values.
        #[arg(long)]
        expected: String,
        /// Comma-separated new values to write.
        #[arg(long)]
        new_values: String,
    },

    /// Insert multiple rows in one request.
    /// Each --row is a comma-separated list of column values (first value = pk).
    BatchInsert {
        /// Row values (repeatable).  e.g. --row 1,10,20 --row 2,30,40
        #[arg(long = "row", num_args = 1)]
        rows: Vec<String>,
    },

    /// Insert a row with a TTL expiry timestamp.
    InsertTtl {
        #[arg(long)]
        pk: u64,
        /// Comma-separated column values.
        #[arg(long)]
        values: String,
        /// Unix nanosecond timestamp at which this row expires.
        #[arg(long)]
        expire_at: u64,
    },

    /// Get a row, returning nothing if it has expired.
    GetTtl {
        #[arg(long)]
        pk: u64,
        /// Current timestamp (Unix nanoseconds).
        #[arg(long)]
        now: u64,
    },

    /// Scan all rows that have not yet expired.
    ScanTtl {
        /// Current timestamp (Unix nanoseconds).
        #[arg(long)]
        now: u64,
    },

    /// Remove all rows that have expired.
    PurgeExpired {
        /// Current timestamp (Unix nanoseconds).
        #[arg(long)]
        now: u64,
    },
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn make_client() -> Result<Client> {
    Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("failed to build HTTP client")
}

fn auth_header(token: &Option<String>) -> Option<String> {
    token.as_ref().map(|t| format!("Bearer {t}"))
}

fn table_url(addr: &str, table: &str) -> String {
    format!("{addr}/tables/{table}")
}

fn parse_u64_list(s: &str) -> Result<Vec<u64>> {
    s.split(',')
        .filter(|p| !p.is_empty())
        .map(|p| {
            p.trim()
                .parse::<u64>()
                .with_context(|| format!("not a u64: {p}"))
        })
        .collect()
}

fn print_response(resp: reqwest::blocking::Response) -> Result<()> {
    let status = resp.status();
    let body: serde_json::Value = resp.json().context("response body was not JSON")?;
    let pretty = serde_json::to_string_pretty(&body).unwrap();
    if status.is_success() {
        println!("{pretty}");
    } else {
        bail!("HTTP {status}\n{pretty}");
    }
    Ok(())
}

fn get_req(
    client: &Client,
    url: &str,
    params: &[(&str, String)],
    token: &Option<String>,
) -> Result<reqwest::blocking::Response> {
    let mut req = client.get(url);
    if let Some(auth) = auth_header(token) {
        req = req.header(AUTHORIZATION, auth);
    }
    for (k, v) in params {
        req = req.query(&[(k, v)]);
    }
    req.send().context("HTTP GET failed")
}

fn post_json(
    client: &Client,
    url: &str,
    body: serde_json::Value,
    token: &Option<String>,
) -> Result<reqwest::blocking::Response> {
    let mut req = client.post(url).json(&body);
    if let Some(auth) = auth_header(token) {
        req = req.header(AUTHORIZATION, auth);
    }
    req.send().context("HTTP POST failed")
}

// ── entry point ───────────────────────────────────────────────────────────────

pub fn run(args: HttpArgs) -> Result<()> {
    let client = make_client()?;
    let base = table_url(&args.addr, &args.table);
    let tok = &args.token;

    match args.command {
        HttpCommand::Query { dsl } => {
            let url = format!("{base}/query");
            let resp = client
                .post(&url)
                .header(CONTENT_TYPE, "text/plain")
                .apply_token(tok)
                .body(dsl)
                .send()
                .context("HTTP POST /query failed")?;
            print_response(resp)?;
        }

        HttpCommand::Histogram {
            col,
            lo,
            hi,
            buckets,
        } => {
            let url = format!("{base}/histogram");
            let resp = get_req(
                &client,
                &url,
                &[
                    ("col", col.to_string()),
                    ("lo", lo.to_string()),
                    ("hi", hi.to_string()),
                    ("buckets", buckets.to_string()),
                ],
                tok,
            )?;
            print_response(resp)?;
        }

        HttpCommand::Percentile { col, p } => {
            let url = format!("{base}/percentile");
            let resp = get_req(
                &client,
                &url,
                &[("col", col.to_string()), ("p", p.to_string())],
                tok,
            )?;
            print_response(resp)?;
        }

        HttpCommand::GroupCount { key_col } => {
            let url = format!("{base}/group/count");
            let resp = get_req(&client, &url, &[("key_col", key_col.to_string())], tok)?;
            print_response(resp)?;
        }

        HttpCommand::GroupSum { key_col, val_col } => {
            let url = format!("{base}/group/sum");
            let resp = get_req(
                &client,
                &url,
                &[
                    ("key_col", key_col.to_string()),
                    ("val_col", val_col.to_string()),
                ],
                tok,
            )?;
            print_response(resp)?;
        }

        HttpCommand::GroupMin { key_col, val_col } => {
            let url = format!("{base}/group/min");
            let resp = get_req(
                &client,
                &url,
                &[
                    ("key_col", key_col.to_string()),
                    ("val_col", val_col.to_string()),
                ],
                tok,
            )?;
            print_response(resp)?;
        }

        HttpCommand::GroupMax { key_col, val_col } => {
            let url = format!("{base}/group/max");
            let resp = get_req(
                &client,
                &url,
                &[
                    ("key_col", key_col.to_string()),
                    ("val_col", val_col.to_string()),
                ],
                tok,
            )?;
            print_response(resp)?;
        }

        HttpCommand::WindowRunningSum { col } => {
            let url = format!("{base}/window/running_sum");
            let resp = get_req(&client, &url, &[("col", col.to_string())], tok)?;
            print_response(resp)?;
        }

        HttpCommand::WindowRunningCount { col } => {
            let url = format!("{base}/window/running_count");
            let resp = get_req(&client, &url, &[("col", col.to_string())], tok)?;
            print_response(resp)?;
        }

        HttpCommand::WindowRank { col } => {
            let url = format!("{base}/window/rank");
            let resp = get_req(&client, &url, &[("col", col.to_string())], tok)?;
            print_response(resp)?;
        }

        HttpCommand::WindowDenseRank { col } => {
            let url = format!("{base}/window/dense_rank");
            let resp = get_req(&client, &url, &[("col", col.to_string())], tok)?;
            print_response(resp)?;
        }

        HttpCommand::Distinct { col } => {
            let url = format!("{base}/distinct");
            let resp = get_req(&client, &url, &[("col", col.to_string())], tok)?;
            print_response(resp)?;
        }

        HttpCommand::ValueCounts { col } => {
            let url = format!("{base}/value_counts");
            let resp = get_req(&client, &url, &[("col", col.to_string())], tok)?;
            print_response(resp)?;
        }

        HttpCommand::UpdateWhere {
            col,
            op,
            val,
            target_col,
            new_val,
        } => {
            let url = format!("{base}/update_where");
            let resp = get_req(
                &client,
                &url,
                &[
                    ("col", col.to_string()),
                    ("op", op),
                    ("val", val.to_string()),
                    ("target_col", target_col.to_string()),
                    ("new_val", new_val.to_string()),
                ],
                tok,
            )?;
            print_response(resp)?;
        }

        HttpCommand::IncrementWhere {
            col,
            op,
            val,
            target_col,
            delta,
        } => {
            let url = format!("{base}/increment_where");
            let resp = get_req(
                &client,
                &url,
                &[
                    ("col", col.to_string()),
                    ("op", op),
                    ("val", val.to_string()),
                    ("target_col", target_col.to_string()),
                    ("delta", delta.to_string()),
                ],
                tok,
            )?;
            print_response(resp)?;
        }

        HttpCommand::DecrementWhere {
            col,
            op,
            val,
            target_col,
            delta,
        } => {
            let url = format!("{base}/decrement_where");
            let resp = get_req(
                &client,
                &url,
                &[
                    ("col", col.to_string()),
                    ("op", op),
                    ("val", val.to_string()),
                    ("target_col", target_col.to_string()),
                    ("delta", delta.to_string()),
                ],
                tok,
            )?;
            print_response(resp)?;
        }

        HttpCommand::ScanWhere { preds, limit } => {
            let url = format!("{base}/scan_where");
            let params: Vec<(&str, String)> = Vec::new();

            // Parse each pred string (col:<N>,op:<op>,val:<V>) into indexed params
            for (i, pred) in preds.iter().enumerate() {
                let mut col: Option<String> = None;
                let mut op: Option<String> = None;
                let mut val: Option<String> = None;
                for part in pred.split(',') {
                    if let Some(v) = part.strip_prefix("col:") {
                        col = Some(v.to_string());
                    } else if let Some(v) = part.strip_prefix("op:") {
                        op = Some(v.to_string());
                    } else if let Some(v) = part.strip_prefix("val:") {
                        val = Some(v.to_string());
                    }
                }
                let col = col.with_context(|| format!("pred {i}: missing col:<N>"))?;
                let op = op.with_context(|| format!("pred {i}: missing op:<op>"))?;
                let val = val.with_context(|| format!("pred {i}: missing val:<V>"))?;
                // We push as (key, value) but the key is dynamic — build owned strings and leak
                // them for the lifetime of this function via a local vec instead.
                let _ = (col, op, val); // handled below with indexed approach
            }

            // Build query string manually since param keys are dynamic
            let mut url_with_params = url.clone();
            let mut sep = '?';
            for (i, pred) in preds.iter().enumerate() {
                let mut col: Option<String> = None;
                let mut op: Option<String> = None;
                let mut val: Option<String> = None;
                for part in pred.split(',') {
                    if let Some(v) = part.strip_prefix("col:") {
                        col = Some(v.to_string());
                    } else if let Some(v) = part.strip_prefix("op:") {
                        op = Some(v.to_string());
                    } else if let Some(v) = part.strip_prefix("val:") {
                        val = Some(v.to_string());
                    }
                }
                let col = col.with_context(|| format!("pred {i}: missing col:<N>"))?;
                let op = op.with_context(|| format!("pred {i}: missing op:<op>"))?;
                let val = val.with_context(|| format!("pred {i}: missing val:<V>"))?;
                url_with_params.push_str(&format!("{sep}col{i}={col}&op{i}={op}&val{i}={val}"));
                sep = '&';
            }
            if let Some(lim) = limit {
                url_with_params.push_str(&format!("{sep}limit={lim}"));
            }

            let mut req = client.get(&url_with_params);
            if let Some(auth) = auth_header(tok) {
                req = req.header(AUTHORIZATION, auth);
            }
            let resp = req.send().context("HTTP GET /scan_where failed")?;
            print_response(resp)?;
            let _ = params;
        }

        HttpCommand::ScanProject { cols, limit } => {
            let url = format!("{base}/scan/project");
            let mut params = vec![("cols", cols)];
            let limit_str;
            if let Some(lim) = limit {
                limit_str = lim.to_string();
                params.push(("limit", limit_str));
            }
            // Build URL manually to avoid borrow issues
            let url_with_params = if params.is_empty() {
                url
            } else {
                let qs = params
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join("&");
                format!("{url}?{qs}")
            };
            let mut req = client.get(&url_with_params);
            if let Some(auth) = auth_header(tok) {
                req = req.header(AUTHORIZATION, auth);
            }
            let resp = req.send().context("HTTP GET /scan/project failed")?;
            print_response(resp)?;
        }

        HttpCommand::IfAbsent { pk, values } => {
            let vals = parse_u64_list(&values)?;
            let url = format!("{base}/rows/{pk}/if_absent");
            let resp = post_json(&client, &url, serde_json::json!({ "values": vals }), tok)?;
            print_response(resp)?;
        }

        HttpCommand::Cas {
            pk,
            expected,
            new_values,
        } => {
            let exp = parse_u64_list(&expected)?;
            let new = parse_u64_list(&new_values)?;
            let url = format!("{base}/rows/{pk}/cas");
            let resp = post_json(
                &client,
                &url,
                serde_json::json!({ "expected": exp, "new_values": new }),
                tok,
            )?;
            print_response(resp)?;
        }

        HttpCommand::BatchInsert { rows } => {
            let mut parsed: Vec<serde_json::Value> = Vec::with_capacity(rows.len());
            for row_str in &rows {
                let nums = parse_u64_list(row_str)?;
                if nums.is_empty() {
                    bail!("row must have at least one value (the pk)");
                }
                let pk = nums[0];
                parsed.push(serde_json::json!({ "pk": pk, "values": nums }));
            }
            let url = format!("{base}/batch/insert");
            let resp = post_json(&client, &url, serde_json::json!({ "rows": parsed }), tok)?;
            print_response(resp)?;
        }

        HttpCommand::InsertTtl {
            pk,
            values,
            expire_at,
        } => {
            let vals = parse_u64_list(&values)?;
            let url = format!("{base}/rows/{pk}/ttl");
            let resp = post_json(
                &client,
                &url,
                serde_json::json!({ "values": vals, "expire_at": expire_at }),
                tok,
            )?;
            print_response(resp)?;
        }

        HttpCommand::GetTtl { pk, now } => {
            let url = format!("{base}/get_with_ttl/{pk}");
            let resp = get_req(&client, &url, &[("now", now.to_string())], tok)?;
            print_response(resp)?;
        }

        HttpCommand::ScanTtl { now } => {
            let url = format!("{base}/scan_with_ttl");
            let resp = get_req(&client, &url, &[("now", now.to_string())], tok)?;
            print_response(resp)?;
        }

        HttpCommand::PurgeExpired { now } => {
            let url = format!("{base}/purge_expired");
            let mut req = client.post(&url).query(&[("now", now.to_string())]);
            if let Some(auth) = auth_header(tok) {
                req = req.header(AUTHORIZATION, auth);
            }
            let resp = req.send().context("HTTP POST /purge_expired failed")?;
            print_response(resp)?;
        }
    }

    Ok(())
}

// ── extension trait to apply auth header ─────────────────────────────────────

trait ApplyToken {
    fn apply_token(self, token: &Option<String>) -> Self;
}

impl ApplyToken for reqwest::blocking::RequestBuilder {
    fn apply_token(self, token: &Option<String>) -> Self {
        if let Some(t) = token {
            self.header(AUTHORIZATION, format!("Bearer {t}"))
        } else {
            self
        }
    }
}
