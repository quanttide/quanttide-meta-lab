# TODO

我计划把它放到真实的工程环境之下去看一看，看看「工作云」这个平台它是如何跟我的建模偏差掉的。首先要把这个测试样例重新设计、写一遍，然后再实行。

## 1. 重新设计测试样例

`docs/dev-guide/reflect_test.md` 现在的样例是「任务提醒」：一句话需求，AI 建出 Task/Schedule/Notification/Channel/Template/RetryPolicy 一整套。换成工作云平台上的真实需求，按 `docs/dev-guide/reflect_spec.md` 的格式重写：

- 代码模块规格：intent、scope、vocabulary（含 NOT）、invariants、interface
- 测试模块规格：covers、scenarios、must_not
- 规格里要写清「我本来想要什么」，否则事后判不出哪一步偏了

## 2. 写 reflect CLI 原型

按 `docs/dev-guide/reflect.md` 的最小闭环：

- 五个命令：`reflect add` / `dep` / `test` / `commit` / `check`
- 三种节点 Claim / Premise / Rule，两条边 depends_on / revises
- 存储先用单个 JSON 文件，不上图数据库
- 先只做拆解（动作 1），跑通再加检验、固化

## 3. 在真实环境里跑

- 把规格交给 AI 实现
- `reflect add` / `dep` 记下结论（AI 实现符合意图）及它依赖的前提
- `reflect test` 找反例：实现里哪部分是我没要的
- `reflect commit` 固化成规则

## 4. 记下偏差

按 `docs/dev-guide/reflect_test.md` 的三类断点归类：

- 词汇断点：我说的词 ≠ AI 理解的词
- 规模断点：没说复杂度上限，AI 往完备走
- 边界断点：没说不要什么，AI 就全都要

产出是下次动手前的约束。
