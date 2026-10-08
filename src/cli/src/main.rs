//! reflect 命令行：两条命令，一条记规则，一条在动手前把它找回来。

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use reflect::recall;
use reflect::store::Store;

#[derive(Parser)]
#[command(
    name = "reflect",
    version,
    about = "通用反思程序：把修正后的前提固化成规则，动手前找回来。"
)]
struct Cli {
    /// 存档文件，一个 JSON 文件。
    #[arg(
        long,
        global = true,
        default_value = "reflect.json",
        value_name = "文件"
    )]
    store: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 固化为规则。
    Commit {
        /// 规则，写成可执行的句子，带上事件里会出现的词。
        rule: String,
        /// 这条规则修正的前提。
        #[arg(long = "revises", value_name = "前提")]
        revises: Vec<String>,
    },
    /// 检索相关规则，动手前提醒。
    Check {
        /// 事件描述。不给就列出全部规则。
        event: Option<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("错误：{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let path = cli.store.as_path();
    let mut store = Store::load(path)?;

    match &cli.command {
        Command::Commit { rule, revises } => {
            store.commit_rule(rule, revises);
            store.save(path)?;
            println!("已固化规则：{rule}");
            for premise in revises {
                println!("修正的前提：{premise}");
            }
        }
        // 检索只读，不落盘。
        Command::Check { event } => match event {
            Some(event) => report_hits(&store, event),
            None => report_rules(&store),
        },
    }

    Ok(())
}

fn report_hits(store: &Store, event: &str) {
    let hits = recall::recall(store, event);
    if hits.is_empty() {
        println!("没有命中规则。");
    }
    for hit in hits {
        println!("命中（{:.0}%）：{}", hit.score * 100.0, hit.rule.text);
        println!("  词：{}", hit.matched.join("、"));
    }
}

fn report_rules(store: &Store) {
    let rules = store.rules();
    if rules.is_empty() {
        println!("还没有规则。");
    }
    for rule in rules {
        println!("规则：{}", rule.text);
    }
}
