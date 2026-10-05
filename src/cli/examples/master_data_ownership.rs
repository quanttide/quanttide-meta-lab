//! 主数据三层归属 —— `docs/gallery/ontology/entity/master-data.md`（2026-10-04 决议）。
//!
//! 归属规则先声明成本体的 OwnedBy 边，引擎只做一件事：检查动作是否符合已声明的本体。
//! 规则要改，改本体声明，不改引擎。

use pr4xis::category::{Arrow, Category, Concept};
use pr4xis::engine::{Action, Engine, EngineError, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

const CITATION: &str = "docs/gallery/ontology/entity/master-data.md (2026-10-04)";

fn axiom_meta(name: &'static str, description: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(CITATION),
        module_path: ModulePath::new_static(module_path!()),
    }
}

pr4xis::ontology! {
    name: "MasterDataOwnership",
    source: "docs/gallery/ontology/entity/master-data.md (2026-10-04)",

    concepts: [MasterData, Definition, OntologyLayer, Operations, MetaEngineering, DataEngineering, BusinessSystem],

    labels: {
        MasterData: ("zh", "主数据", "实体层精炼后的核心子集"),
        Definition: ("zh", "定义", "实体类别、唯一标识、标准字段、元关系 —— 归元工程"),
        OntologyLayer: ("zh", "本体", "实例数据的正本 —— 归各自业务系统"),
        Operations: ("zh", "运营", "采集、清洗、去重合并、分发、监控 —— 归数据工程"),
        MetaEngineering: ("zh", "元工程", "元标准与标准字段的定义域"),
        DataEngineering: ("zh", "数据工程", "数据生产的工程域"),
        BusinessSystem: ("zh", "业务系统", "实例正本所在地（CRM、档案等）"),
    },

    edges: [
        (MasterData, Definition, HasLayer),
        (MasterData, OntologyLayer, HasLayer),
        (MasterData, Operations, HasLayer),
        (Definition, MetaEngineering, OwnedBy),
        (OntologyLayer, BusinessSystem, OwnedBy),
        (Operations, DataEngineering, OwnedBy),
    ],
}

/// 主数据的三层 —— 决议的「定义 / 本体 / 运营」。
#[derive(Debug, Clone, PartialEq)]
enum Layer {
    Definition,
    Ontology,
    Operations,
}

impl Layer {
    fn concept(&self) -> MasterDataOwnershipConcept {
        match self {
            Layer::Definition => MasterDataOwnershipConcept::Definition,
            Layer::Ontology => MasterDataOwnershipConcept::OntologyLayer,
            Layer::Operations => MasterDataOwnershipConcept::Operations,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Layer::Definition => "定义",
            Layer::Ontology => "本体",
            Layer::Operations => "运营",
        }
    }
}

/// 归属方案 —— 每层归哪个主体。
#[derive(Debug, Clone, PartialEq)]
struct OwnershipPlan {
    assignments: Vec<(Layer, MasterDataOwnershipConcept)>,
}

impl Situation for OwnershipPlan {}

#[derive(Debug, Clone)]
enum OwnershipAction {
    Assign {
        layer: Layer,
        owner: MasterDataOwnershipConcept,
    },
}

impl Action for OwnershipAction {
    type Sit = OwnershipPlan;
}

/// 归属只能是本体已声明的 OwnedBy 边。
struct DeclaredInOntology;

impl Precondition<OwnershipAction> for DeclaredInOntology {
    fn check(&self, _situation: &OwnershipPlan, action: &OwnershipAction) -> Verdict {
        let meta = axiom_meta("DeclaredInOntology", "归属必须是本体已声明的 OwnedBy 边");
        let OwnershipAction::Assign { layer, owner } = action;
        let concept = layer.concept();
        let declared = MasterDataOwnershipCategory::morphisms().iter().any(|m| {
            m.source() == concept
                && &m.target() == owner
                && m.kind() == MasterDataOwnershipRelationKind::OwnedBy
        });
        if declared {
            return Ok(Box::new(SimpleProof::new(meta)));
        }
        let mut meta = meta;
        meta.description = Label::new(format!(
            "本体未声明「{}」归 {} —— 已声明的归属见 OwnedBy 边",
            layer.name(),
            label_of(*owner)
        ));
        Err(Box::new(SimpleCounterexample::new(meta)))
    }
}

fn apply(
    situation: &OwnershipPlan,
    action: &OwnershipAction,
) -> Result<OwnershipPlan, Box<dyn Counterexample>> {
    let mut next = situation.clone();
    let OwnershipAction::Assign { layer, owner } = action;
    next.assignments.retain(|(l, _)| l != layer);
    next.assignments.push((layer.clone(), *owner));
    Ok(next)
}

/// 用本体标签打印概念（查不到就退回概念名）。
fn label_of(concept: MasterDataOwnershipConcept) -> String {
    MasterDataOwnershipOntology::labels()
        .iter()
        .find(|(c, _, _, _)| *c == concept)
        .map(|(_, _, label, _)| label.to_string())
        .unwrap_or_else(|| concept.name().to_string())
}

fn main() {
    println!("本体 MasterDataOwnership 已声明的归属（OwnedBy 边）：");
    for m in MasterDataOwnershipCategory::morphisms() {
        if m.kind() == MasterDataOwnershipRelationKind::OwnedBy {
            println!("  {} 归 {}", label_of(m.source()), label_of(m.target()));
        }
    }

    let engine = Engine::new(
        OwnershipPlan {
            assignments: vec![],
        },
        vec![Box::new(DeclaredInOntology)],
        apply,
    );

    // 1) 定义归元工程 —— 决议第一行
    let engine = engine
        .next(OwnershipAction::Assign {
            layer: Layer::Definition,
            owner: MasterDataOwnershipConcept::MetaEngineering,
        })
        .expect("定义归元工程 —— 本体已声明");
    println!("\n1) 定义归元工程 → 通过");

    // 2) 本体层归数据工程 —— 决议否决的那条老路
    let engine = match engine.next(OwnershipAction::Assign {
        layer: Layer::Ontology,
        owner: MasterDataOwnershipConcept::DataEngineering,
    }) {
        Err(EngineError::Violated { engine, violations }) => {
            println!("2) 本体归数据工程 → 被拦下");
            for v in &violations {
                let meta = v.meta();
                println!("   ✗ {}: {}", meta.name.as_str(), meta.description.as_str());
            }
            engine
        }
        Err(other) => panic!("预期 Violated，得到 {other:?}"),
        Ok(_) => panic!("预期本体归数据工程被拦下"),
    };

    // 3) 本体归业务系统、运营归数据工程 —— 三层各归其位
    let engine = engine
        .next(OwnershipAction::Assign {
            layer: Layer::Ontology,
            owner: MasterDataOwnershipConcept::BusinessSystem,
        })
        .expect("本体归业务系统 —— 本体已声明")
        .next(OwnershipAction::Assign {
            layer: Layer::Operations,
            owner: MasterDataOwnershipConcept::DataEngineering,
        })
        .expect("运营归数据工程 —— 本体已声明");
    println!("3) 本体归业务系统、运营归数据工程 → 通过");

    println!("\n最终归属方案：");
    for (layer, owner) in &engine.situation().assignments {
        println!("  {} 归 {}", layer.name(), label_of(*owner));
    }
    println!(
        "\n轨迹共 {} 条（含被拦下的那次）",
        engine.trace().entries().len()
    );
}
