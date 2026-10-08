//! 存储：一份规则表，以及一个 JSON 文件的读写。
//!
//! 存储不判断哪条规则该被召回，那是 [`crate::recall`] 的事；不决定什么时候写，
//! 那是命令行的事。它只认记法。

use std::error::Error;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// 一条规则：修正后的前提，写成可执行的句子。
///
/// 文本就是它的身份，同一条规则不会因为再固化一次变成两条。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    pub text: String,
    /// 这条规则修正的前提。召回时连同规则文本一起算作它的词。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revises: Vec<String>,
}

/// 规则表：一个 JSON 文件的内容。
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Store {
    #[serde(default)]
    pub rules: Vec<Rule>,
}

impl Store {
    /// 读存档；文件不存在时给一个空表。
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

    /// 固化为规则。同一条规则再固化一次只是把新前提并进去，重复的不写第二遍。
    pub fn commit_rule(&mut self, rule: &str, revises: &[String]) {
        let entry = match self.rules.iter_mut().find(|entry| entry.text == rule) {
            Some(entry) => entry,
            None => {
                self.rules.push(Rule {
                    text: rule.to_string(),
                    revises: Vec::new(),
                });
                self.rules.last_mut().expect("just pushed")
            }
        };

        for premise in revises {
            if !entry.revises.contains(premise) {
                entry.revises.push(premise.clone());
            }
        }
    }

    /// 全部规则。
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }
}
