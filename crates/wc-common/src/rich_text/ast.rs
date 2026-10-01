//! 统一富文本 AST（抽象语法树）
//!
//! 所有来源的标记语法（Markdown、`<think>`、`<tool_call>`、diff 等）
//! 归一化后都变成这棵 AST。渲染层只认 AST，不认原始语法。
//!
//! 节点类型覆盖市面上常见 AI 软件的全部特殊标记，详见设计文档 v3 第 5.3 节。

use serde::{Deserialize, Serialize};

/// 富文本文档 = AST 节点数组
pub type RichTextDoc = Vec<AstNode>;

/// 统一 AST 节点
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AstNode {
    // ── 行内 ──────────────────────────────────────
    /// 普通文本
    Text { content: String },
    /// 粗体
    Bold { children: Vec<AstNode> },
    /// 斜体
    Italic { children: Vec<AstNode> },
    /// 删除线
    Strikethrough { children: Vec<AstNode> },
    /// 高亮（==text==）
    Highlight { children: Vec<AstNode> },
    /// 行内代码
    CodeInline { content: String },
    /// 行内数学公式（$...$）
    MathInline { content: String },
    /// 链接
    Link {
        url: String,
        children: Vec<AstNode>,
    },
    /// 图片
    Image { url: String, alt: String },
    /// 安全 HTML（白名单标签）
    HtmlSafe { html: String },

    // ── 块级 ──────────────────────────────────────
    /// 标题
    Heading {
        level: u8, // 1-6
        children: Vec<AstNode>,
    },
    /// 段落
    Paragraph { children: Vec<AstNode> },
    /// 引用
    Quote { children: Vec<AstNode> },
    /// 分割线
    Hr,
    /// 列表
    List {
        ordered: bool,
        items: Vec<ListItem>,
    },
    /// 任务列表
    TaskList { items: Vec<TaskItem> },
    /// 表格
    Table {
        header: Vec<TableCell>,
        rows: Vec<Vec<TableCell>>,
    },

    // ── 代码与命令 ─────────────────────────────────
    /// 通用代码块
    CodeBlock {
        language: Option<String>,
        content: String,
        filename: Option<String>,
    },
    /// 终端命令（bash/shell 代码块且行首是 $/>/PS>）
    TerminalCommand {
        content: String,
        language: String,
    },
    /// 纯文本块（```plain / ```text，不解析 Markdown）
    PlainText { content: String },
    /// 文件修改（diff / SEARCH-REPLACE）
    FileEdit {
        filename: Option<String>,
        diff: String,
    },
    /// Mermaid 图表
    Mermaid { content: String },
    /// 块级数学公式（$$...$$）
    MathBlock { content: String },

    // ── AI 特殊标记 ────────────────────────────────
    /// 思考/推理过程（<think>/<reasoning> 等），默认折叠
    Thinking {
        content: String,
        collapsed: bool,
    },
    /// 工具/函数调用（<tool_call>/<function_calls>/Anthropic XML）
    ToolCall {
        name: String,
        arguments: Option<String>,
        result: Option<String>,
        status: ToolCallStatus,
    },
    /// 通用折叠块（<details>/<collapse>）
    Collapsible {
        summary: String,
        children: Vec<AstNode>,
    },
}

/// 列表项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    pub children: Vec<AstNode>,
    pub sub_list: Option<Box<AstNode>>,
}

/// 任务项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskItem {
    pub checked: bool,
    pub children: Vec<AstNode>,
}

/// 表格单元格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub children: Vec<AstNode>,
}

/// 工具调用状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCallStatus {
    /// 待执行
    Pending,
    /// 执行中
    Running,
    /// 成功
    Success,
    /// 失败
    Failed,
}

impl AstNode {
    /// 节点是否为块级（渲染时独占一行）
    pub fn is_block(&self) -> bool {
        matches!(
            self,
            AstNode::Heading { .. }
                | AstNode::Paragraph { .. }
                | AstNode::Quote { .. }
                | AstNode::Hr
                | AstNode::List { .. }
                | AstNode::TaskList { .. }
                | AstNode::Table { .. }
                | AstNode::CodeBlock { .. }
                | AstNode::TerminalCommand { .. }
                | AstNode::PlainText { .. }
                | AstNode::FileEdit { .. }
                | AstNode::Mermaid { .. }
                | AstNode::MathBlock { .. }
                | AstNode::Thinking { .. }
                | AstNode::ToolCall { .. }
                | AstNode::Collapsible { .. }
        )
    }

    /// 节点是否为行内
    pub fn is_inline(&self) -> bool {
        !self.is_block()
    }
}
