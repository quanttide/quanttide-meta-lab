//! 命令行端到端测试：存得进、召得回。
//!
//! 场景取自 `docs/dev-guide/reflect.md` 的「被收编对 IP 不利」与
//! `docs/dev-guide/reflect_test.md` 的 Day 1 → Day 30。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_reflect")
}

/// 每个用例一个干净目录，返回存档路径。
fn store_path(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir.join("reflect.json")
}

fn reflect(store: &Path, args: &[&str]) -> Output {
    Command::new(binary())
        .arg("--store")
        .arg(store)
        .args(args)
        .output()
        .expect("run reflect")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn archive(store: &Path) -> Value {
    let raw = fs::read_to_string(store).expect("read archive");
    serde_json::from_str(&raw).expect("parse archive")
}

fn node_texts(archive: &Value, kind: &str) -> Vec<String> {
    archive["nodes"]
        .as_array()
        .expect("nodes array")
        .iter()
        .filter(|node| node["kind"] == kind)
        .map(|node| node["text"].as_str().expect("text").to_string())
        .collect()
}

#[test]
fn add_persists_a_claim() {
    let store = store_path("add_claim");
    let output = reflect(&store, &["add", "被收编对IP不利"]);

    assert!(output.status.success(), "{}", stdout(&output));
    assert_eq!(
        node_texts(&archive(&store), "claim"),
        vec!["被收编对IP不利"]
    );
}

#[test]
fn dep_creates_both_nodes_and_the_edge() {
    let store = store_path("dep");
    reflect(&store, &["add", "被收编对IP不利"]);
    reflect(&store, &["dep", "被收编对IP不利", "收编=失去独立性"]);

    let archive = archive(&store);
    assert_eq!(node_texts(&archive, "claim"), vec!["被收编对IP不利"]);
    assert_eq!(node_texts(&archive, "premise"), vec!["收编=失去独立性"]);
    assert_eq!(
        archive["edges"][0],
        serde_json::json!({
            "kind": "depends_on",
            "from": "被收编对IP不利",
            "to": "收编=失去独立性",
        })
    );
}

#[test]
fn test_marks_the_premise_suspect_and_keeps_the_counterexample() {
    let store = store_path("test");
    reflect(&store, &["add", "被收编对IP不利"]);
    reflect(&store, &["dep", "被收编对IP不利", "收编=失去独立性"]);
    reflect(
        &store,
        &[
            "test",
            "收编=失去独立性",
            "--counterexample",
            "分析师转顾问，信息更准、公信力没掉",
        ],
    );

    let archive = archive(&store);
    let premise = archive["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["kind"] == "premise")
        .expect("premise");
    assert_eq!(premise["suspect"], true);
    assert_eq!(
        premise["counterexample"],
        "分析师转顾问，信息更准、公信力没掉"
    );
}

#[test]
fn commit_revises_every_suspect_premise_by_default() {
    let store = store_path("commit");
    reflect(&store, &["add", "被收编对IP不利"]);
    reflect(&store, &["dep", "被收编对IP不利", "收编=失去独立性"]);
    reflect(&store, &["test", "收编=失去独立性"]);
    let output = reflect(&store, &["commit", "收编是否有利，取决于是否要求闭嘴"]);

    assert!(stdout(&output).contains("修正的前提：收编=失去独立性"));
    let archive = archive(&store);
    assert_eq!(
        node_texts(&archive, "rule"),
        vec!["收编是否有利，取决于是否要求闭嘴"]
    );
    assert_eq!(
        archive["edges"][1],
        serde_json::json!({
            "kind": "revises",
            "from": "收编是否有利，取决于是否要求闭嘴",
            "to": "收编=失去独立性",
        })
    );
}

#[test]
fn day_one_to_day_thirty_ends_with_a_hit() {
    let store = store_path("day1_day30");

    // Day 1：拆解、检验、固化。
    reflect(&store, &["add", "AI实现符合我的意图"]);
    reflect(
        &store,
        &["dep", "AI实现符合我的意图", "AI理解的提醒=我的提醒"],
    );
    reflect(
        &store,
        &[
            "test",
            "AI理解的提醒=我的提醒",
            "--counterexample",
            "AI加了 RetryPolicy、Channel、Template，我根本没提",
        ],
    );
    reflect(
        &store,
        &["commit", "需求里的抽象词必须先钉死边界，提醒不含多渠道"],
    );

    // Day 30：又要让 AI 写一个功能，check 应该把它拦住。
    let output = reflect(&store, &["check", "让AI写用户积分功能，抽象词和边界要说清"]);

    let printed = stdout(&output);
    assert!(
        printed.contains("需求里的抽象词必须先钉死边界"),
        "{printed}"
    );
}

#[test]
fn check_lists_rules_when_no_event_is_given() {
    let store = store_path("check_all");
    let empty = reflect(&store, &["check"]);
    assert!(stdout(&empty).contains("还没有规则"));

    reflect(&store, &["commit", "抽象词必须先钉边界"]);
    let listed = reflect(&store, &["check"]);
    assert!(stdout(&listed).contains("规则：抽象词必须先钉边界"));
}

#[test]
fn check_reports_no_hit_for_an_unrelated_event() {
    let store = store_path("check_miss");
    reflect(&store, &["commit", "抽象词必须先钉边界"]);

    let output = reflect(&store, &["check", "数据库连接池配置"]);
    assert!(stdout(&output).contains("没有命中规则"));
}
