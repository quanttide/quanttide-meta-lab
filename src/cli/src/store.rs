//! 存储：三种节点、两条边，以及一个 JSON 文件的读写。
//!
//! 存储不判断哪条规则该被召回，那是 [`crate::recall`] 的事；不决定什么时候写，
//! 那是命令行的事。它只认记法。

use std::error::Error;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// 节点：结论、前提、规则。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// 当时以为对的东西。
    Claim,
    /// 结论压着、却没写出来的东西。
    Premise,
    /// 修正后的前提，写成可执行的句子。
    Rule,
}

/// 一个节点。文本就是它的身份，(kind, text) 唯一。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub kind: Kind,
    pub text: String,
    /// 前提被找到的反例。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counterexample: Option<String>,
    /// 前提是否已被反例打中。
    #[serde(default, skip_serializing_if = "is_false")]
    pub suspect: bool,
}

/// 边：依赖、修正。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// 结论依赖前提。
    DependsOn,
    /// 规则修正前提。
    Revises,
}

/// 一条边。`from` 与 `to` 都是节点文本，方向由 [`EdgeKind`] 定。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub kind: EdgeKind,
    pub from: String,
    pub to: String,
}

/// 整个存档：一个 JSON 文件的内容。
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Store {
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub edges: Vec<Edge>,
}

impl Store {
    /// 读存档；文件不存在时给一个空存档。
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
    }

    /// 写存档；父目录不存在时先建。
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent()
            && !dir.as_os_str().is_empty()
        {
            fs::create_dir_all(dir)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    /// 记下结论。
    pub fn add_claim(&mut self, claim: &str) {
        self.ensure(Kind::Claim, claim);
    }

    /// 记下依赖关系：结论依赖前提。两边缺哪个补哪个。
    pub fn add_dependency(&mut self, claim: &str, premise: &str) {
        self.ensure(Kind::Claim, claim);
        self.ensure(Kind::Premise, premise);
        self.link(EdgeKind::DependsOn, claim, premise);
    }

    /// 检验前提：录入反例并标为可疑。不给反例就只标为可疑。
    pub fn add_counterexample(&mut self, premise: &str, counterexample: Option<&str>) {
        let node = self.ensure(Kind::Premise, premise);
        if let Some(counterexample) = counterexample {
            node.counterexample = Some(counterexample.to_string());
        }
        node.suspect = true;
    }

    /// 固化为规则。`revises` 为空时修正当前全部可疑前提，返回被修正的前提。
    pub fn commit_rule(&mut self, rule: &str, revises: &[String]) -> Vec<String> {
        self.ensure(Kind::Rule, rule);
        let targets = if revises.is_empty() {
            self.suspect_premises()
        } else {
            revises.to_vec()
        };
        for target in &targets {
            self.ensure(Kind::Premise, target);
            self.link(EdgeKind::Revises, rule, target);
        }
        targets
    }

    /// 当前全部可疑前提的文本。
    pub fn suspect_premises(&self) -> Vec<String> {
        self.nodes_of_kind(Kind::Premise)
            .filter(|node| node.suspect)
            .map(|node| node.text.clone())
            .collect()
    }

    /// 全部规则。
    pub fn rules(&self) -> Vec<&Node> {
        self.nodes_of_kind(Kind::Rule).collect()
    }

    /// 规则修正了哪些前提。
    pub fn revised_premises(&self, rule: &str) -> Vec<&str> {
        self.edges_of_kind(EdgeKind::Revises)
            .filter(|edge| edge.from == rule)
            .map(|edge| edge.to.as_str())
            .collect()
    }

    /// 前提被哪些结论依赖。
    pub fn dependent_claims(&self, premise: &str) -> Vec<&str> {
        self.edges_of_kind(EdgeKind::DependsOn)
            .filter(|edge| edge.to == premise)
            .map(|edge| edge.from.as_str())
            .collect()
    }

    fn nodes_of_kind(&self, kind: Kind) -> impl Iterator<Item = &Node> {
        self.nodes.iter().filter(move |node| node.kind == kind)
    }

    fn edges_of_kind(&self, kind: EdgeKind) -> impl Iterator<Item = &Edge> {
        self.edges.iter().filter(move |edge| edge.kind == kind)
    }

    /// 取节点，没有就建一个空的。
    fn ensure(&mut self, kind: Kind, text: &str) -> &mut Node {
        let index = self
            .nodes
            .iter()
            .position(|node| node.kind == kind && node.text == text)
            .unwrap_or_else(|| {
                self.nodes.push(Node {
                    kind,
                    text: text.to_string(),
                    counterexample: None,
                    suspect: false,
                });
                self.nodes.len() - 1
            });
        &mut self.nodes[index]
    }

    /// 连一条边；已有的不连第二遍。
    fn link(&mut self, kind: EdgeKind, from: &str, to: &str) {
        let edge = Edge {
            kind,
            from: from.to_string(),
            to: to.to_string(),
        };
        if !self.edges.contains(&edge) {
            self.edges.push(edge);
        }
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}
