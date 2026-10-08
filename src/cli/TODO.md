# TODO

我计划把它放到真实的工程环境之下去看一看，看看「工作云」这个平台它是如何跟我的建模偏差掉的。首先要把这个测试样例重新设计、写一遍，然后再实行。

## 1. 重新设计测试样例

`docs/dev-guide/reflect_test.md` 现在的样例是「任务提醒」：一句话需求，AI 建出 Task/Schedule/Notification/Channel/Template/RetryPolicy 一整套。换成工作云平台上的真实需求，按 `docs/dev-guide/reflect_spec.md` 的格式重写：

- 代码模块规格：intent、scope、vocabulary（含 NOT）、invariants、interface
- 测试模块规格：covers、scenarios、must_not
- 规格里要写清「我本来想要什么」，否则事后判不出哪一步偏了

## 2. 在真实环境里跑

- 把规格交给 AI 实现
- 偏离固化：`reflect commit <规则> --revises <前提>`
- 下次动手前 `reflect check <事件>`

## 3. 记下偏差

按 `docs/dev-guide/reflect_test.md` 的三类断点归类：

- 词汇断点：我说的词 ≠ AI 理解的词
- 规模断点：没说复杂度上限，AI 往完备走
- 边界断点：没说不要什么，AI 就全都要

产出是下次动手前的约束。

## 4. 没进程序的部分

结论、前提、反例、可疑标记这四样，现在只落在纸上与规格里，程序不维护它们：程序只收规则、发规则。真跑一轮下来如果发现没有它们就写不出规则，再考虑收回来。
