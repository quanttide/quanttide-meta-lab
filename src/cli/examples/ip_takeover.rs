//! 把 `docs/dev-guide/reflect.md` 的「被收编对 IP 不利」走一遍。
//!
//! 三个动作依次跑完，最后让 check 在同类事件上把它召回来。

use reflect::recall;
use reflect::store::Store;

fn main() {
    let mut store = Store::default();

    // 1. 拆解：结论还原成它依赖的前提。
    store.add_claim("被收编对IP不利");
    store.add_dependency("被收编对IP不利", "收编=失去独立性");
    store.add_dependency("被收编对IP不利", "失去独立性=信任崩塌");

    // 2. 检验：找反例，标为可疑。
    store.add_counterexample(
        "收编=失去独立性",
        Some("分析师转顾问，信息更准、公信力没掉"),
    );
    println!("可疑前提：{}", store.suspect_premises().join("；"));

    // 3. 固化：把可疑前提改写成规则。
    let revised = store.commit_rule("收编是否有利，取决于是否要求闭嘴", &[]);
    println!("固化为规则，修正的前提：{}", revised.join("；"));

    // 下次遇到同类事件，check 主动提醒。
    let event = "要不要进入体系、被收编、谈合作";
    println!("事件：{event}");
    for hit in recall::recall(&store, event) {
        println!("  命中（{:.0}%）：{}", hit.score * 100.0, hit.rule.text);
        println!("  词：{}", hit.matched.join("、"));
    }
}
