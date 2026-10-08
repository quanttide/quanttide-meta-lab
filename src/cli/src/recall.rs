//! 召回：把一个事件匹配到相关规则。
//!
//! 第一版用词面匹配：事件里的词出现在规则文本或它所修正的前提里，就算命中。
//! 匹配算法是最容易换掉的一块，换的时候 [`Store`] 与命令行都不该动。

use std::cmp::Ordering;
use std::collections::HashSet;

use crate::store::{Node, Store};

/// 一条命中的规则。
pub struct Hit<'a> {
    pub rule: &'a Node,
    /// 命中词占事件词的比例。
    pub score: f64,
    /// 命中了哪些词。
    pub matched: Vec<String>,
}

/// 把一个事件匹配到相关规则，按命中比例从高到低。
pub fn recall<'a>(store: &'a Store, event: &str) -> Vec<Hit<'a>> {
    let query: HashSet<String> = tokens(event).into_iter().collect();
    if query.is_empty() {
        return Vec::new();
    }

    let mut hits: Vec<Hit<'a>> = store
        .rules()
        .into_iter()
        .filter_map(|rule| match_one(store, rule, &query))
        .collect();

    hits.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.rule.text.cmp(&right.rule.text))
    });
    hits
}

/// 一条规则命中不了事件就返回 `None`。
///
/// 找的是一句话里的词，不是语义：规则文本加上它所修正的前提，去碰事件的词。
fn match_one<'a>(store: &Store, rule: &'a Node, query: &HashSet<String>) -> Option<Hit<'a>> {
    let mut haystack: HashSet<String> = tokens(&rule.text).into_iter().collect();
    for premise in store.revised_premises(&rule.text) {
        haystack.extend(tokens(premise));
    }

    let mut matched: Vec<String> = haystack.intersection(query).cloned().collect();
    if matched.is_empty() {
        return None;
    }
    matched.sort();

    Some(Hit {
        rule,
        score: matched.len() as f64 / query.len() as f64,
        matched,
    })
}

/// 切词：ASCII 字母数字按词切，连续汉字切成二元组，其余字符丢掉。
pub fn tokens(text: &str) -> Vec<String> {
    let lowered = text.to_lowercase();
    let chars: Vec<char> = lowered.chars().collect();
    let mut out = Vec::new();
    let mut index = 0;

    while index < chars.len() {
        if chars[index].is_ascii_alphanumeric() {
            let start = index;
            while index < chars.len() && chars[index].is_ascii_alphanumeric() {
                index += 1;
            }
            out.push(chars[start..index].iter().collect());
        } else if is_cjk(chars[index]) {
            let start = index;
            while index < chars.len() && is_cjk(chars[index]) {
                index += 1;
            }
            let run = &chars[start..index];
            if run.len() == 1 {
                out.push(run[0].to_string());
            } else {
                for pair in run.windows(2) {
                    out.push(pair.iter().collect());
                }
            }
        } else {
            index += 1;
        }
    }

    out.sort();
    out.dedup();
    out
}

fn is_cjk(c: char) -> bool {
    matches!(c as u32, 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_words_are_split_on_non_alphanumeric() {
        assert_eq!(
            tokens("set_reminder(task)"),
            vec!["reminder", "set", "task"]
        );
    }

    #[test]
    fn cjk_runs_become_bigrams() {
        assert_eq!(tokens("抽象词"), vec!["抽象", "象词"]);
    }

    #[test]
    fn single_cjk_char_stays_whole() {
        assert_eq!(tokens("词"), vec!["词"]);
    }
}
