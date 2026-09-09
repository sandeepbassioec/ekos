//! SchemaVersionRegistry — file-based event schema version store (VV-EVT-013).
//!
//! Layout: `event_schemas/<entity>/<event_name>/<version>.json`
//! Unknown/legacy entity stored under `event_schemas/_unbound/<event_name>/`.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use thiserror::Error;

use crate::config::DomainEventSchema;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("schema not found: {0}/{1} @ {2}")]
    NotFound(String, String, String),
}

pub type RegistryResult<T> = Result<T, RegistryError>;

/// Persists every version of every event schema, grouped by entity/domain.
pub struct SchemaVersionRegistry {
    base: PathBuf,
}

impl SchemaVersionRegistry {
    pub fn new(base_dir: impl AsRef<Path>) -> RegistryResult<Self> {
        let base = base_dir.as_ref().join("event_schemas");
        fs::create_dir_all(&base)?;
        Ok(Self { base })
    }

    fn entity_key(schema: &DomainEventSchema) -> &str {
        if schema.entity.is_empty() {
            "_unbound"
        } else {
            &schema.entity
        }
    }

    fn event_dir(&self, entity: &str, event_name: &str) -> PathBuf {
        self.base.join(entity).join(event_name)
    }

    fn event_path(&self, entity: &str, event_name: &str, version: &str) -> PathBuf {
        self.event_dir(entity, event_name)
            .join(format!("{version}.json"))
    }

    /// Store a schema version. No-ops if exact version already exists.
    pub fn register(&self, schema: &DomainEventSchema) -> RegistryResult<()> {
        let entity = Self::entity_key(schema).to_string();
        let dir = self.event_dir(&entity, &schema.name);
        fs::create_dir_all(&dir)?;
        let path = self.event_path(&entity, &schema.name, &schema.version);
        if !path.exists() {
            let json = serde_json::to_string_pretty(schema)?;
            fs::write(&path, json)?;
        }
        Ok(())
    }

    /// List all known entity/domain names.
    pub fn list_domains(&self) -> RegistryResult<Vec<String>> {
        let mut domains = read_dir_names(&self.base)?;
        domains.sort();
        Ok(domains)
    }

    /// List all event names under a domain, sorted.
    pub fn list_events(&self, entity: &str) -> RegistryResult<Vec<String>> {
        let mut events = read_dir_names(&self.base.join(entity))?;
        events.sort();
        Ok(events)
    }

    /// List all versions for an event, sorted ascending.
    pub fn list_versions(&self, entity: &str, event_name: &str) -> RegistryResult<Vec<String>> {
        let dir = self.event_dir(entity, event_name);
        if !dir.exists() {
            return Ok(vec![]);
        }
        let mut versions: Vec<String> = fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.strip_suffix(".json").map(str::to_string)
            })
            .collect();
        versions.sort();
        Ok(versions)
    }

    /// Read a specific schema version.
    pub fn show(
        &self,
        entity: &str,
        event_name: &str,
        version: &str,
    ) -> RegistryResult<DomainEventSchema> {
        let path = self.event_path(entity, event_name, version);
        let json = fs::read_to_string(&path).map_err(|_| {
            RegistryError::NotFound(entity.into(), event_name.into(), version.into())
        })?;
        Ok(serde_json::from_str(&json)?)
    }

    /// Diff two versions — returns JSON patch-style lines prefixed with +/-/~.
    pub fn diff(
        &self,
        entity: &str,
        event_name: &str,
        v1: &str,
        v2: &str,
    ) -> RegistryResult<Vec<String>> {
        let s1: Value = serde_json::to_value(self.show(entity, event_name, v1)?)?;
        let s2: Value = serde_json::to_value(self.show(entity, event_name, v2)?)?;
        let mut diffs = Vec::new();
        if let (Some(o1), Some(o2)) = (s1.as_object(), s2.as_object()) {
            for (k, v) in o1 {
                match o2.get(k) {
                    None => diffs.push(format!("- {k}")),
                    Some(v2_val) if v != v2_val => diffs.push(format!("~ {k}")),
                    _ => {}
                }
            }
            for k in o2.keys() {
                if !o1.contains_key(k) {
                    diffs.push(format!("+ {k}"));
                }
            }
        }
        Ok(diffs)
    }
}

fn read_dir_names(dir: &Path) -> RegistryResult<Vec<String>> {
    if !dir.exists() {
        return Ok(vec![]);
    }
    Ok(fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect())
}
