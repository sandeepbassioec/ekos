//! `vvctl vector` insert / get / search against a real table directory.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::schema::{ColumnDef, ColumnType};
use vectorvolt_engine::{EngineOptions, TableRegistry, TableSchema};
use vvctl::commands::vector::{
    VectorArgs, VectorCommand, VectorGetArgs, VectorInsertArgs, VectorSearchArgs, run,
};

fn make_table(dir: &TempDir) {
    let r = TableRegistry::open(dir.path(), EngineOptions::default()).unwrap();
    let schema = TableSchema::new(vec![
        ColumnDef::new("id", ColumnType::U64),
        ColumnDef::new("emb", ColumnType::Vector(3)),
    ])
    .unwrap();
    r.create_table_typed("items", schema).unwrap();
}

fn insert(dir: &TempDir, pk: u64, v: &str) -> anyhow::Result<()> {
    run(VectorArgs {
        command: VectorCommand::Insert(VectorInsertArgs {
            table: "items".into(),
            col: 1,
            pk,
            vec: v.into(),
            dir: dir.path().into(),
        }),
    })
}

#[test]
fn inserted_vector_is_persisted_and_searchable() {
    let d = TempDir::new().unwrap();
    make_table(&d);
    insert(&d, 1, "1,0,0").unwrap();
    insert(&d, 2, "0,1,0").unwrap();
    insert(&d, 3, "0,0,1").unwrap();

    // Verify through a fresh registry that the CLI's writes are visible.
    let r = Arc::new(TableRegistry::open(d.path(), EngineOptions::default()).unwrap());
    let e = r.get("items").unwrap();
    assert_eq!(e.get_vector(1, 2).unwrap(), vec![0.0, 1.0, 0.0]);
    let hits = e.search_vector(1, &[0.0, 0.0, 1.0], 1).unwrap();
    assert_eq!(hits[0].0, 3);

    run(VectorArgs {
        command: VectorCommand::Search(VectorSearchArgs {
            table: "items".into(),
            col: 1,
            vec: "1,0,0".into(),
            k: 2,
            dir: d.path().into(),
        }),
    })
    .unwrap();
    run(VectorArgs {
        command: VectorCommand::Get(VectorGetArgs {
            table: "items".into(),
            col: 1,
            pk: 1,
            dir: d.path().into(),
        }),
    })
    .unwrap();
}

#[test]
fn vector_errors_surface_as_failures() {
    let d = TempDir::new().unwrap();
    make_table(&d);
    assert!(insert(&d, 1, "a,b,c").is_err(), "non-float rejected");
    assert!(insert(&d, 1, "1,2").is_err(), "wrong dimension rejected");
    let r = run(VectorArgs {
        command: VectorCommand::Insert(VectorInsertArgs {
            table: "ghost".into(),
            col: 1,
            pk: 1,
            vec: "1,2,3".into(),
            dir: d.path().into(),
        }),
    });
    assert!(r.is_err(), "unknown table rejected");
}
