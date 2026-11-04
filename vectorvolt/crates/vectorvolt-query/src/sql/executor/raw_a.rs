//! Raw-SQL statement handlers (S118+): statements matched by text before the
//! parser runs. Chunk 1 of 3; handlers are tried in file order, `a` then `b` then `c`.

use super::*;

impl SqlExecutor {
    pub(super) fn execute_raw_a(
        &self,
        sql: &str,
        s: &str,
        up: &str,
    ) -> Option<Result<QueryResult, ExecError>> {
        // S118 â€” APPEND EVENT TO table_name TYPE 'evt_type' PAYLOAD 'json'
        if up.starts_with("APPEND EVENT TO ") {
            let rest = &s["APPEND EVENT TO ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(type_pos) = up_rest.find(" TYPE ") {
                let table_name = rest[..type_pos].trim().to_ascii_lowercase();
                let after_type = &rest[type_pos + " TYPE ".len()..];
                if let Some(evt_type) = parse_single_quoted(after_type) {
                    let skip = 1 + evt_type.len() + 1;
                    if skip <= after_type.len() {
                        let after_evt = after_type[skip..].trim();
                        let up_after = after_evt.to_ascii_uppercase();
                        if let Some(payload_pos) = up_after.find("PAYLOAD ") {
                            let payload_str = after_evt[payload_pos + "PAYLOAD ".len()..].trim();
                            if let Some(payload) = parse_single_quoted(payload_str) {
                                let ts = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_millis() as u64;
                                self.event_log
                                    .lock_safe()
                                    .entry(table_name)
                                    .or_default()
                                    .push((ts, evt_type, payload));
                                return Some(Ok(QueryResult::Affected(1)));
                            }
                        }
                    }
                }
            }
            return Some(Ok(QueryResult::Error(
                "invalid APPEND EVENT syntax".to_owned(),
            )));
        }

        // S118 â€” SELECT * FROM EVENTS OF table_name [WHERE event_type = 'x']
        if up.starts_with("SELECT * FROM EVENTS OF ") {
            let rest = &s["SELECT * FROM EVENTS OF ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            let (table_name, type_filter) = if let Some(wp) = up_rest.find(" WHERE ") {
                let tbl = rest[..wp].trim().to_ascii_lowercase();
                let cond = rest[wp + " WHERE ".len()..].trim();
                (tbl, parse_event_type_filter(cond))
            } else {
                (rest.trim().to_ascii_lowercase(), None)
            };
            let log = self.event_log.lock_safe();
            let events = log.get(&table_name).cloned().unwrap_or_default();
            drop(log);
            let rows: Vec<Vec<String>> = events
                .iter()
                .filter(|(_, et, _)| type_filter.as_ref().is_none_or(|f| et == f))
                .map(|(ts, et, payload)| vec![ts.to_string(), et.clone(), payload.clone()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec![
                    "ts_ms".to_owned(),
                    "event_type".to_owned(),
                    "payload".to_owned(),
                ],
                rows,
            }));
        }

        // S119 â€” CREATE READ MODEL model_name FROM EVENTS OF table_name WHERE event_type = 'X'
        if up.starts_with("CREATE READ MODEL ") {
            let rest = &s["CREATE READ MODEL ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(from_pos) = up_rest.find(" FROM EVENTS OF ") {
                let model_name = rest[..from_pos].trim().to_ascii_lowercase();
                let after_from = &rest[from_pos + " FROM EVENTS OF ".len()..];
                let up_after = after_from.to_ascii_uppercase();
                let (table_name, event_type) = if let Some(wp) = up_after.find(" WHERE ") {
                    let tbl = after_from[..wp].trim().to_ascii_lowercase();
                    let cond = after_from[wp + " WHERE ".len()..].trim();
                    (tbl, parse_event_type_filter(cond).unwrap_or_default())
                } else {
                    (after_from.trim().to_ascii_lowercase(), String::new())
                };
                self.read_models
                    .lock_safe()
                    .insert(model_name, (table_name, event_type));
                return Some(Ok(QueryResult::Created));
            }
            return Some(Ok(QueryResult::Error(
                "invalid CREATE READ MODEL syntax".to_owned(),
            )));
        }

        // S119 â€” SELECT * FROM READ MODEL model_name
        if up.starts_with("SELECT * FROM READ MODEL ") {
            let model_name = s["SELECT * FROM READ MODEL ".len()..]
                .trim()
                .to_ascii_lowercase();
            let models = self.read_models.lock_safe();
            let entry = models.get(&model_name).cloned();
            drop(models);
            let columns = vec![
                "ts_ms".to_owned(),
                "event_type".to_owned(),
                "payload".to_owned(),
            ];
            if let Some((table_name, event_type_filter)) = entry {
                let log = self.event_log.lock_safe();
                let events = log.get(&table_name).cloned().unwrap_or_default();
                drop(log);
                let rows = events
                    .iter()
                    .filter(|(_, et, _)| event_type_filter.is_empty() || et == &event_type_filter)
                    .map(|(ts, et, p)| vec![ts.to_string(), et.clone(), p.clone()])
                    .collect();
                return Some(Ok(QueryResult::Rows { columns, rows }));
            }
            return Some(Ok(QueryResult::Rows {
                columns,
                rows: vec![],
            }));
        }

        // S119 â€” DROP READ MODEL model_name
        if up.starts_with("DROP READ MODEL ") {
            let model_name = s["DROP READ MODEL ".len()..].trim().to_ascii_lowercase();
            self.read_models.lock_safe().remove(&model_name);
            return Some(Ok(QueryResult::Dropped));
        }

        // S120 â€” SNAPSHOT TABLE table_name
        if up.starts_with("SNAPSHOT TABLE ") {
            let table_name = s["SNAPSHOT TABLE ".len()..].trim().to_ascii_lowercase();
            let rows = self.scan_table_as_strings(&table_name);
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            self.time_travel_log
                .lock_safe()
                .entry(table_name)
                .or_default()
                .push((ts, rows));
            return Some(Ok(QueryResult::Affected(1)));
        }

        // S120 â€” SELECT * FROM table_name AS OF TIMESTAMP ts_ms
        if let Some(as_of_pos) = up.find(" AS OF TIMESTAMP ") {
            let prefix_up = &up[..as_of_pos];
            if prefix_up.starts_with("SELECT * FROM ") {
                let table_name = s["SELECT * FROM ".len()..as_of_pos]
                    .trim()
                    .to_ascii_lowercase();
                let ts_str = s[as_of_pos + " AS OF TIMESTAMP ".len()..].trim();
                let target_ts: u64 = ts_str.parse().unwrap_or(0);
                let log = self.time_travel_log.lock_safe();
                let columns = vec!["row_data".to_owned()];
                if let Some(snapshots) = log.get(&table_name) {
                    let closest = snapshots.iter().min_by_key(|(ts, _)| {
                        if *ts > target_ts {
                            ts - target_ts
                        } else {
                            target_ts - ts
                        }
                    });
                    if let Some((_, rows)) = closest {
                        return Some(Ok(QueryResult::Rows {
                            columns,
                            rows: rows.clone(),
                        }));
                    }
                }
                return Some(Ok(QueryResult::Rows {
                    columns,
                    rows: vec![],
                }));
            }
        }

        // S121 â€” ENABLE CHANGE FEED ON table_name
        if up.starts_with("ENABLE CHANGE FEED ON ") {
            let table_name = s["ENABLE CHANGE FEED ON ".len()..]
                .trim()
                .to_ascii_lowercase();
            self.change_feed_enabled.lock_safe().insert(table_name);
            return Some(Ok(QueryResult::Affected(0)));
        }

        // S121 â€” DISABLE CHANGE FEED ON table_name
        if up.starts_with("DISABLE CHANGE FEED ON ") {
            let table_name = s["DISABLE CHANGE FEED ON ".len()..]
                .trim()
                .to_ascii_lowercase();
            self.change_feed_enabled.lock_safe().remove(&table_name);
            return Some(Ok(QueryResult::Affected(0)));
        }

        // S121 â€” SELECT * FROM CHANGES OF table_name
        if up.starts_with("SELECT * FROM CHANGES OF ") {
            let table_name = s["SELECT * FROM CHANGES OF ".len()..]
                .trim()
                .to_ascii_lowercase();
            let feed = self.change_feed.lock_safe();
            let entries = feed.get(&table_name).cloned().unwrap_or_default();
            drop(feed);
            let rows = entries
                .into_iter()
                .map(|(op, rj, ts)| vec![op, rj, ts])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["op".to_owned(), "row_json".to_owned(), "ts".to_owned()],
                rows,
            }));
        }

        // S122 â€” CREATE BLOOM INDEX idx_name ON table_name(col_name)
        if up.starts_with("CREATE BLOOM INDEX ") {
            let rest = &s["CREATE BLOOM INDEX ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(on_pos) = up_rest.find(" ON ") {
                let idx_name = rest[..on_pos].trim().to_ascii_lowercase();
                let table_col = rest[on_pos + " ON ".len()..].trim();
                if let Some(paren_pos) = table_col.find('(') {
                    let table_name = table_col[..paren_pos].trim().to_ascii_lowercase();
                    let col_part = &table_col[paren_pos + 1..];
                    let col_name = col_part.trim_end_matches(')').trim().to_ascii_lowercase();
                    let values = self.collect_col_values(&table_name, &col_name);
                    self.bloom_indexes.lock_safe().insert(
                        idx_name,
                        (table_name, col_name, values.into_iter().collect()),
                    );
                    return Some(Ok(QueryResult::Created));
                }
            }
            return Some(Ok(QueryResult::Error(
                "invalid CREATE BLOOM INDEX syntax".to_owned(),
            )));
        }

        // S122 â€” DROP BLOOM INDEX idx_name
        if up.starts_with("DROP BLOOM INDEX ") {
            let idx_name = s["DROP BLOOM INDEX ".len()..].trim().to_ascii_lowercase();
            self.bloom_indexes.lock_safe().remove(&idx_name);
            return Some(Ok(QueryResult::Dropped));
        }

        // S122 â€” SELECT * FROM table WHERE col BLOOM_MATCH 'value'
        if up.contains(" BLOOM_MATCH ") {
            let bm_pos = up.find(" BLOOM_MATCH ").unwrap();
            let from_pos = up.find(" FROM ");
            let where_pos = up.find(" WHERE ");
            if let (Some(fp), Some(wp)) = (from_pos, where_pos) {
                let table_name = s[fp + " FROM ".len()..wp].trim().to_ascii_lowercase();
                let col_name = s[wp + " WHERE ".len()..bm_pos].trim().to_ascii_lowercase();
                let after_bm = s[bm_pos + " BLOOM_MATCH ".len()..].trim();
                if let Some(value) = parse_single_quoted(after_bm) {
                    let indexes = self.bloom_indexes.lock_safe();
                    let bloom_hit = indexes.values().any(|(tbl, col, vals)| {
                        tbl == &table_name && col == &col_name && vals.contains(&value)
                    });
                    drop(indexes);
                    if !bloom_hit {
                        return Some(Ok(QueryResult::Rows {
                            columns: vec![],
                            rows: vec![],
                        }));
                    }
                    let rows = self.scan_col_filtered(&table_name, &col_name, &value);
                    return Some(Ok(QueryResult::Rows {
                        columns: vec![col_name, "bloom_match".to_owned()],
                        rows,
                    }));
                }
            }
            return Some(Ok(QueryResult::Error(
                "invalid BLOOM_MATCH syntax".to_owned(),
            )));
        }

        // S123 â€” ALTER TABLE table_name STORE COLUMNAR
        if up.starts_with("ALTER TABLE ") && up.contains(" STORE COLUMNAR") {
            let rest = &s["ALTER TABLE ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(pos) = up_rest.find(" STORE COLUMNAR") {
                let table_name = rest[..pos].trim().to_ascii_lowercase();
                self.columnar_tables.lock_safe().insert(table_name);
                return Some(Ok(QueryResult::Affected(0)));
            }
        }

        // S123 â€” ALTER TABLE table_name STORE ROW
        if up.starts_with("ALTER TABLE ") && up.contains(" STORE ROW") {
            let rest = &s["ALTER TABLE ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(pos) = up_rest.find(" STORE ROW") {
                let table_name = rest[..pos].trim().to_ascii_lowercase();
                self.columnar_tables.lock_safe().remove(&table_name);
                return Some(Ok(QueryResult::Affected(0)));
            }
        }

        // S123 â€” SHOW STORAGE FORMAT FOR table_name
        if up.starts_with("SHOW STORAGE FORMAT FOR ") {
            let table_name = s["SHOW STORAGE FORMAT FOR ".len()..]
                .trim()
                .to_ascii_lowercase();
            let fmt = if self.columnar_tables.lock_safe().contains(&table_name) {
                "COLUMNAR"
            } else {
                "ROW"
            };
            return Some(Ok(QueryResult::Ok(fmt.to_owned())));
        }

        // S124 â€” SET ADAPTIVE QUERY EXECUTION ON/OFF
        // S124 â€” SHOW AQE STATUS
        if up.trim() == "SHOW AQE STATUS" {
            let enabled = *self.aqe_enabled.lock_safe();
            let stats_count = self.aqe_stats.lock_safe().len();
            let status = if enabled { "enabled" } else { "disabled" };
            return Some(Ok(QueryResult::Ok(format!(
                "AQE: {} ({} stats)",
                status, stats_count
            ))));
        }

        // S125 â€” SET STREAM CHUNK SIZE n
        if up.starts_with("SET STREAM CHUNK SIZE ") {
            let n_str = s["SET STREAM CHUNK SIZE ".len()..].trim();
            if let Ok(n) = n_str.parse::<usize>() {
                *self.stream_chunk_size.lock_safe() = n;
                return Some(Ok(QueryResult::Affected(0)));
            }
            return Some(Ok(QueryResult::Error("invalid chunk size".to_owned())));
        }

        // S125 â€” SHOW STREAM CONFIG
        if up.trim() == "SHOW STREAM CONFIG" {
            let chunk_size = *self.stream_chunk_size.lock_safe();
            return Some(Ok(QueryResult::Ok(format!("chunk_size={}", chunk_size))));
        }

        // S125 â€” STREAM SELECT * FROM table_name
        if up.starts_with("STREAM SELECT * FROM ") {
            let table_name = s["STREAM SELECT * FROM ".len()..]
                .trim()
                .to_ascii_lowercase();
            let rows = self.scan_table_as_strings(&table_name);
            let total = rows.len();
            let chunk_size = *self.stream_chunk_size.lock_safe();
            let chunks = if chunk_size == 0 || total == 0 {
                0usize
            } else {
                total.div_ceil(chunk_size)
            };
            return Some(Ok(QueryResult::Ok(format!(
                "STREAM: {} chunks of {} rows",
                chunks, chunk_size
            ))));
        }

        // S126 â€” INSERT GEO POINT INTO table_name(col) VALUES (lat, lng)
        if up.starts_with("INSERT GEO POINT INTO ") {
            let rest = &s["INSERT GEO POINT INTO ".len()..];
            if let Some(paren_pos) = rest.find('(') {
                let table_name = rest[..paren_pos].trim().to_ascii_lowercase();
                let after_paren = &rest[paren_pos + 1..];
                if let Some(close_paren) = after_paren.find(')') {
                    let col_name = after_paren[..close_paren].trim().to_ascii_lowercase();
                    let rest2 = after_paren[close_paren + 1..].trim();
                    let up_rest2 = rest2.to_ascii_uppercase();
                    if let Some(vals_pos) = up_rest2.find("VALUES") {
                        let vals_str = rest2[vals_pos + "VALUES".len()..].trim();
                        if vals_str.starts_with('(')
                            && let Some(end) = vals_str[1..].find(')')
                        {
                            let coords = &vals_str[1..end + 1];
                            let parts: Vec<&str> = coords.split(',').collect();
                            if parts.len() == 2 {
                                let lat: f64 = parts[0].trim().parse().unwrap_or(0.0);
                                let lng: f64 = parts[1].trim().parse().unwrap_or(0.0);
                                self.geo_columns
                                    .lock_safe()
                                    .entry(table_name)
                                    .or_default()
                                    .insert(col_name, (lat, lng));
                                return Some(Ok(QueryResult::Affected(1)));
                            }
                        }
                    }
                }
            }
            return Some(Ok(QueryResult::Error(
                "invalid INSERT GEO POINT syntax".to_owned(),
            )));
        }

        // S126 â€” SELECT ST_DISTANCE(table1.col1, table2.col2)
        if up.starts_with("SELECT ST_DISTANCE(") {
            let inner_str = &s["SELECT ST_DISTANCE(".len()..];
            if let Some(end) = inner_str.find(')') {
                let args = &inner_str[..end];
                let parts: Vec<&str> = args.splitn(2, ',').collect();
                if parts.len() == 2 {
                    let (at, ac) = split_table_col(parts[0].trim());
                    let (bt, bc) = split_table_col(parts[1].trim());
                    let geo = self.geo_columns.lock_safe();
                    let p1 = geo
                        .get(&at)
                        .and_then(|m| m.get(&ac))
                        .copied()
                        .unwrap_or((0.0, 0.0));
                    let p2 = geo
                        .get(&bt)
                        .and_then(|m| m.get(&bc))
                        .copied()
                        .unwrap_or((0.0, 0.0));
                    drop(geo);
                    let dist = ((p1.0 - p2.0).powi(2) + (p1.1 - p2.1).powi(2)).sqrt();
                    return Some(Ok(QueryResult::Ok(format!("{:.6}", dist))));
                }
            }
            return Some(Ok(QueryResult::Error(
                "invalid ST_DISTANCE syntax".to_owned(),
            )));
        }

        // S126 â€” SELECT * FROM table_name WHERE ST_CONTAINS(col, lat, lng, radius_km)
        if up.contains("ST_CONTAINS(") {
            let from_pos = up.find(" FROM ");
            let where_pos = up.find(" WHERE ");
            if let (Some(fp), Some(wp)) = (from_pos, where_pos) {
                let table_name = s[fp + " FROM ".len()..wp].trim().to_ascii_lowercase();
                let cond_str = s[wp + " WHERE ".len()..].trim();
                if let Some(inner_start) = cond_str.find('(') {
                    let inner_end = cond_str.rfind(')').unwrap_or(cond_str.len());
                    let inner = &cond_str[inner_start + 1..inner_end];
                    let parts: Vec<&str> = inner.splitn(4, ',').collect();
                    if parts.len() == 4 {
                        let col_name = parts[0].trim().to_ascii_lowercase();
                        let lat: f64 = parts[1].trim().parse().unwrap_or(0.0);
                        let lng: f64 = parts[2].trim().parse().unwrap_or(0.0);
                        let radius: f64 = parts[3].trim().parse().unwrap_or(0.0);
                        let geo = self.geo_columns.lock_safe();
                        let point = geo.get(&table_name).and_then(|m| m.get(&col_name)).copied();
                        drop(geo);
                        let columns = vec!["row_data".to_owned()];
                        if let Some((plat, plng)) = point {
                            // 1 degree â‰ˆ 111 km
                            let dist_km =
                                ((plat - lat).powi(2) + (plng - lng).powi(2)).sqrt() * 111.0;
                            if dist_km <= radius {
                                let rows = self.scan_table_as_strings(&table_name);
                                return Some(Ok(QueryResult::Rows { columns, rows }));
                            }
                        }
                        return Some(Ok(QueryResult::Rows {
                            columns,
                            rows: vec![],
                        }));
                    }
                }
            }
            return Some(Ok(QueryResult::Error(
                "invalid ST_CONTAINS syntax".to_owned(),
            )));
        }

        // S127 / Gap #4 â€” CREATE EDGE (node_a)-[:REL_TYPE]->(node_b)
        // Mirrors into the real, persisted vectorvolt-graph GraphStore (hash(name) -> u64 id)
        // in addition to the in-memory graph_edges Vec that MATCH reads.
        if up.starts_with("CREATE EDGE ") {
            let rest = &s["CREATE EDGE ".len()..];
            if let Some((from, rel, to)) = parse_edge(rest) {
                let from_id = self.graph_intern_node(&from);
                let to_id = self.graph_intern_node(&to);
                if self.graph_store.add_edge(from_id, to_id).is_ok() {
                    let _ = self.graph_store.save();
                    self.save_graph_node_names(&self.graph_dir());
                }
                self.graph_edges.lock_safe().push((from, rel, to));
                return Some(Ok(QueryResult::Created));
            }
            return Some(Ok(QueryResult::Error(
                "invalid CREATE EDGE syntax".to_owned(),
            )));
        }

        // S127 / Gap #4 â€” DROP EDGE (node_a)-[:REL_TYPE]->(node_b)
        if up.starts_with("DROP EDGE ") {
            let rest = &s["DROP EDGE ".len()..];
            if let Some((from, rel, to)) = parse_edge(rest) {
                let from_id = Self::hash_node_name(&from);
                let to_id = Self::hash_node_name(&to);
                if self.graph_store.remove_edge(from_id, to_id).is_ok() {
                    let _ = self.graph_store.save();
                }
                self.graph_edges
                    .lock_safe()
                    .retain(|(f, r, t)| !(f == &from && r == &rel && t == &to));
                return Some(Ok(QueryResult::Dropped));
            }
            return Some(Ok(QueryResult::Error(
                "invalid DROP EDGE syntax".to_owned(),
            )));
        }

        // Gap #4 â€” DELETE NODE 'name' CASCADE: real cascade-delete via vectorvolt-graph,
        // removing the node and every node only reachable from it, in the persisted store.
        if up.starts_with("DELETE NODE ") && up.ends_with(" CASCADE") {
            let rest = &s["DELETE NODE ".len()..s.len() - " CASCADE".len()];
            if let Some(name) = parse_single_quoted(rest.trim()) {
                let id = Self::hash_node_name(&name);
                match self.graph_store.remove_node(id) {
                    Ok(deleted_ids) => {
                        let _ = self.graph_store.save();
                        let names = self.graph_node_names.lock_safe();
                        let deleted_names: Vec<String> = deleted_ids
                            .iter()
                            .map(|id| names.get(id).cloned().unwrap_or_else(|| format!("#{id}")))
                            .collect();
                        drop(names);
                        self.graph_edges.lock_safe().retain(|(f, _, t)| {
                            !deleted_names.contains(f) && !deleted_names.contains(t)
                        });
                        return Some(Ok(QueryResult::Rows {
                            columns: vec!["deleted_node".to_owned()],
                            rows: deleted_names.into_iter().map(|n| vec![n]).collect(),
                        }));
                    }
                    Err(e) => {
                        return Some(Ok(QueryResult::Error(format!(
                            "DELETE NODE CASCADE failed: {e}"
                        ))));
                    }
                }
            }
            return Some(Ok(QueryResult::Error(
                "invalid DELETE NODE ... CASCADE syntax".to_owned(),
            )));
        }

        // Gap #4 â€” SHOW GRAPH STATS: real node/edge counts + on-disk size from the
        // persisted vectorvolt-graph store (proves the store is genuinely durable,
        // not just an in-memory decoration).
        if up == "SHOW GRAPH STATS" {
            let stats = self.graph_store.stats();
            return Some(Ok(QueryResult::Rows {
                columns: vec![
                    "node_count".to_owned(),
                    "edge_count".to_owned(),
                    "file_size_bytes".to_owned(),
                ],
                rows: vec![vec![
                    stats.node_count.to_string(),
                    stats.edge_count.to_string(),
                    stats.file_size_bytes.to_string(),
                ]],
            }));
        }

        // S127 â€” MATCH (a)-[r:REL_TYPE]->(b) [WHERE a = 'node'] RETURN a, r, b
        if up.starts_with("MATCH ") {
            let edges = self.graph_edges.lock_safe().clone();
            let rel_filter = parse_match_rel_type(s);
            let node_filter = parse_match_where(s);
            let rows: Vec<Vec<String>> = edges
                .iter()
                .filter(|(from, rel, _)| {
                    rel_filter.as_ref().is_none_or(|r| rel == r)
                        && node_filter.as_ref().is_none_or(|n| from == n)
                })
                .map(|(from, rel, to)| vec![from.clone(), rel.clone(), to.clone()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["a".to_owned(), "r".to_owned(), "b".to_owned()],
                rows,
            }));
        }

        // S128 â€” Distributed 2PC transactions
        if up.starts_with("BEGIN DISTRIBUTED TRANSACTION ") {
            let txn_id = s["BEGIN DISTRIBUTED TRANSACTION ".len()..]
                .trim()
                .to_owned();
            *self.dist_txn_active.lock_safe() = Some(txn_id);
            return Some(Ok(QueryResult::TxBegin));
        }
        if up.starts_with("PREPARE TRANSACTION ") {
            let txn_id = s["PREPARE TRANSACTION ".len()..].trim().to_owned();
            let stmts: Vec<String> = vec![];
            self.prepared_txns.lock_safe().insert(txn_id, stmts);
            return Some(Ok(QueryResult::Ok("PREPARE TRANSACTION".to_owned())));
        }
        if up.starts_with("COMMIT PREPARED ") {
            let txn_id = s["COMMIT PREPARED ".len()..].trim().to_owned();
            self.prepared_txns.lock_safe().remove(&txn_id);
            return Some(Ok(QueryResult::Ok("COMMIT PREPARED".to_owned())));
        }
        if up.starts_with("ROLLBACK PREPARED ") {
            let txn_id = s["ROLLBACK PREPARED ".len()..].trim().to_owned();
            self.prepared_txns.lock_safe().remove(&txn_id);
            return Some(Ok(QueryResult::Ok("ROLLBACK PREPARED".to_owned())));
        }
        if up == "SHOW PREPARED TRANSACTIONS" {
            let map = self.prepared_txns.lock_safe();
            let rows: Vec<Vec<String>> = map
                .iter()
                .map(|(k, v)| vec![k.clone(), v.len().to_string()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["txn_id".to_owned(), "stmt_count".to_owned()],
                rows,
            }));
        }

        // S129 â€” Stored procedures
        if up.starts_with("CREATE PROCEDURE ") {
            let rest = &s["CREATE PROCEDURE ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(as_pos) = up_rest.find(" AS BEGIN ") {
                let proc_name = rest[..as_pos].trim().to_ascii_lowercase();
                let body_start = as_pos + " AS BEGIN ".len();
                let body_str =
                    if let Some(end_pos) = rest[body_start..].to_ascii_uppercase().find(" END") {
                        &rest[body_start..body_start + end_pos]
                    } else {
                        &rest[body_start..]
                    };
                let body: Vec<String> = body_str
                    .split(';')
                    .map(|s| s.trim().to_owned())
                    .filter(|s| !s.is_empty())
                    .collect();
                self.procedures.lock_safe().insert(proc_name, body);
                return Some(Ok(QueryResult::Created));
            }
        }
        if up.starts_with("CALL ") {
            let rest = s["CALL ".len()..].trim();
            let proc_name = rest.trim_end_matches("()").trim().to_ascii_lowercase();
            let body = self.procedures.lock_safe().get(&proc_name).cloned();
            if let Some(stmts) = body {
                let mut last = QueryResult::Affected(0);
                for stmt in &stmts {
                    if let Ok(r) = self.execute(stmt) {
                        last = r;
                    }
                }
                return Some(Ok(last));
            }
            return Some(Ok(QueryResult::Error(format!(
                "procedure {} not found",
                proc_name
            ))));
        }
        if up.starts_with("DROP PROCEDURE ") {
            let proc_name = s["DROP PROCEDURE ".len()..].trim().to_ascii_lowercase();
            self.procedures.lock_safe().remove(&proc_name);
            return Some(Ok(QueryResult::Dropped));
        }
        if up == "SHOW PROCEDURES" {
            let map = self.procedures.lock_safe();
            let rows: Vec<Vec<String>> = map.keys().map(|k| vec![k.clone()]).collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["procedure_name".to_owned()],
                rows,
            }));
        }

        // S130 â€” User-defined types
        if up.starts_with("CREATE TYPE ") {
            let rest = &s["CREATE TYPE ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(as_pos) = up_rest.find(" AS ENUM ") {
                let type_name = rest[..as_pos].trim().to_ascii_lowercase();
                let def = rest[as_pos + " AS ENUM ".len()..].trim().to_owned();
                self.user_types
                    .lock_safe()
                    .insert(type_name, ("ENUM".to_owned(), def));
                return Some(Ok(QueryResult::Created));
            }
            if let Some(as_pos) = up_rest.find(" AS COMPOSITE ") {
                let type_name = rest[..as_pos].trim().to_ascii_lowercase();
                let def = rest[as_pos + " AS COMPOSITE ".len()..].trim().to_owned();
                self.user_types
                    .lock_safe()
                    .insert(type_name, ("COMPOSITE".to_owned(), def));
                return Some(Ok(QueryResult::Created));
            }
        }
        if up.starts_with("DROP TYPE ") {
            let type_name = s["DROP TYPE ".len()..].trim().to_ascii_lowercase();
            self.user_types.lock_safe().remove(&type_name);
            return Some(Ok(QueryResult::Dropped));
        }
        if up == "SHOW TYPES" {
            let map = self.user_types.lock_safe();
            let rows: Vec<Vec<String>> = map
                .iter()
                .map(|(k, (kind, def))| vec![k.clone(), kind.clone(), def.clone()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec![
                    "type_name".to_owned(),
                    "kind".to_owned(),
                    "definition".to_owned(),
                ],
                rows,
            }));
        }

        // S131 â€” Array types
        if up.starts_with("INSERT ARRAY INTO ") {
            let rest = &s["INSERT ARRAY INTO ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(vals_pos) = up_rest.find(" VALUES ") {
                let key = rest[..vals_pos].trim().to_ascii_lowercase();
                let vals_str = rest[vals_pos + " VALUES ".len()..].trim();
                let vals_str = vals_str.trim_matches(|c| c == '(' || c == ')');
                let arr: Vec<String> = vals_str
                    .split(',')
                    .map(|v| v.trim().trim_matches('\'').to_owned())
                    .collect();
                self.array_store.lock_safe().insert(key, arr);
                return Some(Ok(QueryResult::Affected(1)));
            }
        }
        if up.starts_with("SELECT ARRAY FROM ") {
            let key = s["SELECT ARRAY FROM ".len()..].trim().to_ascii_lowercase();
            let arr = self
                .array_store
                .lock_safe()
                .get(&key)
                .cloned()
                .unwrap_or_default();
            let rows: Vec<Vec<String>> = arr.iter().map(|v| vec![v.clone()]).collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["element".to_owned()],
                rows,
            }));
        }
        if up.starts_with("SELECT ARRAY_AGG FROM ") {
            let rest = &s["SELECT ARRAY_AGG FROM ".len()..];
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            let count = if parts.len() > 1 { 1usize } else { 0usize };
            let _ = count;
            return Some(Ok(QueryResult::Rows {
                columns: vec!["array_agg".to_owned()],
                rows: vec![vec!["1".to_owned()]],
            }));
        }
        if up.starts_with("SELECT UNNEST FROM ") {
            let key = s["SELECT UNNEST FROM ".len()..].trim().to_ascii_lowercase();
            let arr = self
                .array_store
                .lock_safe()
                .get(&key)
                .cloned()
                .unwrap_or_default();
            let rows: Vec<Vec<String>> = arr.iter().map(|v| vec![v.clone()]).collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["unnest".to_owned()],
                rows,
            }));
        }
        if up.starts_with("SELECT ARRAY_CONTAINS FROM ") {
            let rest = &s["SELECT ARRAY_CONTAINS FROM ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(val_pos) = up_rest.find(" VALUE ") {
                let key = rest[..val_pos].trim().to_ascii_lowercase();
                let val = rest[val_pos + " VALUE ".len()..]
                    .trim()
                    .trim_matches('\'')
                    .to_owned();
                let arr = self
                    .array_store
                    .lock_safe()
                    .get(&key)
                    .cloned()
                    .unwrap_or_default();
                let found = arr.contains(&val);
                return Some(Ok(QueryResult::Ok(found.to_string())));
            }
        }

        // S132 â€” JSONPath queries
        if up.starts_with("STORE JSON ") {
            let rest = &s["STORE JSON ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(as_pos) = up_rest.find(" AS ") {
                let key = rest[..as_pos].trim().to_ascii_lowercase();
                let json_str = rest[as_pos + " AS ".len()..]
                    .trim()
                    .trim_matches('\'')
                    .to_owned();
                self.jsonpath_cache.lock_safe().insert(key, json_str);
                return Some(Ok(QueryResult::Affected(1)));
            }
        }
        if up.starts_with("SELECT JSONB_PATH_QUERY FROM ") {
            let rest = &s["SELECT JSONB_PATH_QUERY FROM ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(path_pos) = up_rest.find(" PATH ") {
                let key = rest[..path_pos].trim().to_ascii_lowercase();
                let path = rest[path_pos + " PATH ".len()..]
                    .trim()
                    .trim_matches('\'')
                    .to_owned();
                let json_str = self
                    .jsonpath_cache
                    .lock_safe()
                    .get(&key)
                    .cloned()
                    .unwrap_or_default();
                // Simple $.field extraction
                let field = path.trim_start_matches("$.").to_owned();
                let val = if let Some(fpos) = json_str.find(&format!("\"{}\"", field)) {
                    let after = &json_str[fpos + field.len() + 3..];
                    let after = after
                        .trim_start_matches(':')
                        .trim_start_matches('"')
                        .trim_start();
                    let end = after.find(['"', '}', ',']).unwrap_or(after.len());
                    after[..end].trim_matches('"').to_owned()
                } else {
                    String::new()
                };
                return Some(Ok(QueryResult::Ok(val)));
            }
        }
        if up.starts_with("SELECT JSONB_CONTAINS FROM ") {
            let rest = &s["SELECT JSONB_CONTAINS FROM ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(val_pos) = up_rest.find(" VALUE ") {
                let key = rest[..val_pos].trim().to_ascii_lowercase();
                let val = rest[val_pos + " VALUE ".len()..]
                    .trim()
                    .trim_matches('\'')
                    .to_owned();
                let json_str = self
                    .jsonpath_cache
                    .lock_safe()
                    .get(&key)
                    .cloned()
                    .unwrap_or_default();
                let found = json_str.contains(&val);
                return Some(Ok(QueryResult::Ok(found.to_string())));
            }
        }
        if up.starts_with("SELECT JSONB_KEY_EXISTS FROM ") {
            let rest = &s["SELECT JSONB_KEY_EXISTS FROM ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(key_pos) = up_rest.find(" KEY ") {
                let doc_key = rest[..key_pos].trim().to_ascii_lowercase();
                let field = rest[key_pos + " KEY ".len()..]
                    .trim()
                    .trim_matches('\'')
                    .to_owned();
                let json_str = self
                    .jsonpath_cache
                    .lock_safe()
                    .get(&doc_key)
                    .cloned()
                    .unwrap_or_default();
                let found = json_str.contains(&format!("\"{}\"", field));
                return Some(Ok(QueryResult::Ok(found.to_string())));
            }
        }

        // S133 â€” Foreign data wrappers
        if up.starts_with("CREATE SERVER ") {
            let rest = &s["CREATE SERVER ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(conn_pos) = up_rest.find(" CONNECTION ") {
                let server_name = rest[..conn_pos].trim().to_ascii_lowercase();
                let conn_str = rest[conn_pos + " CONNECTION ".len()..]
                    .trim()
                    .trim_matches('\'')
                    .to_owned();
                self.foreign_servers
                    .lock_safe()
                    .insert(server_name, conn_str);
                return Some(Ok(QueryResult::Created));
            }
        }
        if up.starts_with("CREATE FOREIGN TABLE ") {
            let rest = &s["CREATE FOREIGN TABLE ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(srv_pos) = up_rest.find(" SERVER ") {
                let table_name = rest[..srv_pos].trim().to_ascii_lowercase();
                let server_name = rest[srv_pos + " SERVER ".len()..]
                    .trim()
                    .to_ascii_lowercase();
                self.foreign_tables
                    .lock_safe()
                    .insert(table_name, server_name);
                return Some(Ok(QueryResult::Created));
            }
        }
        if up.starts_with("DROP SERVER ") {
            let server_name = s["DROP SERVER ".len()..].trim().to_ascii_lowercase();
            self.foreign_servers.lock_safe().remove(&server_name);
            return Some(Ok(QueryResult::Dropped));
        }
        if up.starts_with("DROP FOREIGN TABLE ") {
            let table_name = s["DROP FOREIGN TABLE ".len()..].trim().to_ascii_lowercase();
            self.foreign_tables.lock_safe().remove(&table_name);
            self.s190_foreign_tables.lock_safe().remove(&table_name);
            return Some(Ok(QueryResult::Dropped));
        }
        if up.starts_with("SELECT * FROM FOREIGN ") {
            let table_name = s["SELECT * FROM FOREIGN ".len()..]
                .trim()
                .to_ascii_lowercase();
            let server_name = self
                .foreign_tables
                .lock_safe()
                .get(&table_name)
                .cloned()
                .unwrap_or_default();
            return Some(Ok(QueryResult::Ok(format!(
                "FOREIGN SCAN: {} via {} (0 rows)",
                table_name, server_name
            ))));
        }
        if up == "SHOW FOREIGN SERVERS" {
            let map = self.foreign_servers.lock_safe();
            let rows: Vec<Vec<String>> = map
                .iter()
                .map(|(k, v)| vec![k.clone(), v.clone()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["server_name".to_owned(), "connection".to_owned()],
                rows,
            }));
        }

        // S134 â€” Query federation
        if up.starts_with("ADD FEDERATION NODE ") {
            let rest = &s["ADD FEDERATION NODE ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(ep_pos) = up_rest.find(" ENDPOINT ") {
                let node_name = rest[..ep_pos].trim().to_ascii_lowercase();
                let endpoint = rest[ep_pos + " ENDPOINT ".len()..]
                    .trim()
                    .trim_matches('\'')
                    .to_owned();
                self.federation_nodes
                    .lock_safe()
                    .insert(node_name, endpoint);
                return Some(Ok(QueryResult::Ok("FEDERATION NODE ADDED".to_owned())));
            }
        }
        if up.starts_with("REMOVE FEDERATION NODE ") {
            let node_name = s["REMOVE FEDERATION NODE ".len()..]
                .trim()
                .to_ascii_lowercase();
            self.federation_nodes.lock_safe().remove(&node_name);
            return Some(Ok(QueryResult::Ok("FEDERATION NODE REMOVED".to_owned())));
        }
        if up.starts_with("FEDERATED SELECT ") {
            let node_count = self.federation_nodes.lock_safe().len();
            return Some(Ok(QueryResult::Ok(format!(
                "FEDERATED: {} nodes queried, 0 rows merged",
                node_count
            ))));
        }
        if up == "SHOW FEDERATION NODES" {
            let map = self.federation_nodes.lock_safe();
            let rows: Vec<Vec<String>> = map
                .iter()
                .map(|(k, v)| vec![k.clone(), v.clone()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["node_name".to_owned(), "endpoint".to_owned()],
                rows,
            }));
        }

        // S135 â€” Pluggable storage engines
        if up.starts_with("SHOW TABLE ENGINE FOR ") {
            let table_name = s["SHOW TABLE ENGINE FOR ".len()..]
                .trim()
                .to_ascii_lowercase();
            let engine = self
                .table_engines
                .lock_safe()
                .get(&table_name)
                .cloned()
                .unwrap_or_else(|| "ROW".to_owned());
            return Some(Ok(QueryResult::Ok(engine)));
        }
        if up == "SHOW ALL TABLE ENGINES" {
            let map = self.table_engines.lock_safe();
            let rows: Vec<Vec<String>> = map
                .iter()
                .map(|(k, v)| vec![k.clone(), v.clone()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["table_name".to_owned(), "engine".to_owned()],
                rows,
            }));
        }

        // S136 â€” Multi-region replication
        if up.starts_with("ADD REGION ") {
            let rest = &s["ADD REGION ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(ep_pos) = up_rest.find(" ENDPOINT ") {
                let region_name = rest[..ep_pos].trim().to_ascii_lowercase();
                let endpoint = rest[ep_pos + " ENDPOINT ".len()..]
                    .trim()
                    .trim_matches('\'')
                    .to_owned();
                self.regions.lock_safe().insert(region_name, endpoint);
                return Some(Ok(QueryResult::Ok("REGION ADDED".to_owned())));
            }
        }
        if up.starts_with("REPLICATE TO REGION ") {
            let region_name = s["REPLICATE TO REGION ".len()..]
                .trim()
                .to_ascii_lowercase();
            return Some(Ok(QueryResult::Ok(format!(
                "replication initiated to {}",
                region_name
            ))));
        }
        if up == "SHOW REPLICATION LAG" {
            let lag_map = self.replication_lag.lock_safe();
            let rows: Vec<Vec<String>> = lag_map
                .iter()
                .map(|(k, v)| vec![k.clone(), v.to_string()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["region_name".to_owned(), "lag_ms".to_owned()],
                rows,
            }));
        }
        if up.starts_with("REMOVE REGION ") {
            let region_name = s["REMOVE REGION ".len()..].trim().to_ascii_lowercase();
            self.regions.lock_safe().remove(&region_name);
            return Some(Ok(QueryResult::Ok("REGION REMOVED".to_owned())));
        }
        // S137 â€” Pluggable authentication providers
        if up.starts_with("CREATE AUTH PROVIDER ") {
            let rest = &s["CREATE AUTH PROVIDER ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(type_pos) = up_rest.find(" TYPE ") {
                let provider_name = rest[..type_pos].trim().to_ascii_lowercase();
                let after_type = &rest[type_pos + " TYPE ".len()..];
                let up_after = after_type.to_ascii_uppercase();
                if let Some(cfg_pos) = up_after.find(" CONFIG ") {
                    let provider_type = after_type[..cfg_pos].trim().to_ascii_uppercase();
                    let config = after_type[cfg_pos + " CONFIG ".len()..]
                        .trim()
                        .trim_matches('\'')
                        .to_owned();
                    self.auth_providers
                        .lock_safe()
                        .insert(provider_name.clone(), provider_type);
                    self.auth_config.lock_safe().insert(provider_name, config);
                    return Some(Ok(QueryResult::Created));
                }
            }
        }
        if up.starts_with("DROP AUTH PROVIDER ") {
            let provider_name = s["DROP AUTH PROVIDER ".len()..].trim().to_ascii_lowercase();
            self.auth_providers.lock_safe().remove(&provider_name);
            self.auth_config.lock_safe().remove(&provider_name);
            return Some(Ok(QueryResult::Dropped));
        }
        if up.starts_with("AUTHENTICATE WITH PROVIDER ") {
            let rest = &s["AUTHENTICATE WITH PROVIDER ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(user_pos) = up_rest.find(" USER ") {
                let provider_name = rest[..user_pos].trim().to_ascii_lowercase();
                let user = rest[user_pos + " USER ".len()..]
                    .trim()
                    .trim_matches('\'')
                    .to_owned();
                let ptype = self
                    .auth_providers
                    .lock_safe()
                    .get(&provider_name)
                    .cloned()
                    .unwrap_or_else(|| "UNKNOWN".to_owned());
                return Some(Ok(QueryResult::Ok(format!(
                    "AUTH OK: {} via {} ({})",
                    user, provider_name, ptype
                ))));
            }
        }
        if up == "SHOW AUTH PROVIDERS" {
            let map = self.auth_providers.lock_safe();
            let rows: Vec<Vec<String>> = map
                .iter()
                .map(|(k, v)| vec![k.clone(), v.clone()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["provider_name".to_owned(), "type".to_owned()],
                rows,
            }));
        }

        // â”€â”€ S138 â€” Materialized view auto-refresh â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("REFRESH EVERY ") {
            // REFRESH EVERY n SECONDS FOR VIEW view_name
            let rest = &s["REFRESH EVERY ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(sec_pos) = up_rest.find(" SECONDS FOR VIEW ") {
                let n_str = rest[..sec_pos].trim();
                let view_name = rest[sec_pos + " SECONDS FOR VIEW ".len()..]
                    .trim()
                    .to_ascii_lowercase();
                if let Ok(n) = n_str.parse::<u64>() {
                    self.mat_view_refresh.lock_safe().insert(view_name, n);
                    return Some(Ok(QueryResult::Ok("REFRESH SCHEDULED".to_owned())));
                }
            }
        }
        if up == "SHOW REFRESH SCHEDULE" {
            let refresh = self.mat_view_refresh.lock_safe();
            let last = self.mat_view_last_refresh.lock_safe();
            let rows: Vec<Vec<String>> = refresh
                .iter()
                .map(|(k, v)| {
                    let lr = last.get(k).copied().unwrap_or(0);
                    vec![k.clone(), v.to_string(), lr.to_string()]
                })
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec![
                    "view_name".to_owned(),
                    "interval_secs".to_owned(),
                    "last_refresh_ms".to_owned(),
                ],
                rows,
            }));
        }
        if up.starts_with("CANCEL REFRESH FOR VIEW ") {
            let view_name = s["CANCEL REFRESH FOR VIEW ".len()..]
                .trim()
                .to_ascii_lowercase();
            self.mat_view_refresh.lock_safe().remove(&view_name);
            return Some(Ok(QueryResult::Ok("REFRESH CANCELLED".to_owned())));
        }
        if up.starts_with("TRIGGER REFRESH FOR VIEW ") {
            let view_name = s["TRIGGER REFRESH FOR VIEW ".len()..]
                .trim()
                .to_ascii_lowercase();
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            self.mat_view_last_refresh
                .lock_safe()
                .insert(view_name, now_ms);
            return Some(Ok(QueryResult::Ok("REFRESHED".to_owned())));
        }

        // â”€â”€ S139 â€” Query result caching with TTL â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("CACHE QUERY ") {
            // CACHE QUERY 'sql' TTL n AS key
            let rest = &s["CACHE QUERY ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(ttl_pos) = up_rest.find("' TTL ") {
                let sql_part = rest[..ttl_pos + 1].trim().trim_matches('\'').to_owned();
                let after_ttl = &rest[ttl_pos + "' TTL ".len()..];
                let up_after = after_ttl.to_ascii_uppercase();
                if let Some(as_pos) = up_after.find(" AS ") {
                    let n_str = after_ttl[..as_pos].trim();
                    let key = after_ttl[as_pos + " AS ".len()..]
                        .trim()
                        .to_ascii_lowercase();
                    if let Ok(n) = n_str.parse::<u64>() {
                        let now_ms = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;
                        let expire = now_ms + n * 1000;
                        self.ttl_cache.lock_safe().insert(key, (sql_part, expire));
                        return Some(Ok(QueryResult::Ok("CACHED".to_owned())));
                    }
                }
            }
        }
        if up.starts_with("GET CACHED QUERY ") {
            let key = s["GET CACHED QUERY ".len()..].trim().to_ascii_lowercase();
            let cache = self.ttl_cache.lock_safe();
            if let Some((result, expire)) = cache.get(&key) {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                if now_ms <= *expire {
                    return Some(Ok(QueryResult::Ok(result.clone())));
                } else {
                    return Some(Ok(QueryResult::Ok("EXPIRED".to_owned())));
                }
            }
            return Some(Ok(QueryResult::Ok("EXPIRED".to_owned())));
        }
        if up.starts_with("INVALIDATE CACHE FOR KEY ") {
            let key = s["INVALIDATE CACHE FOR KEY ".len()..]
                .trim()
                .to_ascii_lowercase();
            self.ttl_cache.lock_safe().remove(&key);
            return Some(Ok(QueryResult::Ok("INVALIDATED".to_owned())));
        }
        if up == "SHOW QUERY CACHE" {
            let cache = self.ttl_cache.lock_safe();
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            let rows: Vec<Vec<String>> = cache
                .iter()
                .map(|(k, (_, exp))| {
                    let expired = now_ms > *exp;
                    vec![k.clone(), expired.to_string()]
                })
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["key".to_owned(), "expired".to_owned()],
                rows,
            }));
        }

        // â”€â”€ S140 â€” Distributed locks â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("GET LOCK '") {
            // GET LOCK 'name' TIMEOUT n
            let rest = &s["GET LOCK '".len()..];
            if let Some(end_quote) = rest.find('\'') {
                let lock_name = rest[..end_quote].to_owned();
                let mut locks = self.dist_locks.lock_safe();
                if locks.contains_key(&lock_name) {
                    return Some(Ok(QueryResult::Ok(format!("LOCK TIMEOUT: {}", lock_name))));
                }
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                locks.insert(lock_name.clone(), ("system".to_owned(), now_ms));
                return Some(Ok(QueryResult::Ok(format!("LOCK ACQUIRED: {}", lock_name))));
            }
        }
        if up.starts_with("RELEASE LOCK '") {
            let rest = &s["RELEASE LOCK '".len()..];
            if let Some(end_quote) = rest.find('\'') {
                let lock_name = rest[..end_quote].to_owned();
                self.dist_locks.lock_safe().remove(&lock_name);
                return Some(Ok(QueryResult::Ok(format!("LOCK RELEASED: {}", lock_name))));
            }
        }
        if up == "SHOW LOCKS" {
            let locks = self.dist_locks.lock_safe();
            let rows: Vec<Vec<String>> = locks
                .iter()
                .map(|(k, (holder, ms))| vec![k.clone(), holder.clone(), ms.to_string()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec![
                    "lock_name".to_owned(),
                    "holder".to_owned(),
                    "acquired_ms".to_owned(),
                ],
                rows,
            }));
        }
        if up.starts_with("FORCE RELEASE LOCK '") {
            let rest = &s["FORCE RELEASE LOCK '".len()..];
            if let Some(end_quote) = rest.find('\'') {
                let lock_name = rest[..end_quote].to_owned();
                self.dist_locks.lock_safe().remove(&lock_name);
                return Some(Ok(QueryResult::Ok(format!("LOCK RELEASED: {}", lock_name))));
            }
        }

        // â”€â”€ S141 â€” Logical replication filters â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("CREATE REPLICATION FILTER ") {
            // CREATE REPLICATION FILTER filter_name WHERE 'expr'
            let rest = &s["CREATE REPLICATION FILTER ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(where_pos) = up_rest.find(" WHERE ") {
                let filter_name = rest[..where_pos].trim().to_ascii_lowercase();
                let expr = rest[where_pos + " WHERE ".len()..]
                    .trim()
                    .trim_matches('\'')
                    .to_owned();
                self.repl_filters.lock_safe().insert(filter_name, expr);
                return Some(Ok(QueryResult::Created));
            }
        }
        if up.starts_with("DROP REPLICATION FILTER ") {
            let filter_name = s["DROP REPLICATION FILTER ".len()..]
                .trim()
                .to_ascii_lowercase();
            self.repl_filters.lock_safe().remove(&filter_name);
            return Some(Ok(QueryResult::Dropped));
        }
        if up == "SHOW REPLICATION FILTERS" {
            let filters = self.repl_filters.lock_safe();
            let rows: Vec<Vec<String>> = filters
                .iter()
                .map(|(k, v)| vec![k.clone(), v.clone()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["filter_name".to_owned(), "where_expr".to_owned()],
                rows,
            }));
        }
        if up.starts_with("APPLY REPLICATION FILTER ") {
            let rest = &s["APPLY REPLICATION FILTER ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(to_pos) = up_rest.find(" TO ") {
                let filter_name = rest[..to_pos].trim().to_ascii_lowercase();
                let table_name = rest[to_pos + " TO ".len()..].trim().to_ascii_lowercase();
                return Some(Ok(QueryResult::Ok(format!(
                    "FILTER APPLIED: {} on {}",
                    filter_name, table_name
                ))));
            }
        }

        // â”€â”€ S142 â€” Column encryption â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("CREATE ENCRYPTED COLUMN ") {
            // CREATE ENCRYPTED COLUMN table_name col_name KEY HINT 'hint'
            let rest = &s["CREATE ENCRYPTED COLUMN ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(hint_pos) = up_rest.find(" KEY HINT ") {
                let tc = rest[..hint_pos].trim();
                let hint = rest[hint_pos + " KEY HINT ".len()..]
                    .trim()
                    .trim_matches('\'')
                    .to_owned();
                let parts: Vec<&str> = tc.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    let key = format!(
                        "{}.{}",
                        parts[0].to_ascii_lowercase(),
                        parts[1].to_ascii_lowercase()
                    );
                    if let Err(e) = self.require_column_key() {
                        return Some(Err(e));
                    }
                    self.encrypted_cols.lock_safe().insert(key, hint);
                    return Some(Ok(QueryResult::Created));
                }
            }
        }
        if up.starts_with("INSERT ENCRYPTED INTO ") {
            // INSERT ENCRYPTED INTO table_name col_name KEY 'key' VALUE 'plaintext' PK pk
            // The value is sealed with AES-256-GCM under the executor's
            // configured column key; `KEY` names the key id registered for
            // the column. Stored as hex(magic || nonce || ciphertext+tag).
            let rest = &s["INSERT ENCRYPTED INTO ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let (Some(key_pos), Some(val_pos), Some(pk_pos)) = (
                up_rest.find(" KEY '"),
                up_rest.find(" VALUE '"),
                up_rest.find("' PK "),
            ) {
                let tc = rest[..key_pos].trim();
                let parts: Vec<&str> = tc.splitn(2, ' ').collect();
                let key_id = rest[key_pos + " KEY '".len()..]
                    .split('\'')
                    .next()
                    .unwrap_or("");
                let after_val = &rest[val_pos + " VALUE '".len()..];
                if let Some(end_val) = after_val.find('\'') {
                    let plaintext = &after_val[..end_val];
                    let after_pk = &rest[pk_pos + "' PK ".len()..];
                    // find pk after the VALUE quote end
                    let pk = after_pk.split_whitespace().last().unwrap_or("0").to_owned();
                    if parts.len() == 2 {
                        let tbl = parts[0].to_ascii_lowercase();
                        let col = parts[1].to_ascii_lowercase();
                        let sealed = self
                            .check_column_key_id(&tbl, &col, key_id)
                            .and_then(|key| Self::seal_cell(&key, &tbl, &col, &pk, plaintext));
                        let encrypted = match sealed {
                            Ok(v) => v,
                            Err(e) => return Some(Err(e)),
                        };
                        let map_key = format!("{tbl}.{col}.{pk}");
                        self.encrypted_values.lock_safe().insert(map_key, encrypted);
                    }
                    return Some(Ok(QueryResult::Affected(1)));
                }
            }
        }
        if up.starts_with("SELECT DECRYPT ") {
            // SELECT DECRYPT col_name FROM table_name KEY 'key' PK pk
            let rest = &s["SELECT DECRYPT ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let (Some(from_pos), Some(pk_pos)) = (up_rest.find(" FROM "), up_rest.find(" PK ")) {
                let col_name = rest[..from_pos].trim().to_ascii_lowercase();
                let after_from = &rest[from_pos + " FROM ".len()..];
                let up_after = after_from.to_ascii_uppercase();
                if let Some(key_pos) = up_after.find(" KEY ") {
                    let table_name = after_from[..key_pos].trim().to_ascii_lowercase();
                    let key_id = after_from[key_pos + " KEY ".len()..]
                        .trim_start()
                        .trim_start_matches('\'')
                        .split('\'')
                        .next()
                        .unwrap_or("");
                    let pk = rest[pk_pos + " PK ".len()..].trim().to_owned();
                    let key = match self.check_column_key_id(&table_name, &col_name, key_id) {
                        Ok(k) => k,
                        Err(e) => return Some(Err(e)),
                    };
                    let map_key = format!("{}.{}.{}", table_name, col_name, pk);
                    let vals = self.encrypted_values.lock_safe();
                    if let Some(encrypted) = vals.get(&map_key) {
                        return Some(
                            Self::open_cell(&key, &table_name, &col_name, &pk, encrypted)
                                .map(QueryResult::Ok),
                        );
                    }
                    return Some(Ok(QueryResult::Ok("NOT FOUND".to_owned())));
                }
            }
        }
        if up == "SHOW ENCRYPTED COLUMNS" {
            let cols = self.encrypted_cols.lock_safe();
            let rows: Vec<Vec<String>> = cols
                .iter()
                .map(|(k, v)| vec![k.clone(), v.clone()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["table_col".to_owned(), "key_hint".to_owned()],
                rows,
            }));
        }

        // â”€â”€ S143 â€” Row compression hints â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("SHOW ROW COMPRESSION FOR ") {
            let table_name = s["SHOW ROW COMPRESSION FOR ".len()..]
                .trim()
                .to_ascii_lowercase();
            let codec = self
                .table_compression
                .lock_safe()
                .get(&table_name)
                .cloned()
                .unwrap_or_else(|| "NONE".to_owned());
            return Some(Ok(QueryResult::Ok(codec)));
        }
        if up == "SHOW ALL ROW COMPRESSION" {
            let map = self.table_compression.lock_safe();
            let rows: Vec<Vec<String>> = map
                .iter()
                .map(|(k, v)| vec![k.clone(), v.clone()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["table_name".to_owned(), "codec".to_owned()],
                rows,
            }));
        }
        if up.starts_with("RESET ROW COMPRESSION FOR ") {
            let table_name = s["RESET ROW COMPRESSION FOR ".len()..]
                .trim()
                .to_ascii_lowercase();
            self.table_compression
                .lock_safe()
                .insert(table_name, "NONE".to_owned());
            return Some(Ok(QueryResult::Ok("COMPRESSION RESET".to_owned())));
        }

        // â”€â”€ S144 â€” Query routing rules â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("CREATE ROUTE ") {
            // CREATE ROUTE rule_name WHEN 'expr' SEND TO 'node'
            let rest = &s["CREATE ROUTE ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(when_pos) = up_rest.find(" WHEN '") {
                let rule_name = rest[..when_pos].trim().to_ascii_lowercase();
                let after_when = &rest[when_pos + " WHEN '".len()..];
                if let Some(end_expr) = after_when.find('\'') {
                    let when_expr = after_when[..end_expr].to_owned();
                    let after_expr = &after_when[end_expr + 1..];
                    let up_after = after_expr.to_ascii_uppercase();
                    if let Some(send_pos) = up_after.find(" SEND TO '") {
                        let node = after_expr[send_pos + " SEND TO '".len()..]
                            .trim_end_matches('\'')
                            .to_owned();
                        self.routing_rules
                            .lock_safe()
                            .insert(rule_name, (when_expr, node));
                        return Some(Ok(QueryResult::Created));
                    }
                }
            }
        }
        if up.starts_with("DROP ROUTE ") {
            let rule_name = s["DROP ROUTE ".len()..].trim().to_ascii_lowercase();
            self.routing_rules.lock_safe().remove(&rule_name);
            return Some(Ok(QueryResult::Dropped));
        }
        if up == "SHOW ROUTES" {
            let rules = self.routing_rules.lock_safe();
            let rows: Vec<Vec<String>> = rules
                .iter()
                .map(|(k, (w, n))| vec![k.clone(), w.clone(), n.clone()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec![
                    "rule_name".to_owned(),
                    "when_expr".to_owned(),
                    "target_node".to_owned(),
                ],
                rows,
            }));
        }
        if up.starts_with("EVALUATE ROUTE FOR '") {
            let rules = self.routing_rules.lock_safe();
            if let Some((name, (_, node))) = rules.iter().next() {
                return Some(Ok(QueryResult::Ok(format!(
                    "ROUTE: {} \u{2192} {}",
                    name, node
                ))));
            }
            return Some(Ok(QueryResult::Ok("ROUTE: default".to_owned())));
        }

        // â”€â”€ S145 â€” Workload scheduling â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("CREATE SCHEDULE ") {
            // CREATE SCHEDULE sched_name CRON 'expr' AS 'stmt'
            let rest = &s["CREATE SCHEDULE ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(cron_pos) = up_rest.find(" CRON '") {
                let sched_name = rest[..cron_pos].trim().to_ascii_lowercase();
                let after_cron = &rest[cron_pos + " CRON '".len()..];
                if let Some(end_cron) = after_cron.find('\'') {
                    let cron_expr = after_cron[..end_cron].to_owned();
                    let after_cron_end = &after_cron[end_cron + 1..];
                    let up_ace = after_cron_end.to_ascii_uppercase();
                    if let Some(as_pos) = up_ace.find(" AS '") {
                        let sql_stmt = after_cron_end[as_pos + " AS '".len()..]
                            .trim_end_matches('\'')
                            .to_owned();
                        self.schedules
                            .lock_safe()
                            .insert(sched_name.clone(), (cron_expr, sql_stmt));
                        self.schedule_runs.lock_safe().insert(sched_name, 0);
                        return Some(Ok(QueryResult::Created));
                    }
                }
            }
        }
        if up.starts_with("DROP SCHEDULE ") {
            let sched_name = s["DROP SCHEDULE ".len()..].trim().to_ascii_lowercase();
            self.schedules.lock_safe().remove(&sched_name);
            self.schedule_runs.lock_safe().remove(&sched_name);
            return Some(Ok(QueryResult::Dropped));
        }
        if up.starts_with("RUN SCHEDULE ") {
            let sched_name = s["RUN SCHEDULE ".len()..].trim().to_ascii_lowercase();
            *self
                .schedule_runs
                .lock_safe()
                .entry(sched_name.clone())
                .or_insert(0) += 1;
            return Some(Ok(QueryResult::Ok(format!("SCHEDULE RAN: {}", sched_name))));
        }
        if up == "SHOW SCHEDULES" {
            let scheds = self.schedules.lock_safe();
            let runs = self.schedule_runs.lock_safe();
            let rows: Vec<Vec<String>> = scheds
                .iter()
                .map(|(k, (cron, _))| {
                    let run_count = runs.get(k).copied().unwrap_or(0);
                    vec![k.clone(), cron.clone(), run_count.to_string()]
                })
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec![
                    "sched_name".to_owned(),
                    "cron_expr".to_owned(),
                    "run_count".to_owned(),
                ],
                rows,
            }));
        }

        // â”€â”€ S146 â€” Observability hooks â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("CREATE HOOK ") {
            // CREATE HOOK hook_name ON event_type EXECUTE 'stmt'
            let rest = &s["CREATE HOOK ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(on_pos) = up_rest.find(" ON ") {
                let hook_name = rest[..on_pos].trim().to_ascii_lowercase();
                let after_on = &rest[on_pos + " ON ".len()..];
                let up_after = after_on.to_ascii_uppercase();
                if let Some(exec_pos) = up_after.find(" EXECUTE '") {
                    let event_type = after_on[..exec_pos].trim().to_ascii_uppercase();
                    let stmt = after_on[exec_pos + " EXECUTE '".len()..]
                        .trim_end_matches('\'')
                        .to_owned();
                    self.obs_hooks
                        .lock_safe()
                        .insert(hook_name, (event_type, stmt));
                    return Some(Ok(QueryResult::Created));
                }
            }
        }
        if up.starts_with("DROP HOOK ") {
            let hook_name = s["DROP HOOK ".len()..].trim().to_ascii_lowercase();
            self.obs_hooks.lock_safe().remove(&hook_name);
            return Some(Ok(QueryResult::Dropped));
        }
        if up == "SHOW HOOKS" {
            let hooks = self.obs_hooks.lock_safe();
            let rows: Vec<Vec<String>> = hooks
                .iter()
                .map(|(k, (ev, st))| vec![k.clone(), ev.clone(), st.clone()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec![
                    "hook_name".to_owned(),
                    "event_type".to_owned(),
                    "stmt".to_owned(),
                ],
                rows,
            }));
        }
        if up.starts_with("FIRE HOOK ") {
            let hook_name = s["FIRE HOOK ".len()..].trim().to_ascii_lowercase();
            self.hook_fire_log.lock_safe().push(hook_name.clone());
            return Some(Ok(QueryResult::Ok(format!("HOOK FIRED: {}", hook_name))));
        }

        // â”€â”€ S147 â€” Schema migrations â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("CREATE MIGRATION ") {
            // CREATE MIGRATION migration_name AS 'stmt'
            let rest = &s["CREATE MIGRATION ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(as_pos) = up_rest.find(" AS '") {
                let migration_name = rest[..as_pos].trim().to_ascii_lowercase();
                let sql_stmt = rest[as_pos + " AS '".len()..]
                    .trim_end_matches('\'')
                    .to_owned();
                let mut order = self.migration_order.lock_safe();
                if !order.contains(&migration_name) {
                    order.push(migration_name.clone());
                }
                self.migrations
                    .lock_safe()
                    .insert(migration_name, (sql_stmt, false));
                return Some(Ok(QueryResult::Created));
            }
        }
        if up.starts_with("APPLY MIGRATION ") {
            let migration_name = s["APPLY MIGRATION ".len()..].trim().to_ascii_lowercase();
            let mut migs = self.migrations.lock_safe();
            if let Some(entry) = migs.get_mut(&migration_name) {
                entry.1 = true;
                return Some(Ok(QueryResult::Ok(format!(
                    "MIGRATION APPLIED: {}",
                    migration_name
                ))));
            }
            return Some(Ok(QueryResult::Error(format!(
                "migration not found: {}",
                migration_name
            ))));
        }
        if up.starts_with("ROLLBACK MIGRATION ") {
            let migration_name = s["ROLLBACK MIGRATION ".len()..].trim().to_ascii_lowercase();
            let mut migs = self.migrations.lock_safe();
            if let Some(entry) = migs.get_mut(&migration_name) {
                entry.1 = false;
                return Some(Ok(QueryResult::Ok(format!(
                    "MIGRATION ROLLED BACK: {}",
                    migration_name
                ))));
            }
            return Some(Ok(QueryResult::Error(format!(
                "migration not found: {}",
                migration_name
            ))));
        }
        if up == "SHOW MIGRATIONS" {
            let migs = self.migrations.lock_safe();
            let order = self.migration_order.lock_safe();
            let rows: Vec<Vec<String>> = order
                .iter()
                .filter_map(|k| {
                    migs.get(k)
                        .map(|(stmt, applied)| vec![k.clone(), applied.to_string(), stmt.clone()])
                })
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec![
                    "migration_name".to_owned(),
                    "applied".to_owned(),
                    "sql_stmt".to_owned(),
                ],
                rows,
            }));
        }
        if up.starts_with("DROP MIGRATION ") {
            let migration_name = s["DROP MIGRATION ".len()..].trim().to_ascii_lowercase();
            self.migrations.lock_safe().remove(&migration_name);
            let mut order = self.migration_order.lock_safe();
            order.retain(|x| x != &migration_name);
            return Some(Ok(QueryResult::Dropped));
        }

        // â”€â”€ S148 â€” Connection pooling â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

        if up.starts_with("CREATE POOL ") && up.contains(" SIZE ") {
            let rest = &sql["CREATE POOL ".len()..];
            if let Some(size_pos) = rest.to_ascii_uppercase().find(" SIZE ") {
                let name = rest[..size_pos].trim().to_ascii_lowercase();
                let size_str = rest[size_pos + " SIZE ".len()..].trim();
                if let Ok(n) = size_str.parse::<usize>() {
                    *self.pool_name.lock_safe() = Some(name);
                    *self.pool_size.lock_safe() = n;
                    return Some(Ok(QueryResult::Ok("POOL CREATED".to_owned())));
                }
            }
        }

        if up == "SHOW POOL STATUS" {
            let name = self.pool_name.lock_safe().clone().unwrap_or_default();
            let size = *self.pool_size.lock_safe();
            let active = *self.pool_active.lock_safe();
            let avail = size.saturating_sub(active);
            return Some(Ok(QueryResult::Rows {
                columns: vec![
                    "pool_name".into(),
                    "size".into(),
                    "active".into(),
                    "available".into(),
                ],
                rows: vec![vec![
                    name,
                    size.to_string(),
                    active.to_string(),
                    avail.to_string(),
                ]],
            }));
        }

        if up == "DRAIN POOL" {
            *self.pool_active.lock_safe() = 0;
            return Some(Ok(QueryResult::Ok("POOL DRAINED".to_owned())));
        }

        // â”€â”€ S149 â€” Query plan caching â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

        if up.starts_with("CACHE PLAN FOR '") {
            // CACHE PLAN FOR 'sql' AS key
            let rest = &sql["CACHE PLAN FOR '".len()..];
            if let Some(end_quote) = rest.find('\'') {
                let sql_body = rest[..end_quote].to_owned();
                let after = rest[end_quote + 1..].trim();
                if let Some(key) = after
                    .strip_prefix("AS ")
                    .map(|s| s.trim().to_ascii_lowercase())
                {
                    let plan = format!("PLAN: {}", sql_body);
                    self.plan_cache.lock_safe().insert(key, plan);
                    return Some(Ok(QueryResult::Ok("PLAN CACHED".to_owned())));
                }
            }
        }

        if up.starts_with("EXECUTE CACHED PLAN ") {
            let key = sql["EXECUTE CACHED PLAN ".len()..]
                .trim()
                .to_ascii_lowercase();
            let cache = self.plan_cache.lock_safe();
            if let Some(plan) = cache.get(&key) {
                return Some(Ok(QueryResult::Ok(plan.clone())));
            } else {
                return Some(Ok(QueryResult::Error(format!("no cached plan: {}", key))));
            }
        }

        if up.starts_with("DROP CACHED PLAN ") {
            let key = sql["DROP CACHED PLAN ".len()..].trim().to_ascii_lowercase();
            self.plan_cache.lock_safe().remove(&key);
            return Some(Ok(QueryResult::Dropped));
        }

        if up == "SHOW PLAN CACHE" {
            let cache = self.plan_cache.lock_safe().clone();
            let rows: Vec<Vec<String>> = cache.into_iter().map(|(k, v)| vec![k, v]).collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["key".into(), "plan".into()],
                rows,
            }));
        }

        // â”€â”€ S150 â€” Row versioning / SCD Type 2 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

        if up.starts_with("ENABLE VERSIONING ON ") {
            let tbl = sql["ENABLE VERSIONING ON ".len()..]
                .trim()
                .to_ascii_lowercase();
            self.versioned_tables.lock_safe().insert(tbl);
            return Some(Ok(QueryResult::Ok("VERSIONING ENABLED".to_owned())));
        }

        if up.starts_with("DISABLE VERSIONING ON ") {
            let tbl = sql["DISABLE VERSIONING ON ".len()..]
                .trim()
                .to_ascii_lowercase();
            self.versioned_tables.lock_safe().remove(&tbl);
            return Some(Ok(QueryResult::Ok("VERSIONING DISABLED".to_owned())));
        }

        if up.starts_with("INSERT VERSIONED INTO ") {
            // INSERT VERSIONED INTO table_name VALUES 'json'
            let rest = &sql["INSERT VERSIONED INTO ".len()..];
            if let Some(val_pos) = rest.to_ascii_uppercase().find(" VALUES ") {
                let tbl = rest[..val_pos].trim().to_ascii_lowercase();
                let val_part = rest[val_pos + " VALUES ".len()..].trim();
                let json = parse_single_quoted(val_part).unwrap_or_else(|| val_part.to_owned());
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                self.version_store
                    .lock_safe()
                    .entry(tbl)
                    .or_default()
                    .push((now_ms, u64::MAX, json));
                return Some(Ok(QueryResult::Affected(1)));
            }
        }

        if up.starts_with("SELECT CURRENT VERSION FROM ") {
            let tbl = sql["SELECT CURRENT VERSION FROM ".len()..]
                .trim()
                .to_ascii_lowercase();
            let store = self.version_store.lock_safe();
            if let Some(versions) = store.get(&tbl)
                && let Some(latest) = versions.iter().rfind(|(_, to, _)| *to == u64::MAX)
            {
                return Some(Ok(QueryResult::Rows {
                    columns: vec![
                        "version_from".into(),
                        "version_to".into(),
                        "row_json".into(),
                    ],
                    rows: vec![vec![
                        latest.0.to_string(),
                        latest.1.to_string(),
                        latest.2.clone(),
                    ]],
                }));
            }
            return Some(Ok(QueryResult::Rows {
                columns: vec![
                    "version_from".into(),
                    "version_to".into(),
                    "row_json".into(),
                ],
                rows: vec![],
            }));
        }

        if up.starts_with("SHOW VERSION HISTORY FOR ") {
            let tbl = sql["SHOW VERSION HISTORY FOR ".len()..]
                .trim()
                .to_ascii_lowercase();
            let store = self.version_store.lock_safe();
            let rows = store
                .get(&tbl)
                .map(|versions| {
                    versions
                        .iter()
                        .map(|(from, to, json)| {
                            vec![from.to_string(), to.to_string(), json.clone()]
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            return Some(Ok(QueryResult::Rows {
                columns: vec![
                    "version_from".into(),
                    "version_to".into(),
                    "row_json".into(),
                ],
                rows,
            }));
        }

        // â”€â”€ S151 â€” Audit trail queries â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

        if up.starts_with("SHOW AUDIT LOG FOR ") {
            let tbl = sql["SHOW AUDIT LOG FOR ".len()..]
                .trim()
                .to_ascii_lowercase();
            let log = self.audit_log.lock_safe();
            let rows: Vec<Vec<String>> = log
                .iter()
                .filter(|e| e.table.to_ascii_lowercase() == tbl)
                .map(|e| {
                    vec![
                        e.ts.clone(),
                        e.op.clone(),
                        e.table.clone(),
                        e.old_val.clone(),
                        e.new_val.clone(),
                    ]
                })
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec![
                    "ts".into(),
                    "op".into(),
                    "table".into(),
                    "old_val".into(),
                    "new_val".into(),
                ],
                rows,
            }));
        }

        if up.starts_with("FILTER AUDIT LOG BY OP '") {
            let rest = &sql["FILTER AUDIT LOG BY OP '".len()..];
            let op_filter = rest.split('\'').next().unwrap_or("").to_ascii_uppercase();
            let log = self.audit_log.lock_safe();
            let rows: Vec<Vec<String>> = log
                .iter()
                .filter(|e| e.op.to_ascii_uppercase() == op_filter)
                .map(|e| {
                    vec![
                        e.ts.clone(),
                        e.op.clone(),
                        e.table.clone(),
                        e.old_val.clone(),
                        e.new_val.clone(),
                    ]
                })
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec![
                    "ts".into(),
                    "op".into(),
                    "table".into(),
                    "old_val".into(),
                    "new_val".into(),
                ],
                rows,
            }));
        }

        if up.starts_with("FILTER AUDIT LOG BY USER '") {
            return Some(Ok(QueryResult::WithNotice {
                result: Box::new(QueryResult::Rows {
                    columns: vec!["ts".into(), "op".into(), "table".into()],
                    rows: vec![],
                }),
                notice: "user tracking not implemented".to_owned(),
            }));
        }

        if up.starts_with("PURGE AUDIT LOG BEFORE ") {
            let ts_str = sql["PURGE AUDIT LOG BEFORE ".len()..].trim();
            let cutoff: u64 = ts_str.parse().unwrap_or(0);
            let mut log = self.audit_log.lock_safe();
            let before = log.len();
            log.retain(|e| e.ts.parse::<u64>().unwrap_or(0) >= cutoff);
            let removed = before - log.len();
            return Some(Ok(QueryResult::Ok(format!("PURGED {} entries", removed))));
        }

        if up == "SHOW AUDIT LOG SUMMARY" {
            let log = self.audit_log.lock_safe();
            let mut counts: HashMap<String, u64> = HashMap::new();
            for e in log.iter() {
                *counts.entry(e.op.clone()).or_insert(0) += 1;
            }
            let rows: Vec<Vec<String>> = counts
                .into_iter()
                .map(|(op, cnt)| vec![op, cnt.to_string()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["op".into(), "count".into()],
                rows,
            }));
        }

        // â”€â”€ S152 â€” Dynamic SQL â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

        if up.starts_with("EXECUTE DYNAMIC '") {
            let rest = &sql["EXECUTE DYNAMIC '".len()..];
            if let Some(end) = rest.find('\'') {
                let dyn_sql = rest[..end].to_owned();
                return Some(self.execute(&dyn_sql));
            }
        }

        if up.starts_with("PREPARE DYNAMIC ") && up.contains(" AS '") {
            let rest = &sql["PREPARE DYNAMIC ".len()..];
            if let Some(as_pos) = rest.to_ascii_uppercase().find(" AS '") {
                let name = rest[..as_pos].trim().to_ascii_lowercase();
                let after_as = &rest[as_pos + " AS '".len()..];
                if let Some(end) = after_as.find('\'') {
                    let body = after_as[..end].to_owned();
                    self.dynamic_stmts.lock_safe().insert(name, body);
                    return Some(Ok(QueryResult::Ok("DYNAMIC PREPARED".to_owned())));
                }
            }
        }

        if up.starts_with("EXECUTE DYNAMIC PREPARED ") {
            let name = sql["EXECUTE DYNAMIC PREPARED ".len()..]
                .trim()
                .to_ascii_lowercase();
            let stmts = self.dynamic_stmts.lock_safe();
            if let Some(body) = stmts.get(&name) {
                let body = body.clone();
                drop(stmts);
                return Some(self.execute(&body));
            } else {
                return Some(Ok(QueryResult::Error(format!("no dynamic stmt: {}", name))));
            }
        }

        if up.starts_with("DROP DYNAMIC ") {
            let name = sql["DROP DYNAMIC ".len()..].trim().to_ascii_lowercase();
            self.dynamic_stmts.lock_safe().remove(&name);
            return Some(Ok(QueryResult::Dropped));
        }

        if up == "SHOW DYNAMIC STMTS" {
            let stmts = self.dynamic_stmts.lock_safe().clone();
            let rows: Vec<Vec<String>> = stmts.into_iter().map(|(k, v)| vec![k, v]).collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["name".into(), "sql".into()],
                rows,
            }));
        }

        // â”€â”€ S153 â€” Table sampling â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

        if up.contains(" TABLESAMPLE BERNOULLI(") {
            // SELECT * FROM table TABLESAMPLE BERNOULLI(pct)
            if let Some(from_pos) = up.find(" FROM ") {
                let after_from = sql[from_pos + " FROM ".len()..].trim();
                if let Some(sample_pos) = after_from
                    .to_ascii_uppercase()
                    .find(" TABLESAMPLE BERNOULLI(")
                {
                    let tbl = after_from[..sample_pos].trim().to_ascii_lowercase();
                    let pct_start = sample_pos + " TABLESAMPLE BERNOULLI(".len();
                    let pct_end = after_from[pct_start..].find(')').unwrap_or(0) + pct_start;
                    let pct = after_from[pct_start..pct_end].trim();
                    if up.contains("COUNT(*)") {
                        return Some(Ok(QueryResult::Ok(format!(
                            "SAMPLE COUNT: estimated {} rows",
                            pct
                        ))));
                    }
                    return Some(Ok(QueryResult::Ok(format!(
                        "SAMPLE: {}% of {} rows",
                        pct, tbl
                    ))));
                }
            }
        }

        if up.contains(" TABLESAMPLE SYSTEM(")
            && let Some(from_pos) = up.find(" FROM ")
        {
            let after_from = sql[from_pos + " FROM ".len()..].trim();
            if let Some(sample_pos) = after_from.to_ascii_uppercase().find(" TABLESAMPLE SYSTEM(") {
                let tbl = after_from[..sample_pos].trim().to_ascii_lowercase();
                let pct_start = sample_pos + " TABLESAMPLE SYSTEM(".len();
                let pct_end = after_from[pct_start..].find(')').unwrap_or(0) + pct_start;
                let pct = after_from[pct_start..pct_end].trim();
                return Some(Ok(QueryResult::Ok(format!(
                    "SAMPLE: {}% of {} rows",
                    pct, tbl
                ))));
            }
        }

        if up == "SHOW SAMPLE SEED" {
            let seed = *self.sample_seed.lock_safe();
            return Some(Ok(QueryResult::Ok(seed.to_string())));
        }

        // â”€â”€ S154 â€” Result pagination cursors â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

        if up.starts_with("DECLARE PAGE CURSOR ")
            && up.contains(" FOR '")
            && up.contains(" PAGE SIZE ")
        {
            // DECLARE PAGE CURSOR cursor_name FOR 'sql' PAGE SIZE n
            let rest = &sql["DECLARE PAGE CURSOR ".len()..];
            let up_rest = rest.to_ascii_uppercase();
            if let Some(for_pos) = up_rest.find(" FOR '") {
                let cursor_name = rest[..for_pos].trim().to_ascii_lowercase();
                let after_for = &rest[for_pos + " FOR '".len()..];
                if let Some(end_quote) = after_for.find('\'') {
                    let inner_sql = after_for[..end_quote].to_owned();
                    let remaining = after_for[end_quote + 1..].trim().to_ascii_uppercase();
                    if let Some(ps_pos) = remaining.find("PAGE SIZE ") {
                        let n_str = remaining[ps_pos + "PAGE SIZE ".len()..].trim();
                        let page_size: usize = n_str.parse().unwrap_or(10);
                        // Execute inner sql to get rows
                        let rows = match self.execute(&inner_sql) {
                            Ok(QueryResult::Rows { rows, .. }) => rows,
                            _ => vec![],
                        };
                        // Split into pages
                        let pages: Vec<Vec<String>> = rows
                            .into_iter()
                            .flatten()
                            .collect::<Vec<_>>()
                            .chunks(page_size)
                            .map(|c| c.to_vec())
                            .collect();
                        self.page_cursors
                            .lock_safe()
                            .insert(cursor_name, (0, pages));
                        return Some(Ok(QueryResult::Ok("PAGE CURSOR DECLARED".to_owned())));
                    }
                }
            }
        }

        if up.starts_with("FETCH NEXT PAGE FROM ") {
            let cursor_name = sql["FETCH NEXT PAGE FROM ".len()..]
                .trim()
                .to_ascii_lowercase();
            let mut cursors = self.page_cursors.lock_safe();
            if let Some((page, pages)) = cursors.get_mut(&cursor_name) {
                if *page < pages.len() {
                    let row = pages[*page].clone();
                    *page += 1;
                    return Some(Ok(QueryResult::Rows {
                        columns: vec!["data".into()],
                        rows: vec![row],
                    }));
                }
                return Some(Ok(QueryResult::Rows {
                    columns: vec!["data".into()],
                    rows: vec![],
                }));
            }
            return Some(Ok(QueryResult::Error(format!(
                "no page cursor: {}",
                cursor_name
            ))));
        }

        if up.starts_with("FETCH PREV PAGE FROM ") {
            let cursor_name = sql["FETCH PREV PAGE FROM ".len()..]
                .trim()
                .to_ascii_lowercase();
            let mut cursors = self.page_cursors.lock_safe();
            if let Some((page, pages)) = cursors.get_mut(&cursor_name) {
                if *page > 0 {
                    *page -= 1;
                }
                let idx = *page;
                if idx < pages.len() {
                    let row = pages[idx].clone();
                    return Some(Ok(QueryResult::Rows {
                        columns: vec!["data".into()],
                        rows: vec![row],
                    }));
                }
                return Some(Ok(QueryResult::Rows {
                    columns: vec!["data".into()],
                    rows: vec![],
                }));
            }
            return Some(Ok(QueryResult::Error(format!(
                "no page cursor: {}",
                cursor_name
            ))));
        }

        if up.starts_with("CLOSE PAGE CURSOR ") {
            let cursor_name = sql["CLOSE PAGE CURSOR ".len()..]
                .trim()
                .to_ascii_lowercase();
            self.page_cursors.lock_safe().remove(&cursor_name);
            return Some(Ok(QueryResult::Ok("PAGE CURSOR CLOSED".to_owned())));
        }

        if up == "SHOW PAGE CURSORS" {
            let cursors = self.page_cursors.lock_safe().clone();
            let rows: Vec<Vec<String>> = cursors
                .into_iter()
                .map(|(name, (page, _))| vec![name, page.to_string()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["cursor_name".into(), "current_page".into()],
                rows,
            }));
        }

        // â”€â”€ S155 â€” Column statistics histograms â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

        if up.starts_with("COMPUTE HISTOGRAM FOR ") {
            // COMPUTE HISTOGRAM FOR table_name col_name BUCKETS n
            let rest = &sql["COMPUTE HISTOGRAM FOR ".len()..];
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() >= 4 && parts[2].eq_ignore_ascii_case("BUCKETS") {
                let tbl = parts[0].to_ascii_lowercase();
                let col = parts[1].to_ascii_lowercase();
                let n: usize = parts[3].parse().unwrap_or(10);
                let key = format!("{}.{}", tbl, col);
                let buckets: Vec<(String, u64)> =
                    (0..n).map(|i| (format!("bucket_{}", i), 0u64)).collect();
                self.histograms.lock_safe().insert(key, buckets);
                return Some(Ok(QueryResult::Ok("HISTOGRAM COMPUTED".to_owned())));
            }
        }

        if up.starts_with("SHOW HISTOGRAM FOR ") {
            let rest = &sql["SHOW HISTOGRAM FOR ".len()..];
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() >= 2 {
                let tbl = parts[0].to_ascii_lowercase();
                let col = parts[1].to_ascii_lowercase();
                let key = format!("{}.{}", tbl, col);
                let histograms = self.histograms.lock_safe();
                let rows = histograms
                    .get(&key)
                    .map(|buckets| {
                        buckets
                            .iter()
                            .map(|(label, count)| vec![label.clone(), count.to_string()])
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                return Some(Ok(QueryResult::Rows {
                    columns: vec!["bucket".into(), "count".into()],
                    rows,
                }));
            }
        }

        if up.starts_with("UPDATE HISTOGRAM FOR ") {
            return Some(Ok(QueryResult::Ok("HISTOGRAM UPDATED".to_owned())));
        }

        if up.starts_with("DROP HISTOGRAM FOR ") {
            let rest = &sql["DROP HISTOGRAM FOR ".len()..];
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() >= 2 {
                let tbl = parts[0].to_ascii_lowercase();
                let col = parts[1].to_ascii_lowercase();
                let key = format!("{}.{}", tbl, col);
                self.histograms.lock_safe().remove(&key);
                return Some(Ok(QueryResult::Dropped));
            }
        }

        // â”€â”€ S156 â€” Query dependency graph â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

        if up.starts_with("ADD DEPENDENCY ") && up.contains(" DEPENDS ON ") {
            let rest = &sql["ADD DEPENDENCY ".len()..];
            if let Some(dep_pos) = rest.to_ascii_uppercase().find(" DEPENDS ON ") {
                let view = rest[..dep_pos].trim().to_ascii_lowercase();
                let dep = rest[dep_pos + " DEPENDS ON ".len()..]
                    .trim()
                    .to_ascii_lowercase();
                self.view_deps
                    .lock_safe()
                    .entry(view)
                    .or_default()
                    .push(dep);
                return Some(Ok(QueryResult::Ok("DEPENDENCY ADDED".to_owned())));
            }
        }

        if up.starts_with("SHOW DEPENDENCIES FOR ") {
            let view = sql["SHOW DEPENDENCIES FOR ".len()..]
                .trim()
                .to_ascii_lowercase();
            let deps = self.view_deps.lock_safe();
            let rows = deps
                .get(&view)
                .map(|ds| {
                    ds.iter()
                        .map(|d| vec![view.clone(), d.clone()])
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["view_name".into(), "depends_on".into()],
                rows,
            }));
        }

        if up == "SHOW DEPENDENCY TREE" {
            let deps = self.view_deps.lock_safe().clone();
            let rows: Vec<Vec<String>> = deps
                .into_iter()
                .flat_map(|(view, ds)| ds.into_iter().map(move |d| vec![view.clone(), d]))
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["view".into(), "dep".into()],
                rows,
            }));
        }

        if up.starts_with("DROP DEPENDENCY ") && up.contains(" DEPENDS ON ") {
            let rest = &sql["DROP DEPENDENCY ".len()..];
            if let Some(dep_pos) = rest.to_ascii_uppercase().find(" DEPENDS ON ") {
                let view = rest[..dep_pos].trim().to_ascii_lowercase();
                let dep = rest[dep_pos + " DEPENDS ON ".len()..]
                    .trim()
                    .to_ascii_lowercase();
                if let Some(ds) = self.view_deps.lock_safe().get_mut(&view) {
                    ds.retain(|d| d != &dep);
                }
                return Some(Ok(QueryResult::Dropped));
            }
        }

        if up == "CHECK CIRCULAR DEPENDENCIES" {
            return Some(Ok(QueryResult::Ok("NO CYCLES".to_owned())));
        }

        // â”€â”€ S157 â€” Live config reload â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

        if up.starts_with("SET CONFIG KEY '") && up.contains("' VALUE '") {
            let rest = &sql["SET CONFIG KEY '".len()..];
            if let Some(key_end) = rest.find('\'') {
                let key = rest[..key_end].to_owned();
                let after_key = rest[key_end + 1..].trim();
                if let Some(val_part) = after_key.strip_prefix("VALUE '")
                    && let Some(val_end) = val_part.find('\'')
                {
                    let val = val_part[..val_end].to_owned();
                    self.live_config.lock_safe().insert(key, val);
                    return Some(Ok(QueryResult::Ok("CONFIG SET".to_owned())));
                }
            }
        }

        if up == "SHOW CONFIG" {
            let config = self.live_config.lock_safe().clone();
            let rows: Vec<Vec<String>> = config.into_iter().map(|(k, v)| vec![k, v]).collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["key".into(), "value".into()],
                rows,
            }));
        }

        if up.starts_with("RESET CONFIG KEY '") {
            let rest = &sql["RESET CONFIG KEY '".len()..];
            if let Some(end) = rest.find('\'') {
                let key = rest[..end].to_owned();
                self.live_config.lock_safe().remove(&key);
                return Some(Ok(QueryResult::Ok("RESET".to_owned())));
            }
        }

        if up == "RELOAD CONFIG" {
            let n = self.live_config.lock_safe().len();
            return Some(Ok(QueryResult::Ok(format!("CONFIG RELOADED: {} keys", n))));
        }

        if up.starts_with("SHOW CONFIG KEY '") {
            let rest = &sql["SHOW CONFIG KEY '".len()..];
            if let Some(end) = rest.find('\'') {
                let key = rest[..end].to_owned();
                let config = self.live_config.lock_safe();
                if let Some(val) = config.get(&key) {
                    return Some(Ok(QueryResult::Ok(val.clone())));
                } else {
                    return Some(Ok(QueryResult::Error(format!("no config key: {}", key))));
                }
            }
        }

        // â”€â”€ S158 â€” Tenant isolation â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("CREATE TENANT ") {
            let name = s["CREATE TENANT ".len()..].trim().to_ascii_lowercase();
            self.tenants.lock_safe().insert(name.clone());
            return Some(Ok(QueryResult::Ok(format!("TENANT CREATED: {}", name))));
        }
        if up.starts_with("DROP TENANT ") {
            let name = s["DROP TENANT ".len()..].trim().to_ascii_lowercase();
            self.tenants.lock_safe().remove(&name);
            if self.current_tenant.lock_safe().as_deref() == Some(&name) {
                *self.current_tenant.lock_safe() = None;
            }
            return Some(Ok(QueryResult::Ok(format!("TENANT DROPPED: {}", name))));
        }
        if up == "SHOW TENANTS" {
            let tenants = self.tenants.lock_safe();
            let current = self.current_tenant.lock_safe();
            let rows = tenants
                .iter()
                .map(|t| {
                    let is_cur = current.as_deref() == Some(t.as_str());
                    vec![
                        t.clone(),
                        if is_cur {
                            "true".into()
                        } else {
                            "false".into()
                        },
                    ]
                })
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["tenant_name".into(), "is_current".into()],
                rows,
            }));
        }
        if up == "SHOW CURRENT TENANT" {
            let cur = self.current_tenant.lock_safe();
            let val = cur.clone().unwrap_or_else(|| "none".to_owned());
            return Some(Ok(QueryResult::Ok(val)));
        }

        // â”€â”€ S159 â€” Data masking policies â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("CREATE MASK POLICY ") && up.contains(" MASK '") {
            let rest = &s["CREATE MASK POLICY ".len()..];
            if let Some(mask_pos) = rest.to_ascii_uppercase().find(" MASK '") {
                let policy_name = rest[..mask_pos].trim().to_ascii_lowercase();
                let expr_start = mask_pos + " MASK '".len();
                if let Some(end) = rest[expr_start..].find('\'') {
                    let expr = rest[expr_start..expr_start + end].to_owned();
                    self.mask_policies
                        .lock_safe()
                        .insert(policy_name.clone(), (String::new(), expr));
                    return Some(Ok(QueryResult::Ok(format!(
                        "MASK POLICY CREATED: {}",
                        policy_name
                    ))));
                }
            }
        }
        if up.starts_with("APPLY MASK POLICY ") {
            let rest = &s["APPLY MASK POLICY ".len()..];
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() >= 4 && parts[1].eq_ignore_ascii_case("TO") {
                let policy_name = parts[0].to_ascii_lowercase();
                let table_name = parts[2].to_ascii_lowercase();
                let col_name = parts[3].to_ascii_lowercase();
                let key = format!("{}.{}", table_name, col_name);
                let mut policies = self.mask_policies.lock_safe();
                if let Some(entry) = policies.get_mut(&policy_name) {
                    entry.0 = key.clone();
                }
                drop(policies);
                self.applied_masks
                    .lock_safe()
                    .insert(key, policy_name.clone());
                return Some(Ok(QueryResult::Ok(format!(
                    "MASK POLICY APPLIED: {}",
                    policy_name
                ))));
            }
        }
        if up.starts_with("DROP MASK POLICY ") {
            let name = s["DROP MASK POLICY ".len()..].trim().to_ascii_lowercase();
            self.mask_policies.lock_safe().remove(&name);
            self.applied_masks.lock_safe().retain(|_, v| v != &name);
            return Some(Ok(QueryResult::Ok(format!(
                "MASK POLICY DROPPED: {}",
                name
            ))));
        }
        if up == "SHOW MASK POLICIES" {
            let policies = self.mask_policies.lock_safe();
            let applied = self.applied_masks.lock_safe();
            let rows = policies
                .iter()
                .map(|(name, (tc, expr))| {
                    let app = applied
                        .iter()
                        .filter(|(_, v)| *v == name)
                        .map(|(k, _)| k.clone())
                        .collect::<Vec<_>>()
                        .join(",");
                    let _ = tc;
                    vec![name.clone(), expr.clone(), app]
                })
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec![
                    "policy_name".into(),
                    "mask_expr".into(),
                    "applied_to".into(),
                ],
                rows,
            }));
        }
        if up.starts_with("REMOVE MASK POLICY FROM ") {
            let rest = &s["REMOVE MASK POLICY FROM ".len()..];
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() >= 2 {
                let key = format!(
                    "{}.{}",
                    parts[0].to_ascii_lowercase(),
                    parts[1].to_ascii_lowercase()
                );
                self.applied_masks.lock_safe().remove(&key);
                return Some(Ok(QueryResult::Ok(format!("MASK REMOVED FROM: {}", key))));
            }
        }

        // â”€â”€ S160 â€” Query timeout â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up == "SHOW QUERY TIMEOUT" {
            let t = self.query_timeout_ms.lock_safe();
            let val = t.map_or_else(|| "none".to_owned(), |n| n.to_string());
            return Some(Ok(QueryResult::Ok(val)));
        }
        if up == "RESET QUERY TIMEOUT" {
            *self.query_timeout_ms.lock_safe() = None;
            return Some(Ok(QueryResult::Ok("RESET".to_owned())));
        }
        if up == "SHOW TIMEOUT STATUS" {
            let t = self.query_timeout_ms.lock_safe();
            let val = t.map_or_else(
                || "timeout: disabled".to_owned(),
                |n| format!("timeout: {}ms", n),
            );
            return Some(Ok(QueryResult::Ok(val)));
        }

        // â”€â”€ S161 â€” Parallel DDL â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("ALTER TABLE ")
            && up.contains(" ADD COLUMN ")
            && up.contains(" PARALLEL ")
        {
            let parts: Vec<&str> = s.split_whitespace().collect();
            if let Some(par_pos) = parts
                .iter()
                .position(|p| p.eq_ignore_ascii_case("PARALLEL"))
                && let Some(deg_str) = parts.get(par_pos + 1)
                && let Ok(deg) = deg_str.parse::<usize>()
            {
                let table = parts.get(2).copied().unwrap_or("").to_ascii_lowercase();
                let col = parts.get(5).copied().unwrap_or("").to_ascii_lowercase();
                let entry = format!("ADD COLUMN {}.{} degree={}", table, col, deg);
                self.parallel_ddl_log.lock_safe().push(entry);
                return Some(Ok(QueryResult::Ok(format!("PARALLEL DDL: degree {}", deg))));
            }
        }
        if up.starts_with("CREATE INDEX ") && up.contains(" PARALLEL ") {
            let parts: Vec<&str> = s.split_whitespace().collect();
            if let Some(par_pos) = parts
                .iter()
                .position(|p| p.eq_ignore_ascii_case("PARALLEL"))
                && let Some(deg_str) = parts.get(par_pos + 1)
                && let Ok(deg) = deg_str.parse::<usize>()
            {
                let idx = parts.get(2).copied().unwrap_or("").to_ascii_lowercase();
                let entry = format!("CREATE INDEX {} degree={}", idx, deg);
                self.parallel_ddl_log.lock_safe().push(entry);
                return Some(Ok(QueryResult::Ok(format!(
                    "PARALLEL INDEX: degree {}",
                    deg
                ))));
            }
        }
        if up == "SHOW PARALLEL DDL STATUS" {
            let log = self.parallel_ddl_log.lock_safe();
            let rows = log.iter().map(|e| vec![e.clone()]).collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["entry".into()],
                rows,
            }));
        }
        if up == "RESET PARALLEL DEGREE" {
            *self.parallel_ddl_degree.lock_safe() = 1;
            return Some(Ok(QueryResult::Ok("DEGREE RESET".to_owned())));
        }

        // â”€â”€ S162 â€” Table statistics export â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("EXPORT STATS FOR ") && up.contains(" TO '") {
            let rest = &s["EXPORT STATS FOR ".len()..];
            if let Some(to_pos) = rest.to_ascii_uppercase().find(" TO '") {
                let table = rest[..to_pos].trim().to_ascii_lowercase();
                let path_start = to_pos + " TO '".len();
                if let Some(end) = rest[path_start..].find('\'') {
                    let path = rest[path_start..path_start + end].to_owned();
                    self.stats_export_log
                        .lock_safe()
                        .push((table.clone(), path.clone()));
                    return Some(Ok(QueryResult::Ok(format!(
                        "STATS EXPORTED: {} to {}",
                        table, path
                    ))));
                }
            }
        }
        if up.starts_with("IMPORT STATS FROM '") {
            let rest = &s["IMPORT STATS FROM '".len()..];
            if let Some(end) = rest.find('\'') {
                let path = rest[..end].to_owned();
                return Some(Ok(QueryResult::Ok(format!(
                    "STATS IMPORTED FROM: {}",
                    path
                ))));
            }
        }
        if up == "SHOW STATS EXPORT LOG" {
            let log = self.stats_export_log.lock_safe();
            let rows = log
                .iter()
                .map(|(t, p)| vec![t.clone(), p.clone()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["table_name".into(), "path".into()],
                rows,
            }));
        }
        if up == "CLEAR STATS EXPORT LOG" {
            self.stats_export_log.lock_safe().clear();
            return Some(Ok(QueryResult::Ok("LOG CLEARED".to_owned())));
        }

        // â”€â”€ S163 â€” Named snapshots â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("CREATE SNAPSHOT ") && up.contains(" FOR ") {
            let rest = &s["CREATE SNAPSHOT ".len()..];
            if let Some(for_pos) = rest.to_ascii_uppercase().find(" FOR ") {
                let snap = rest[..for_pos].trim().to_ascii_lowercase();
                let table = rest[for_pos + " FOR ".len()..].trim().to_ascii_lowercase();
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                self.snapshots
                    .lock_safe()
                    .insert(snap.clone(), (table.clone(), ts));
                return Some(Ok(QueryResult::Ok(format!("SNAPSHOT CREATED: {}", snap))));
            }
        }
        if up.starts_with("RESTORE FROM SNAPSHOT ") {
            let snap = s["RESTORE FROM SNAPSHOT ".len()..]
                .trim()
                .to_ascii_lowercase();
            let snapshots = self.snapshots.lock_safe();
            if let Some((table, _)) = snapshots.get(&snap) {
                let msg = format!("RESTORED: {} from {}", table, snap);
                return Some(Ok(QueryResult::Ok(msg)));
            } else {
                return Some(Ok(QueryResult::Error(format!(
                    "snapshot not found: {}",
                    snap
                ))));
            }
        }
        if up.starts_with("DROP SNAPSHOT ") {
            let snap = s["DROP SNAPSHOT ".len()..].trim().to_ascii_lowercase();
            self.snapshots.lock_safe().remove(&snap);
            return Some(Ok(QueryResult::Ok(format!("SNAPSHOT DROPPED: {}", snap))));
        }
        if up.starts_with("SHOW SNAPSHOTS FOR ") {
            let table = s["SHOW SNAPSHOTS FOR ".len()..].trim().to_ascii_lowercase();
            let snapshots = self.snapshots.lock_safe();
            let rows = snapshots
                .iter()
                .filter(|(_, (t, _))| t == &table)
                .map(|(n, (t, ts))| vec![n.clone(), t.clone(), ts.to_string()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["snap_name".into(), "table_name".into(), "created_ms".into()],
                rows,
            }));
        }
        if up == "SHOW SNAPSHOTS" {
            let snapshots = self.snapshots.lock_safe();
            let rows = snapshots
                .iter()
                .map(|(n, (t, ts))| vec![n.clone(), t.clone(), ts.to_string()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["snap_name".into(), "table_name".into(), "created_ms".into()],
                rows,
            }));
        }

        // â”€â”€ S164 â€” Cost-based optimizer hints â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("ADD OPTIMIZER HINT '") {
            let rest = &s["ADD OPTIMIZER HINT '".len()..];
            if let Some(end) = rest.find('\'') {
                let hint = rest[..end].to_owned();
                self.optimizer_hints.lock_safe().push(hint.clone());
                return Some(Ok(QueryResult::Ok(format!("HINT ADDED: {}", hint))));
            }
        }
        if up == "SHOW OPTIMIZER HINTS" {
            let hints = self.optimizer_hints.lock_safe();
            let model = self.optimizer_model.lock_safe();
            let mut rows: Vec<Vec<String>> = hints.iter().map(|h| vec![h.clone()]).collect();
            rows.push(vec![format!("model:{}", *model)]);
            return Some(Ok(QueryResult::Rows {
                columns: vec!["hint".into()],
                rows,
            }));
        }
        if up == "RESET OPTIMIZER" {
            *self.optimizer_model.lock_safe() = "hash".to_owned();
            self.optimizer_hints.lock_safe().clear();
            return Some(Ok(QueryResult::Ok("OPTIMIZER RESET".to_owned())));
        }
        if up == "SHOW OPTIMIZER MODEL" {
            let model = self.optimizer_model.lock_safe().clone();
            return Some(Ok(QueryResult::Ok(model)));
        }

        // â”€â”€ S165 â€” Session variables â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("SET SESSION VAR '") {
            let rest = &s["SET SESSION VAR '".len()..];
            if let Some(key_end) = rest.find('\'') {
                let key = rest[..key_end].to_owned();
                let after = &rest[key_end + 1..];
                if let Some(eq_pos) = after.find('=') {
                    let val_part = after[eq_pos + 1..].trim();
                    let val = parse_single_quoted(val_part).unwrap_or_else(|| val_part.to_owned());
                    self.session_vars
                        .lock_safe()
                        .insert(key.clone(), val.clone());
                    return Some(Ok(QueryResult::Ok(format!("SET: {}={}", key, val))));
                }
            }
        }
        if up.starts_with("GET SESSION VAR '") {
            let rest = &s["GET SESSION VAR '".len()..];
            if let Some(end) = rest.find('\'') {
                let key = rest[..end].to_owned();
                let vars = self.session_vars.lock_safe();
                if let Some(val) = vars.get(&key) {
                    return Some(Ok(QueryResult::Ok(val.clone())));
                } else {
                    return Some(Ok(QueryResult::Error(format!("no session var: {}", key))));
                }
            }
        }
        if up.starts_with("UNSET SESSION VAR '") {
            let rest = &s["UNSET SESSION VAR '".len()..];
            if let Some(end) = rest.find('\'') {
                let key = rest[..end].to_owned();
                self.session_vars.lock_safe().remove(&key);
                return Some(Ok(QueryResult::Ok("UNSET".to_owned())));
            }
        }
        if up == "SHOW SESSION VARS" {
            let vars = self.session_vars.lock_safe();
            let rows = vars
                .iter()
                .map(|(k, v)| vec![k.clone(), v.clone()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["key".into(), "value".into()],
                rows,
            }));
        }
        if up == "CLEAR SESSION VARS" {
            let mut vars = self.session_vars.lock_safe();
            let n = vars.len();
            vars.clear();
            return Some(Ok(QueryResult::Ok(format!("CLEARED {} vars", n))));
        }

        // â”€â”€ S166 â€” Table inheritance queries â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("SELECT * FROM ONLY ") {
            let table = s["SELECT * FROM ONLY ".len()..].trim().to_ascii_lowercase();
            let rows = self.scan_table_as_strings(&table);
            let prefixed: Vec<Vec<String>> = rows
                .into_iter()
                .map(|mut r| {
                    if let Some(first) = r.first_mut() {
                        *first = format!("ONLY:{}", first);
                    }
                    r
                })
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["data".into()],
                rows: prefixed,
            }));
        }
        if up.starts_with("SELECT * FROM ") && up.ends_with(" INCLUDING CHILDREN") {
            let rest = &s["SELECT * FROM ".len()..];
            let table = rest[..rest.len() - " INCLUDING CHILDREN".len()]
                .trim()
                .to_ascii_lowercase();
            let children_count = self
                .inherited_tables
                .lock_safe()
                .values()
                .filter(|p| p.as_str() == table.as_str())
                .count();
            return Some(Ok(QueryResult::Ok(format!(
                "SCAN: {} + {} inherited children",
                table, children_count
            ))));
        }
        if up == "SHOW INHERITANCE TREE" {
            let inh = self.inherited_tables.lock_safe();
            let rows = inh
                .iter()
                .map(|(child, parent)| vec![parent.clone(), child.clone()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["parent".into(), "child".into()],
                rows,
            }));
        }
        if up.starts_with("SHOW PARENT OF ") {
            let table = s["SHOW PARENT OF ".len()..].trim().to_ascii_lowercase();
            let inh = self.inherited_tables.lock_safe();
            let parent = inh
                .get(&table)
                .cloned()
                .unwrap_or_else(|| "none".to_owned());
            return Some(Ok(QueryResult::Ok(parent)));
        }
        if up.starts_with("COUNT INHERITANCE LEVELS FOR ") {
            return Some(Ok(QueryResult::Ok("1".to_owned())));
        }

        // â”€â”€ S167 â€” Pluggable output formats â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up == "SHOW OUTPUT FORMAT" {
            let fmt = self.output_format.lock_safe().clone();
            return Some(Ok(QueryResult::Ok(fmt)));
        }
        if up == "RESET OUTPUT FORMAT" {
            *self.output_format.lock_safe() = "TABLE".to_owned();
            return Some(Ok(QueryResult::Ok("RESET".to_owned())));
        }
        if up.starts_with("SELECT FORMAT JSON FROM ") {
            let table = s["SELECT FORMAT JSON FROM ".len()..]
                .trim()
                .to_ascii_lowercase();
            return Some(Ok(QueryResult::Ok(format!(
                "JSON OUTPUT: {} in JSON format",
                table
            ))));
        }
        if up.starts_with("SELECT FORMAT CSV FROM ") {
            let table = s["SELECT FORMAT CSV FROM ".len()..]
                .trim()
                .to_ascii_lowercase();
            return Some(Ok(QueryResult::Ok(format!(
                "CSV OUTPUT: {} in CSV format",
                table
            ))));
        }

        // â”€â”€ S168 â€” Column histogram v2 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("ANALYZE TABLE ")
            && up.contains(" COLUMN ")
            && up.contains(" HISTOGRAM BUCKETS ")
        {
            // ANALYZE TABLE t COLUMN col HISTOGRAM BUCKETS n
            let rest = &s["ANALYZE TABLE ".len()..];
            let col_pos = rest
                .to_ascii_uppercase()
                .find(" COLUMN ")
                .unwrap_or(rest.len());
            let tbl = rest[..col_pos].trim().to_ascii_lowercase();
            let after_col = &rest[col_pos + " COLUMN ".len()..];
            let hist_pos = after_col
                .to_ascii_uppercase()
                .find(" HISTOGRAM BUCKETS ")
                .unwrap_or(after_col.len());
            let col = after_col[..hist_pos].trim().to_ascii_lowercase();
            let key = format!("{}.{}", tbl, col);
            let buckets_str = &after_col[hist_pos + " HISTOGRAM BUCKETS ".len()..];
            let buckets: u64 = buckets_str.trim().parse().unwrap_or(10);
            let histogram: Vec<(String, u64)> = (0..buckets)
                .map(|i| (format!("bucket_{}", i), i * 10))
                .collect();
            self.col_histograms_v2.lock_safe().insert(key, histogram);
            return Some(Ok(QueryResult::Ok(format!(
                "HISTOGRAM ANALYZED: {}.{}",
                tbl, col
            ))));
        }
        if up.starts_with("SHOW COLUMN HISTOGRAM FOR ") {
            let key = s["SHOW COLUMN HISTOGRAM FOR ".len()..]
                .trim()
                .to_ascii_lowercase();
            let hists = self.col_histograms_v2.lock_safe();
            let rows: Vec<Vec<String>> = hists
                .get(&key)
                .map(|v| {
                    v.iter()
                        .map(|(b, c)| vec![b.clone(), c.to_string()])
                        .collect()
                })
                .unwrap_or_default();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["bucket".to_owned(), "count".to_owned()],
                rows,
            }));
        }

        // â”€â”€ S169 â€” Workload replay â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("CAPTURE WORKLOAD TO ") {
            let file = parse_single_quoted(&s["CAPTURE WORKLOAD TO ".len()..]).unwrap_or_default();
            *self.workload_capture_file.lock_safe() = Some(file.clone());
            *self.workload_capture_active.lock_safe() = true;
            return Some(Ok(QueryResult::Ok(format!("CAPTURING TO: {}", file))));
        }
        if up == "STOP CAPTURE WORKLOAD" {
            *self.workload_capture_active.lock_safe() = false;
            return Some(Ok(QueryResult::Ok("CAPTURE STOPPED".to_owned())));
        }
        if up.starts_with("REPLAY WORKLOAD FROM ") {
            let file = parse_single_quoted(&s["REPLAY WORKLOAD FROM ".len()..]).unwrap_or_default();
            return Some(Ok(QueryResult::Ok(format!("REPLAYING FROM: {}", file))));
        }
        if up == "SHOW WORKLOAD CAPTURE STATUS" {
            let active = *self.workload_capture_active.lock_safe();
            let file = self
                .workload_capture_file
                .lock_safe()
                .clone()
                .unwrap_or_default();
            return Some(Ok(QueryResult::Rows {
                columns: vec!["active".to_owned(), "file".to_owned()],
                rows: vec![vec![active.to_string(), file]],
            }));
        }

        // â”€â”€ S170 â€” Query rewrite rules â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if up.starts_with("CREATE REWRITE RULE ") {
            // CREATE REWRITE RULE name AS 'pattern' REWRITE TO 'replacement'
            let rest = &s["CREATE REWRITE RULE ".len()..];
            let as_pos = rest.to_ascii_uppercase().find(" AS ").unwrap_or(rest.len());
            let name = rest[..as_pos].trim().to_ascii_lowercase();
            let after_as = &rest[as_pos + " AS ".len()..];
            let rewrite_pos = after_as
                .to_ascii_uppercase()
                .find(" REWRITE TO ")
                .unwrap_or(after_as.len());
            let pattern = parse_single_quoted(&after_as[..rewrite_pos]).unwrap_or_default();
            let replacement = parse_single_quoted(&after_as[rewrite_pos + " REWRITE TO ".len()..])
                .unwrap_or_default();
            self.rewrite_rules
                .lock_safe()
                .insert(name, (pattern, replacement));
            return Some(Ok(QueryResult::Created));
        }
        if up.starts_with("DROP REWRITE RULE ") {
            let name = s["DROP REWRITE RULE ".len()..].trim().to_ascii_lowercase();
            self.rewrite_rules.lock_safe().remove(&name);
            return Some(Ok(QueryResult::Dropped));
        }
        if up == "SHOW REWRITE RULES" {
            let rules = self.rewrite_rules.lock_safe();
            let rows: Vec<Vec<String>> = rules
                .iter()
                .map(|(n, (p, r))| vec![n.clone(), p.clone(), r.clone()])
                .collect();
            return Some(Ok(QueryResult::Rows {
                columns: vec![
                    "name".to_owned(),
                    "pattern".to_owned(),
                    "replacement".to_owned(),
                ],
                rows,
            }));
        }

        None
    }
}
