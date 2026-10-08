//! 把 `docs/dev-guide/reflect_test.md` 的 Day 1 → Day 30 走一遍。
//!
//! 第一次偏离固化成规则，第二次动手前 check 把它召回来。

use reflect::recall;
use reflect::store::Store;

fn main() {
    let mut store = Store::default();
    let report = |store: &Store, event: &str| {
        println!("事件：{event}");
        let hits = recall::recall(store, event);
        if hits.is_empty() {
            println!("  没有命中规则。");
        }
        for hit in hits {
            println!("  命中（{:.0}%）：{}", hit.score * 100.0, hit.rule.text);
        }
    };

    // Day 1：一句话需求，AI 建出一整套通知系统。
    store.commit_rule(
        "让AI写功能前先给复杂度预算：最多几个文件、几个概念",
        &["AI会把复杂度和需求规模匹配".to_string()],
    );
    store.commit_rule(
        "需求里的抽象词必须先钉死边界",
        &["AI理解的提醒=我的提醒".to_string()],
    );

    // Day 30：又要让 AI 写「用户积分」功能。
    report(&store, "让AI写用户积分功能，抽象词和边界要说清");

    // 无关事件不该被召回。
    report(&store, "数据库连接池配置");
}
