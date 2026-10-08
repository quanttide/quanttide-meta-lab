# reflect 开发指南

这套文档写给接手 reflect 的人。AI 写错代码好办，一跑就知道。麻烦的是另一种情形：AI 建了一个模型，跟你脑子里想的不是一回事；它能跑，看着也像样，但你看得出「不对」，说不出哪里不对，也说不清它是怎么走到那一步的。reflect 把这个不可见的过程记下来，让偏离点可查，并把修正固化成下一次动手前的约束。

一句话概括：反思就是把结论拆成前提，检验前提，把修正后的前提固化成规则。

## 工程分解与现状

按变更频率切，有一条稳定的缝，分成五个小项目加一次验收。前三个已经落地在 `src/cli/`。

存储负责三种节点两条边，以及一个 JSON 文件的读写，在 `src/store.rs`。它变更最少，其余都依赖它。

召回负责给一个事件找出相关规则，在 `src/recall.rs`。第一版用词面匹配，以后换嵌入或图查询时，存储与命令行都不动。

命令行负责五条命令，编排存储与召回，在 `src/main.rs`。它是壳。

反思规范管怎么想，不写代码，写在 `docs/dev-guide/skill.md`。

规格格式管动手前交给 AI 的模板，不写代码，写在 `spec.md`。

验收要把五件合起来，放到真实工程环境跑一次，还没做。见 `eval.md`。

## 怎么跑

```bash
cargo build
cargo test
cargo run --example ip_takeover
cargo run --example ai_model_drift
cargo clippy --all-targets -- -D warnings
```

五条命令的用法在 `docs/dev-guide/cli.md`。

## 依赖与顺序

依赖单向：存储 ← 召回 ← 命令行。反思规范与规格格式不依赖代码，只依赖存储约定的记法。验收依赖全部。

动手顺序按风险，不按层次。整个项目最不确定的一环是召得回——规则记下来了，下次动手前到底能不能被送到面前。所以先验的是它：`tests/cli.rs` 里 `day_one_to_day_thirty_ends_with_a_hit` 从 Day 1 的偏离固化出规则，Day 30 用 `check` 把它命中，这一环成立之后才补记录侧的其余命令。

## 各篇

本目录放 reflect 这个模块的建模：`index.md`（本文）、`recall.md`、`spec.md`、`eval.md`。

上一级 `docs/dev-guide/` 放工具本身：`cli.md`（命令壳）、`storage.md`（存储机制）、`skill.md`（反思规范）。

设计草稿在仓库根的 `docs/dev-guide/`：`reflect.md` 是设计正本，`reflect_spec.md` 是规格格式，`reflect_test.md` 是开发场景的验证用例。当前任务在 `src/cli/TODO.md`。
