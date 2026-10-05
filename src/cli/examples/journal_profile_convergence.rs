//! 日志 ↔ 档案收敛 —— `docs/gallery/category/journal-vs-profile.md` 第七节的工程翻译。
//!
//! 题面：日志有三条事件，档案只有「进行中」一个状态。候选一致视图 `convergence_candidate`
//! 停在 pending，能不能直接标成 converged？引擎用两条公理检查，不通就报出是哪条公理。

use pr4xis::engine::{Action, Engine, EngineError, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

const CITATION: &str = "docs/gallery/category/journal-vs-profile.md";

fn axiom_meta(name: &'static str, description: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(CITATION),
        module_path: ModulePath::new_static(module_path!()),
    }
}

/// 收敛阶段 —— `convergence_candidate.status`。
#[derive(Debug, Clone, PartialEq)]
enum Stage {
    Pending,
    Converged,
}

/// 候选一致视图 —— `convergence_candidate` 表的一行。
#[derive(Debug, Clone, PartialEq)]
struct Candidate {
    /// 日志事件（按时间追加）。
    log_events: Vec<&'static str>,
    /// 已在档案侧找到对应的事件下标。
    aligned: Vec<usize>,
    /// 档案侧无对应、被显式标记的事件下标（信息丢失可见）。
    unmatched: Vec<usize>,
    /// 人类确认 —— `convergence_candidate.approved_by`。
    approved_by: Option<&'static str>,
    stage: Stage,
}

#[derive(Debug, Clone)]
enum CandidateAction {
    /// 自然变换给出的对齐：日志事件 → 档案状态转换。
    Align { event: usize },
    /// 档案侧无对应，显式标记（第七节「哪些事件在档案中没有对应」）。
    MarkUnmatched { event: usize },
    /// 人类在余锥里选定方案。
    Approve { by: &'static str },
    /// 写入 `converged_master`。
    Converge,
}

impl Situation for Candidate {}

impl Action for CandidateAction {
    type Sit = Candidate;
}

/// 公理一：收敛前，每条日志事件都要在档案侧有对应，或被显式标记为无对应。
struct AlignmentComplete;

impl Precondition<CandidateAction> for AlignmentComplete {
    fn check(&self, situation: &Candidate, action: &CandidateAction) -> Verdict {
        let meta = axiom_meta(
            "AlignmentComplete",
            "收敛前：每条日志事件都要在档案侧有对应，或被显式标记为无对应",
        );
        if !matches!(action, CandidateAction::Converge) {
            return Ok(Box::new(SimpleProof::new(meta)));
        }
        let missing: Vec<usize> = (0..situation.log_events.len())
            .filter(|i| !situation.aligned.contains(i) && !situation.unmatched.contains(i))
            .collect();
        if missing.is_empty() {
            return Ok(Box::new(SimpleProof::new(meta)));
        }
        let events: Vec<&str> = missing.iter().map(|&i| situation.log_events[i]).collect();
        let mut meta = meta;
        meta.description = Label::new(format!("未对齐的日志事件：{events:?}"));
        Err(Box::new(SimpleCounterexample::new(meta)))
    }
}

/// 公理二：收敛前必须有人类确认（`approved_by` 非空）。
struct HumanConfirmed;

impl Precondition<CandidateAction> for HumanConfirmed {
    fn check(&self, situation: &Candidate, action: &CandidateAction) -> Verdict {
        let meta = axiom_meta("HumanConfirmed", "收敛前必须有人类确认（approved_by 非空）");
        if !matches!(action, CandidateAction::Converge) {
            return Ok(Box::new(SimpleProof::new(meta)));
        }
        if situation.approved_by.is_some() {
            return Ok(Box::new(SimpleProof::new(meta)));
        }
        let mut meta = meta;
        meta.description = Label::new_static("候选视图尚无人类确认，停在 pending");
        Err(Box::new(SimpleCounterexample::new(meta)))
    }
}

fn apply(
    situation: &Candidate,
    action: &CandidateAction,
) -> Result<Candidate, Box<dyn Counterexample>> {
    let mut next = situation.clone();
    match action {
        CandidateAction::Align { event } => {
            if !next.aligned.contains(event) {
                next.aligned.push(*event);
            }
        }
        CandidateAction::MarkUnmatched { event } => {
            if !next.unmatched.contains(event) {
                next.unmatched.push(*event);
            }
        }
        CandidateAction::Approve { by } => next.approved_by = Some(by),
        CandidateAction::Converge => next.stage = Stage::Converged,
    }
    Ok(next)
}

fn main() {
    println!("题目：日志说「10月5日暂停B渠道」，档案说「进行中」，候选视图停在 pending。");
    println!("出处：{CITATION} 第七节\n");

    let initial = Candidate {
        log_events: vec!["10月1日投放A渠道", "10月3日调整预算", "10月5日暂停B渠道"],
        aligned: vec![0],
        unmatched: vec![],
        approved_by: None,
        stage: Stage::Pending,
    };

    let engine = Engine::new(
        initial,
        vec![Box::new(AlignmentComplete), Box::new(HumanConfirmed)],
        apply,
    );

    // 1) 未对齐、未经确认就收敛 —— 两条公理把它拦下
    let engine = match engine.next(CandidateAction::Converge) {
        Err(EngineError::Violated { engine, violations }) => {
            println!("1) 直接 next(Converge) —— 被公理拦下：");
            for v in &violations {
                let meta = v.meta();
                println!("   ✗ {}: {}", meta.name.as_str(), meta.description.as_str());
            }
            println!("   候选停在 {:?}\n", engine.situation().stage);
            engine
        }
        Err(other) => panic!("预期 Violated，得到 {other:?}"),
        Ok(_) => panic!("预期 Converge 被拦下"),
    };

    // 2~4) 补齐对齐、显式标记信息丢失、人类确认
    let engine = engine
        .next(CandidateAction::Align { event: 1 })
        .expect("档案侧有对应的事件，对齐应通过")
        .next(CandidateAction::MarkUnmatched { event: 2 })
        .expect("档案侧无对应的事件，显式标记应通过")
        .next(CandidateAction::Approve {
            by: "human-reviewer",
        })
        .expect("人类确认应通过");
    println!("2) Align(调整预算) + MarkUnmatched(暂停B渠道) + Approve → 通过\n");

    // 5) 两条公理都满足 —— 收敛
    let engine = engine
        .next(CandidateAction::Converge)
        .expect("公理都满足，收敛应通过");
    println!("3) next(Converge) → {:?}", engine.situation().stage);

    // 6) 撤销与重做
    let engine = engine.back().expect(".back() 应能撤销");
    println!("4) .back()  → {:?}", engine.situation().stage);
    let engine = engine.forward().expect(".forward() 应能重做");
    println!("   .forward() → {:?}", engine.situation().stage);

    println!("\n最终候选：{:?}", engine.situation());
    println!(
        "轨迹共 {} 条（含被拦下的那次）",
        engine.trace().entries().len()
    );
}
