//! 把 `docs/dev-guide/reflect.md` 的「被收编对 IP 不利」走一遍。
//!
//! 一次偏离固化成一条规则，下次遇到同类事件时 check 把它召回来。

use reflect::recall;
use reflect::store::Store;

fn main() {
    let mut store = Store::default();

    // 反思的产物：把可疑前提改写成一条规则，连同它修正的前提记下来。
    store.commit_rule(
        "收编是否有利，取决于是否要求闭嘴",
        &["收编=失去独立性".to_string()],
    );

    // 下次遇到同类事件，check 主动提醒。
    let event = "要不要进入体系、被收编、谈合作";
    println!("事件：{event}");
    for hit in recall::recall(&store, event) {
        println!("  命中（{:.0}%）：{}", hit.score * 100.0, hit.rule.text);
        println!("  词：{}", hit.matched.join("、"));
    }
}
