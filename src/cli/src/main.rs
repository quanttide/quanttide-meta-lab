//! reflect 命令行：五条命令，一一对应三个动作加一次检索。

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use reflect::recall;
use reflect::store::Store;

#[derive(Parser)]
#[command(
    name = "reflect",
    version,
    about = "通用反思程序：把结论拆成前提，检验前提，把修正后的前提固化成规则。"
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
    /// 记下结论。
    Add {
        /// 结论。
        claim: String,
    },
    /// 记下依赖关系：结论依赖前提。
    Dep {
        /// 结论。
        claim: String,
        /// 前提。
        premise: String,
    },
    /// 检验前提：录入反例并标为可疑。
    Test {
        /// 前提。
        premise: String,
        /// 反例。不给就只把前提标为可疑。
        #[arg(long, value_name = "反例")]
        counterexample: Option<String>,
    },
    /// 固化为规则。
    Commit {
        /// 规则。
        rule: String,
        /// 这条规则修正的前提。不给就修正当前全部可疑前提。
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
        Command::Add { claim } => {
            store.add_claim(claim);
            store.save(path)?;
            println!("已记下结论：{claim}");
        }
        Command::Dep { claim, premise } => {
            store.add_dependency(claim, premise);
            store.save(path)?;
            println!("已记下依赖：{claim} ——依赖——► {premise}");
        }
        Command::Test {
            premise,
            counterexample,
        } => {
            store.add_counterexample(premise, counterexample.as_deref());
            store.save(path)?;
            println!("已标为可疑前提：{premise}");
            if let Some(counterexample) = counterexample {
                println!("反例：{counterexample}");
            }
        }
        Command::Commit { rule, revises } => {
            let targets = store.commit_rule(rule, revises);
            store.save(path)?;
            println!("已固化规则：{rule}");
            report_revised(&targets);
        }
        // 检索只读，不落盘。
        Command::Check { event } => match event {
            Some(event) => report_hits(&store, event),
            None => report_rules(&store),
        },
    }

    Ok(())
}

fn report_revised(targets: &[String]) {
    if targets.is_empty() {
        println!("修正的前提：无（当前没有可疑前提）");
        return;
    }
    for target in targets {
        println!("修正的前提：{target}");
    }
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
