//! 命令行端到端测试：规则存得进、召得回。
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

fn rule_texts(archive: &Value) -> Vec<String> {
    texts(archive, "text")
}

fn revised_premises(archive: &Value) -> Vec<String> {
    texts(archive, "revises")
}

fn texts(archive: &Value, field: &str) -> Vec<String> {
    archive["rules"]
        .as_array()
        .expect("rules array")
        .iter()
        .flat_map(|rule| match &rule[field] {
            Value::String(text) => vec![text.clone()],
            Value::Array(items) => items
                .iter()
                .map(|item| item.as_str().expect("text").to_string())
                .collect(),
            other => panic!("unexpected {field}: {other}"),
        })
        .collect()
}

#[test]
fn commit_persists_a_rule() {
    let store = store_path("commit");
    let output = reflect(&store, &["commit", "收编是否有利，取决于是否要求闭嘴"]);

    assert!(output.status.success(), "{}", stdout(&output));
    assert_eq!(
        rule_texts(&archive(&store)),
        vec!["收编是否有利，取决于是否要求闭嘴"]
    );
}

#[test]
fn commit_keeps_the_premises_it_revises() {
    let store = store_path("commit_revises");
    let output = reflect(
        &store,
        &[
            "commit",
            "收编是否有利，取决于是否要求闭嘴",
            "--revises",
            "收编=失去独立性",
            "--revises",
            "失去独立性=信任崩塌",
        ],
    );

    assert!(stdout(&output).contains("修正的前提：收编=失去独立性"));
    assert_eq!(
        revised_premises(&archive(&store)),
        vec!["收编=失去独立性", "失去独立性=信任崩塌"]
    );
}

#[test]
fn committing_the_same_rule_again_merges_premises() {
    let store = store_path("commit_twice");
    reflect(
        &store,
        &["commit", "抽象词必须先钉边界", "--revises", "提醒=通知系统"],
    );
    reflect(
        &store,
        &[
            "commit",
            "抽象词必须先钉边界",
            "--revises",
            "提醒=通知系统",
            "--revises",
            "AI理解的提醒=我的提醒",
        ],
    );

    let archive = archive(&store);
    assert_eq!(rule_texts(&archive), vec!["抽象词必须先钉边界"]);
    assert_eq!(
        revised_premises(&archive),
        vec!["提醒=通知系统", "AI理解的提醒=我的提醒"]
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
fn check_does_not_write_the_archive() {
    let store = store_path("check_read_only");
    reflect(&store, &["check"]);

    assert!(!store.exists(), "check 只读，不该落盘");
}

#[test]
fn check_reports_no_hit_for_an_unrelated_event() {
    let store = store_path("check_miss");
    reflect(&store, &["commit", "抽象词必须先钉边界"]);

    let output = reflect(&store, &["check", "数据库连接池配置"]);
    assert!(stdout(&output).contains("没有命中规则"));
}

#[test]
fn check_finds_a_rule_through_the_premise_it_revises() {
    let store = store_path("check_premise");
    reflect(
        &store,
        &[
            "commit",
            "抽象词必须先钉边界",
            "--revises",
            "AI理解的提醒=我的提醒",
        ],
    );

    let output = reflect(&store, &["check", "让AI写用户积分功能"]);
    let printed = stdout(&output);
    assert!(printed.contains("抽象词必须先钉边界"), "{printed}");
}

#[test]
fn day_one_to_day_thirty_ends_with_a_hit() {
    let store = store_path("day1_day30");

    // Day 1：一次偏离固化成两条规则。
    reflect(
        &store,
        &[
            "commit",
            "让AI写功能前先给复杂度预算：最多几个文件、几个概念",
            "--revises",
            "AI会把复杂度和需求规模匹配",
        ],
    );
    reflect(
        &store,
        &[
            "commit",
            "需求里的抽象词必须先钉死边界",
            "--revises",
            "AI理解的提醒=我的提醒",
        ],
    );

    // Day 30：又要让 AI 写一个功能，check 应该把它拦住。
    let output = reflect(&store, &["check", "让AI写用户积分功能，抽象词和边界要说清"]);

    let printed = stdout(&output);
    assert!(
        printed.contains("需求里的抽象词必须先钉死边界"),
        "{printed}"
    );
}
