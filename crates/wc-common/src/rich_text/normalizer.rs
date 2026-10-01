//! 归一化器 — 将各种来源的原始文本转换为统一富文本 AST
//!
//! 核心逻辑：
//! 1. 根据 `app_id` 从 [`AppRegistry`] 查找该 APP 专属的规则集
//! 2. 找不到时使用 `generic` 通用规则集
//! 3. 按优先级从高到低依次执行规则（管道式：前一条输出作为后一条输入）
//! 4. 最终输出 [`RichTextDoc`]

use crate::rich_text::app_registry::AppRegistry;
use crate::rich_text::ast::{AstNode, RichTextDoc};

/// 归一化器
pub struct Normalizer {
    registry: &'static AppRegistry,
}

impl Default for Normalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Normalizer {
    /// 创建归一化器（使用全局 APP 注册表）
    pub fn new() -> Self {
        Self {
            registry: AppRegistry::global(),
        }
    }

    /// 将原始文本归一化为统一富文本 AST
    ///
    /// # 参数
    /// - `app_id`: 来源 APP 标识（如 "qwen"、"claude"），匹配不到走 generic
    /// - `input`: 原始文本（可能包含各种标记语法）
    pub fn normalize(&self, app_id: &str, input: &str) -> RichTextDoc {
        let rule_set = self.registry.get(app_id);

        if tracing::enabled!(tracing::Level::DEBUG) {
            tracing::debug!(
                "Normalizing input for app='{}' (matched: '{}'), rules={}",
                app_id,
                rule_set.info.id,
                rule_set.rules().len()
            );
        }

        // 管道式执行：初始输入是整个文本作为一个 Text 节点
        // 每条规则接收当前节点列表的文本拼接，输出新的节点列表
        // 注意：第一版规则都是基于纯文本的，后续 Markdown 解析会改为基于 AST 的后处理
        let mut current_text = input.to_string();

        for rule in rule_set.rules() {
            let nodes = rule.normalize(&current_text);
            // 将节点重新拼接为文本，供下一条规则处理
            // （第一版规则都是文本级的，这样做是安全的）
            current_text = nodes_to_text(&nodes);
        }

        // 最终执行一次，得到 AST
        let mut result = Vec::new();
        for rule in rule_set.rules() {
            let nodes = rule.normalize(&current_text);
            // 只要有一条规则产出了非 Text 节点，就用它的结果
            if nodes.iter().any(|n| !matches!(n, AstNode::Text { .. })) {
                result = nodes;
                break;
            }
        }

        if result.is_empty() {
            result = vec![AstNode::Text {
                content: input.to_string(),
            }];
        }

        result
    }
}

/// 将 AST 节点列表拼接为纯文本（用于规则管道的中间传递）
fn nodes_to_text(nodes: &[AstNode]) -> String {
    let mut text = String::new();
    for node in nodes {
        match node {
            AstNode::Text { content } => text.push_str(content),
            AstNode::Thinking { content, .. } => {
                // 保留思考标签，让后续规则能重新识别
                text.push_str("<think>");
                text.push_str(content);
                text.push_str("</think>");
            }
            AstNode::ToolCall {
                name,
                arguments,
                ..
            } => {
                text.push_str("<tool_call>");
                if let Some(args) = arguments {
                    text.push_str(&format!(
                        r#"{{"name":"{}","arguments":{}}}"#,
                        name, args
                    ));
                }
                text.push_str("</tool_call>");
            }
            AstNode::CodeBlock {
                language,
                content,
                ..
            } => {
                text.push_str("```");
                if let Some(lang) = language {
                    text.push_str(lang);
                }
                text.push('\n');
                text.push_str(content);
                text.push_str("\n```");
            }
            AstNode::FileEdit { diff, .. } => {
                text.push_str("```diff\n");
                text.push_str(diff);
                text.push_str("\n```");
            }
            AstNode::PlainText { content } => {
                text.push_str("```plain\n");
                text.push_str(content);
                text.push_str("\n```");
            }
            AstNode::Mermaid { content } => {
                text.push_str("```mermaid\n");
                text.push_str(content);
                text.push_str("\n```");
            }
            AstNode::TerminalCommand { content, language } => {
                text.push_str("```");
                text.push_str(language);
                text.push('\n');
                text.push_str(content);
                text.push_str("\n```");
            }
            // 其他节点暂时转为文本占位
            _ => {}
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_think_tag_qwen() {
        let normalizer = Normalizer::new();
        let input = "<think>这是思考过程</think>这是回答";
        let doc = normalizer.normalize("qwen", input);
        assert!(doc.iter().any(|n| matches!(n, AstNode::Thinking { .. })));
    }

    #[test]
    fn test_tool_call_openai() {
        let normalizer = Normalizer::new();
        let input = r#"<tool_call>{"name":"get_weather","arguments":{"city":"上海"}}</tool_call>"#;
        let doc = normalizer.normalize("openai", input);
        assert!(doc.iter().any(|n| matches!(n, AstNode::ToolCall { .. })));
    }

    #[test]
    fn test_diff_code_block() {
        let normalizer = Normalizer::new();
        let input = "```diff\n--- a/main.rs\n+++ b/main.rs\n@@ -1,3 +1,3 @@\n-fn old() {}\n+fn new() {}\n```";
        let doc = normalizer.normalize("generic", input);
        assert!(doc.iter().any(|n| matches!(n, AstNode::FileEdit { .. })));
    }

    #[test]
    fn test_unknown_app_falls_back_to_generic() {
        let normalizer = Normalizer::new();
        let input = "<think>测试</think>";
        let doc = normalizer.normalize("nonexistent_app_xyz", input);
        assert!(doc.iter().any(|n| matches!(n, AstNode::Thinking { .. })));
    }

    #[test]
    fn test_plain_text_passthrough() {
        let normalizer = Normalizer::new();
        let input = "这是一段普通文本，没有任何标记";
        let doc = normalizer.normalize("generic", input);
        assert_eq!(doc.len(), 1);
        assert!(matches!(&doc[0], AstNode::Text { content } if content == input));
    }
}
