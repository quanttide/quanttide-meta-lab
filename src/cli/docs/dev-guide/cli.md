# 命令行

命令行负责让你用得顺手。它是整套东西的外壳，代码在 `src/main.rs`，篇幅的九成是 `clap` 的声明。

## 五条命令

```text
reflect add <结论>
reflect dep <结论> <前提>
reflect test <前提> [--counterexample <反例>]
reflect commit <规则> [--revises <前提>]...
reflect check [事件]
```

`add` 记结论，`dep` 记依赖，`test` 录反例并标可疑，`commit` 固化成规则，`check` 检索相关规则。三个动作加一次检索，正好五条。

`--store <文件>` 是全局参数，默认 `reflect.json`，可以写在子命令前后。

`check` 不给事件就列出全部规则，给了事件就按命中比例排序，连同命中了哪些词一起打出来。

## 为什么保持五条

命令一一对应动作，是为了敲命令时不用想这一步该用哪条。多一条就多一次判断，判断一多，记的人就懒得记，记不下来后面全废。所以不合并、不增设。

## 是壳

`main.rs` 只做编排：解析参数、调存储、调召回、打结果。它不定数据格式，那是 `src/store.rs` 的事；不做匹配判断，那是 `src/recall.rs` 的事。这样以后换交互方式——加个 TUI、套一层别的接口——前两个模块都不用动。

命令行自己产生的默认只有一处：`commit` 不给 `--revises` 时修正全部可疑前提。它写在 `Store::commit_rule` 里，不写在参数解析里。

## 验收

`tests/cli.rs` 用 `CARGO_BIN_EXE_reflect` 直接跑二进制，覆盖五条命令的落盘结果与输出。要手工过一遍，在空目录里依次敲上面五条即可，存档就是 `reflect.json`。
