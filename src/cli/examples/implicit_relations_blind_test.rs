//! 盲测：docs/gallery 文档明说的关系里，pr4xis 能推出多少隐含关系。
//!
//! 实验协议（本文件与 EXPECTED 预测先提交，运行在后，git 提交时间即盲底）：
//! 1. 只声明四篇文档明说的关系，隐含关系一条不写；
//! 2. EXPECTED 是运行前写死的预测，四条全符 → 自动推导可信；
//!    任何一条不符 → 自动部分按不可信处理，物化出的每条预期外关系人工裁决；
//! 3. 对照组是手工推导清单（即 EXPECTED）与 if 规则表（零推导）。

use pr4xis::category::laws::category_law_axioms;
use pr4xis::category::{Arrow, Category};
use pr4xis::logic::Axiom;

// ---------------------------------------------------------------------------
// 材料：只声明文档明说的关系
// ---------------------------------------------------------------------------

pr4xis::ontology! {
    name: "GalleryRules",
    source: "docs/gallery (master-data 2026-10-04; doc-format 2026-08-26; credit; property)",

    concepts: [
        Domain,
        MasterData, Definition, OntologyLayer, Operations,
        MetaEngineering, DataEngineering, BusinessSystem,
        DocumentFormat, DocumentEngineering, NarrativeEngineering,
        CreditManagement, Credit, Payment,
        Property, BasicProperty, ClassifiedProperty, Id,
    ],

    // 主数据三层归属决议：定义/本体/运营是主数据的三层，各归其位
    has_a: [
        (MasterData, Definition),
        (MasterData, OntologyLayer),
        (MasterData, Operations),
    ],
    edges: [
        (Definition, MetaEngineering, OwnedBy),
        (OntologyLayer, BusinessSystem, OwnedBy),
        (Operations, DataEngineering, OwnedBy),
    ],

    // 文档格式归文档工程，与叙事工程有分界；元工程域/数据工程域是领域
    is_a: [
        (DocumentFormat, DocumentEngineering),
        (MetaEngineering, Domain),
        (DataEngineering, Domain),
        (CreditManagement, Domain),
        // 本体标准属性：id 等属基础组，基础/分类两组都是属性
        (Id, BasicProperty),
        (BasicProperty, Property),
        (ClassifiedProperty, Property),
    ],
    opposes: [
        (DocumentEngineering, NarrativeEngineering),
        (Credit, Payment),
    ],
}

// ---------------------------------------------------------------------------
// 盲底：运行前写死的四条预测
// ---------------------------------------------------------------------------

const EXPECTED_EDGE: [(&str, GalleryRulesConcept, GalleryRulesConcept, bool, &str); 3] = [
    (
        "P1",
        GalleryRulesConcept::Id,
        GalleryRulesConcept::Property,
        true,
        "is_a 两级链会物化出 Id→Property（继承闭包）",
    ),
    (
        "P2",
        GalleryRulesConcept::DocumentFormat,
        GalleryRulesConcept::NarrativeEngineering,
        false,
        "对立不沿 is_a 传递，文档格式→叙事工程不应物化",
    ),
    (
        "P3",
        GalleryRulesConcept::MasterData,
        GalleryRulesConcept::MetaEngineering,
        false,
        "has_a 与自定义归属边不复合，主数据→元工程不应物化",
    ),
];

fn edge_exists(from: GalleryRulesConcept, to: GalleryRulesConcept) -> bool {
    GalleryRulesCategory::morphisms()
        .iter()
        .any(|m| m.source() == from && m.target() == to)
}

fn name(c: GalleryRulesConcept) -> &'static str {
    pr4xis::category::Concept::name(&c)
}

/// 逐条验证一组公理，返回（条数，失败清单）——与 Ontology::validate
/// 同一套检查（范畴定律 + 结构公理目录），逐条打印便于人工裁决。
fn verify_all(items: Vec<Box<dyn Axiom>>, label: &str) -> (usize, Vec<String>) {
    let n = items.len();
    let mut failures = Vec::new();
    for ax in items {
        if let Err(c) = ax.verify() {
            failures.push(format!(
                "{label} {} 未通过：{}",
                ax.name().as_str(),
                c.meta().name.as_str()
            ));
        }
    }
    (n, failures)
}

fn main() {
    println!("盲测：文档明说的关系 → pr4xis 物化出的隐含关系");
    println!("判据：四条预测（P1–P3 关系 + P4 校验）全符才计为通过\n");

    let morphisms = GalleryRulesCategory::morphisms();
    println!("[物化] 共 {} 条关系：", morphisms.len());
    for m in &morphisms {
        println!(
            "  {} -[{:?}]-> {}",
            name(m.source()),
            m.kind(),
            name(m.target())
        );
    }

    println!();
    let mut matched = 0usize;
    for (id, from, to, expect_present, claim) in EXPECTED_EDGE {
        let actual = edge_exists(from, to);
        let ok = actual == expect_present;
        if ok {
            matched += 1;
        }
        println!(
            "[{id}] {}（{} → {}）预期{}，实际{}",
            claim,
            name(from),
            name(to),
            if expect_present {
                "产生"
            } else {
                "不产生"
            },
            if actual { "产生" } else { "不产生" }
        );
        println!(
            "     {}",
            if ok {
                "符合"
            } else {
                "不符——需人工裁决"
            }
        );
    }

    // P4：范畴定律 + 结构公理目录
    let (law_n, mut failures) =
        verify_all(category_law_axioms::<GalleryRulesCategory>(), "范畴定律");
    let (struct_n, f2) = verify_all(
        GalleryRulesOntology::generated_structural_axioms(),
        "结构公理",
    );
    failures.extend(f2);
    let (domain_n, f3) = verify_all(GalleryRulesOntology::generated_domain_axioms(), "领域公理");
    failures.extend(f3);
    let p4_ok = failures.is_empty();
    if p4_ok {
        matched += 1;
    }
    println!(
        "[P4] 范畴定律 + 结构公理：检查 {} 条（定律 {law_n} + 结构 {struct_n} + 领域 {domain_n}），失败 {} 条",
        law_n + struct_n + domain_n,
        failures.len()
    );
    for f in &failures {
        println!("     {f}");
    }

    println!("\n[盲测总成绩] 四条预测，符合 {} 条", matched);
    if matched == 4 {
        println!("结论：自动推导与声明一致，隐含关系可交给它推。");
    } else {
        println!("结论：存在不符项，自动部分按不可信处理，逐条裁决见上方输出。");
    }
}
