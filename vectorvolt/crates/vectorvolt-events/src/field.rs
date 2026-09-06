//! Field resolvers: computed (Rhai), nested (object), lookup enrichment.
//! VV-EVT-014, VV-EVT-015, VV-EVT-016.

use rhai::{Engine, Scope};
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::config::{EventFieldDef, StructuredField};

/// Resolve all fields of an event schema against a mutation's pk + values.
///
/// `lookup_fn` is called for lookup fields — the caller provides a closure that
/// reads from an in-process table store (signature: `(table, pk) -> Option<Vec<u64>>`).
pub fn resolve_fields<F>(
    fields: &HashMap<String, EventFieldDef>,
    pk: u64,
    values: &[u64],
    lookup_fn: &F,
) -> Value
where
    F: Fn(&str, u64) -> Option<Vec<u64>>,
{
    let engine = Engine::new();
    let mut obj = serde_json::Map::new();

    for (name, def) in fields {
        let v = resolve_one(def, pk, values, &engine, lookup_fn);
        obj.insert(name.clone(), v);
    }

    Value::Object(obj)
}

fn resolve_one<F>(
    def: &EventFieldDef,
    pk: u64,
    values: &[u64],
    engine: &Engine,
    lookup_fn: &F,
) -> Value
where
    F: Fn(&str, u64) -> Option<Vec<u64>>,
{
    match def {
        EventFieldDef::Direct(s) => resolve_direct(s, pk, values),
        EventFieldDef::Structured(sf) => resolve_structured(sf, pk, values, engine, lookup_fn),
    }
}

fn resolve_direct(s: &str, pk: u64, values: &[u64]) -> Value {
    if s == "pk" {
        return json!(pk);
    }
    // "values[N]"
    if let Some(rest) = s.strip_prefix("values[")
        && let Some(idx_str) = rest.strip_suffix(']')
        && let Ok(idx) = idx_str.parse::<usize>()
    {
        return json!(values.get(idx).copied().unwrap_or(0));
    }
    json!(s)
}

fn resolve_structured<F>(
    sf: &StructuredField,
    pk: u64,
    values: &[u64],
    engine: &Engine,
    lookup_fn: &F,
) -> Value
where
    F: Fn(&str, u64) -> Option<Vec<u64>>,
{
    // Computed (VV-EVT-014)
    if let Some(expr) = &sf.expr {
        let mut scope = Scope::new();
        scope.push("pk", pk as i64);
        let values_arr: rhai::Array = values
            .iter()
            .map(|&v| rhai::Dynamic::from(v as i64))
            .collect();
        scope.push("values", values_arr);
        return match engine.eval_expression_with_scope::<i64>(&mut scope, expr) {
            Ok(v) => json!(v),
            Err(_) => Value::Null,
        };
    }

    // Nested object (VV-EVT-015)
    if let Some(sub_fields) = &sf.object {
        let mut sub = serde_json::Map::new();
        for (k, ref_str) in sub_fields {
            sub.insert(k.clone(), resolve_direct(ref_str, pk, values));
        }
        return Value::Object(sub);
    }

    // Lookup enrichment (VV-EVT-016)
    if let Some(lk) = &sf.lookup {
        let fk_val = resolve_direct(&lk.pk, pk, values);
        let fk_u64 = fk_val.as_u64().unwrap_or(0);
        if let Some(row) = lookup_fn(&lk.table, fk_u64) {
            return json!(row.get(lk.column).copied().unwrap_or(0));
        }
        return Value::Null;
    }

    Value::Null
}
