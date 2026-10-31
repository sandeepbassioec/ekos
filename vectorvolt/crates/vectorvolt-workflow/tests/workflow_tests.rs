use std::sync::Arc;
use tempfile::TempDir;
use vectorvolt_workflow::{StepState, WorkflowEngine, WorkflowState};

fn setup() -> (WorkflowEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let engine = WorkflowEngine::open(dir.path()).unwrap();
    (engine, dir)
}

#[test]
fn create_workflow() {
    let (engine, _dir) = setup();
    let id = engine
        .create("test", vec![("step1".into(), vec![])])
        .unwrap();
    assert!(id >= 1);
}

#[test]
fn start_workflow() {
    let (engine, _dir) = setup();
    let id = engine.create("wf", vec![("s1".into(), vec![])]).unwrap();
    engine.start(id).unwrap();
    let wf = engine.get(id).unwrap().unwrap();
    assert_eq!(wf.state, WorkflowState::Running);
}

#[test]
fn ready_steps_returns_steps_with_no_deps() {
    let (engine, _dir) = setup();
    let id = engine
        .create("wf", vec![("s0".into(), vec![]), ("s1".into(), vec![0])])
        .unwrap();
    engine.start(id).unwrap();
    let ready = engine.ready_steps(id).unwrap();
    assert_eq!(ready, vec![0]);
}

#[test]
fn complete_step_advances_state() {
    let (engine, _dir) = setup();
    let id = engine.create("wf", vec![("s0".into(), vec![])]).unwrap();
    engine.start(id).unwrap();
    engine.complete_step(id, 0).unwrap();
    let wf = engine.get(id).unwrap().unwrap();
    assert_eq!(wf.steps[0].state, StepState::Completed);
}

#[test]
fn fail_step_marks_failed() {
    let (engine, _dir) = setup();
    let id = engine.create("wf", vec![("s0".into(), vec![])]).unwrap();
    engine.start(id).unwrap();
    engine.fail_step(id, 0, "oops").unwrap();
    let wf = engine.get(id).unwrap().unwrap();
    assert_eq!(wf.state, WorkflowState::Failed);
    assert!(matches!(wf.steps[0].state, StepState::Failed(_)));
}

#[test]
fn workflow_completes_when_all_steps_done() {
    let (engine, _dir) = setup();
    let id = engine
        .create("wf", vec![("s0".into(), vec![]), ("s1".into(), vec![])])
        .unwrap();
    engine.start(id).unwrap();
    engine.complete_step(id, 0).unwrap();
    engine.complete_step(id, 1).unwrap();
    assert!(engine.is_complete(id).unwrap());
}

#[test]
fn workflow_fails_when_step_fails() {
    let (engine, _dir) = setup();
    let id = engine.create("wf", vec![("s0".into(), vec![])]).unwrap();
    engine.start(id).unwrap();
    engine.fail_step(id, 0, "disk full").unwrap();
    let wf = engine.get(id).unwrap().unwrap();
    assert_eq!(wf.state, WorkflowState::Failed);
}

#[test]
fn get_returns_workflow() {
    let (engine, _dir) = setup();
    let id = engine.create("mywf", vec![]).unwrap();
    let wf = engine.get(id).unwrap();
    assert!(wf.is_some());
    assert_eq!(wf.unwrap().name, "mywf");
}

#[test]
fn ready_steps_excludes_incomplete_deps() {
    let (engine, _dir) = setup();
    // s1 depends on s0. s0 not yet complete => s1 not ready.
    let id = engine
        .create("wf", vec![("s0".into(), vec![]), ("s1".into(), vec![0])])
        .unwrap();
    engine.start(id).unwrap();
    let ready = engine.ready_steps(id).unwrap();
    assert!(!ready.contains(&1));
}

#[test]
fn linear_chain_step2_ready_after_step1() {
    let (engine, _dir) = setup();
    let id = engine
        .create("chain", vec![("s0".into(), vec![]), ("s1".into(), vec![0])])
        .unwrap();
    engine.start(id).unwrap();
    engine.complete_step(id, 0).unwrap();
    let ready = engine.ready_steps(id).unwrap();
    assert!(ready.contains(&1));
}

#[test]
fn diamond_dag_step3_ready_after_step2() {
    let (engine, _dir) = setup();
    // s0 -> s1 -> s3
    //          -> s2 -> s3
    // Actually simpler diamond: s0 -> s1, s0 -> s2, {s1,s2} -> s3
    let id = engine
        .create(
            "diamond",
            vec![
                ("s0".into(), vec![]),
                ("s1".into(), vec![0]),
                ("s2".into(), vec![0]),
                ("s3".into(), vec![1, 2]),
            ],
        )
        .unwrap();
    engine.start(id).unwrap();
    engine.complete_step(id, 0).unwrap();
    engine.complete_step(id, 1).unwrap();
    engine.complete_step(id, 2).unwrap();
    let ready = engine.ready_steps(id).unwrap();
    assert!(ready.contains(&3));
}

#[test]
fn is_complete_false_then_true() {
    let (engine, _dir) = setup();
    let id = engine.create("wf", vec![("s0".into(), vec![])]).unwrap();
    engine.start(id).unwrap();
    assert!(!engine.is_complete(id).unwrap());
    engine.complete_step(id, 0).unwrap();
    assert!(engine.is_complete(id).unwrap());
}

#[test]
fn concurrent_complete_step_calls_safe() {
    let dir = TempDir::new().unwrap();
    let engine = Arc::new(WorkflowEngine::open(dir.path()).unwrap());
    // Create 10 independent steps.
    let steps: Vec<(String, Vec<u64>)> = (0..10).map(|i| (format!("s{i}"), vec![])).collect();
    let id = engine.create("concurrent", steps).unwrap();
    engine.start(id).unwrap();

    let mut handles = Vec::new();
    for step_id in 0..10u64 {
        let e = Arc::clone(&engine);
        handles.push(std::thread::spawn(move || {
            e.complete_step(id, step_id).unwrap();
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    assert!(engine.is_complete(id).unwrap());
}
