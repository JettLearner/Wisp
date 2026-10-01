//! APP 专属规则注册表
//!
//! 核心原则（用户明确要求）：**所有 tag 先匹配特定 APP，匹配不到库里的 APP 再走通用库。**
//!
//! 每个 APP 有自己的 [`AppRuleSet`]，包含该 APP 专属的归一化规则。
//! 归一化器根据消息的 `app_id` 查找对应规则集，找不到时使用 `generic`。
//!
//! ## Tag 来源可靠性
//! 每个内置 APP 的规则均标注官方文档来源。未经验证的标记不纳入第一版。

use crate::rich_text::ast::AstNode;
use std::collections::HashMap;
use std::sync::OnceLock;

// ── APP 元信息 ──────────────────────────────────────

/// APP 元信息
#[derive(Debug, Clone)]
pub struct AppInfo {
    /// 唯一标识（适配脚本的 app_id）
    pub id: &'static str,
    /// 显示名称
    pub name: &'static str,
    /// 官方文档 URL（tag 语法的来源依据）
    pub doc_url: &'static str,
    /// 该 APP 特有的标记语法说明
    pub markup_notes: &'static str,
}

// ── 归一化规则 ──────────────────────────────────────

/// 单条归一化规则
///
/// 规则按优先级从高到低执行，前一条的输出作为后一条的输入（管道式）。
/// 每条规则只处理自己认识的标记，不认识的内容原样返回。
pub trait NormalizeRule: Send + Sync {
    /// 规则名称（用于日志和调试）
    fn name(&self) -> &str;
    /// 优先级（0-255，越大越先执行）
    fn priority(&self) -> u8;
    /// 对输入文本进行归一化，输出 AST 节点数组
    fn normalize(&self, input: &str) -> Vec<AstNode>;
}

/// 用函数包装的简易规则（方便内置规则快速定义）
pub struct FnRule {
    name: &'static str,
    priority: u8,
    func: fn(&str) -> Vec<AstNode>,
}

impl FnRule {
    pub const fn new(name: &'static str, priority: u8, func: fn(&str) -> Vec<AstNode>) -> Self {
        Self { name, priority, func }
    }
}

impl NormalizeRule for FnRule {
    fn name(&self) -> &str {
        self.name
    }
    fn priority(&self) -> u8 {
        self.priority
    }
    fn normalize(&self, input: &str) -> Vec<AstNode> {
        (self.func)(input)
    }
}

// ── APP 规则集 ──────────────────────────────────────

/// 一个 APP 的专属规则集
pub struct AppRuleSet {
    pub info: AppInfo,
    rules: Vec<Box<dyn NormalizeRule>>,
}

impl AppRuleSet {
    pub fn new(info: AppInfo) -> Self {
        Self {
            info,
            rules: Vec::new(),
        }
    }

    /// 添加规则（自动按优先级排序）
    pub fn add_rule(&mut self, rule: Box<dyn NormalizeRule>) {
        self.rules.push(rule);
        self.rules.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// 获取所有规则（按优先级降序）
    pub fn rules(&self) -> &[Box<dyn NormalizeRule>] {
        &self.rules
    }
}

// ── 内置 APP 定义 ───────────────────────────────────

/// OpenAI / ChatGPT
/// 来源：https://platform.openai.com/docs/guides/function-calling
/// 标记：标准 Markdown；函数调用在 API 层为结构化 tool_calls，透传场景可能出现 <tool_call>JSON</tool_call>
const APP_OPENAI: AppInfo = AppInfo {
    id: "openai",
    name: "OpenAI / ChatGPT",
    doc_url: "https://platform.openai.com/docs/guides/function-calling",
    markup_notes: "标准 Markdown；函数调用为结构化字段，透传场景识别 <tool_call> 标签",
};

/// Anthropic / Claude
/// 来源：https://docs.anthropic.com/en/docs/build-with-claude/tool-use
/// 标记：标准 Markdown；工具使用 XML 格式 <tool_name>...</tool_name>；常用 ```diff 展示文件修改
const APP_CLAUDE: AppInfo = AppInfo {
    id: "claude",
    name: "Anthropic / Claude",
    doc_url: "https://docs.anthropic.com/en/docs/build-with-claude/tool-use",
    markup_notes: "XML 工具调用标签 <tool_name>；常用 ```diff 代码块展示文件修改",
};

/// Google / Gemini
/// 来源：https://ai.google.dev/gemini-api/docs/function-calling
/// 标记：标准 Markdown；函数调用为结构化 functionCall 字段
const APP_GEMINI: AppInfo = AppInfo {
    id: "gemini",
    name: "Google / Gemini",
    doc_url: "https://ai.google.dev/gemini-api/docs/function-calling",
    markup_notes: "标准 Markdown；函数调用为结构化字段，无特殊文本标记",
};

/// Qwen3（阿里）
/// 来源：https://qwen.readthedocs.io/ 及 HuggingFace Qwen3 模型卡
/// 标记：<think>...</think> 思考标签；标准 Markdown
const APP_QWEN: AppInfo = AppInfo {
    id: "qwen",
    name: "Qwen / 通义千问",
    doc_url: "https://qwen.readthedocs.io/",
    markup_notes: "<think> 思考标签；标准 Markdown",
};

/// DeepSeek
/// 来源：https://api-docs.deepseek.com/ 及 DeepSeek-R1 模型卡
/// 标记：<think>...</think> 思考标签；标准 Markdown
const APP_DEEPSEEK: AppInfo = AppInfo {
    id: "deepseek",
    name: "DeepSeek",
    doc_url: "https://api-docs.deepseek.com/",
    markup_notes: "<think> 思考标签（推理模型）；标准 Markdown",
};

/// Cursor
/// 来源：https://docs.cursor.com/context/agent
/// 标记：标准 Markdown；工具调用内嵌标记；常用 ```diff 和 ```bash
const APP_CURSOR: AppInfo = AppInfo {
    id: "cursor",
    name: "Cursor",
    doc_url: "https://docs.cursor.com/context/agent",
    markup_notes: "工具调用内嵌标记；```diff 文件修改；```bash 终端命令",
};

/// Ollama（本地模型运行时）
/// 来源：https://github.com/ollama/ollama/blob/main/docs/api.md
/// 标记：Ollama 本身不添加标记，标记取决于运行的模型；常见 <think>（Qwen/DeepSeek 系列）
const APP_OLLAMA: AppInfo = AppInfo {
    id: "ollama",
    name: "Ollama",
    doc_url: "https://github.com/ollama/ollama/blob/main/docs/api.md",
    markup_notes: "标记取决于底层模型；常见 <think>（Qwen/DeepSeek 系列）",
};

/// 通用兜底（匹配不到 APP 时使用）
/// 覆盖市面上最常见的所有标记语法
const APP_GENERIC: AppInfo = AppInfo {
    id: "generic",
    name: "通用（兜底）",
    doc_url: "",
    markup_notes: "覆盖所有常见标记：思考标签、工具调用、diff、折叠、数学公式、Mermaid、终端命令",
};

// ── 基础规则实现（第一版） ───────────────────────────
// 完整 Markdown 解析将在后续阶段引入 pulldown-cmark。
// 第一版先实现 AI 特有的非 Markdown 标记识别。

/// 思考标签规则：识别 <think>/<reasoning>/<thought>/<analysis>
fn rule_thinking_tags(input: &str) -> Vec<AstNode> {
    // 匹配常见思考标签对（不区分大小写，允许标签内有属性）
    let patterns = [
        (r"(?is)<think\b[^>]*>(.*?)</think>", "think"),
        (r"(?is)<reasoning\b[^>]*>(.*?)</reasoning>", "reasoning"),
        (r"(?is)<thought\b[^>]*>(.*?)</thought>", "thought"),
        (r"(?is)<analysis\b[^>]*>(.*?)</analysis>", "analysis"),
    ];

    let mut nodes = Vec::new();
    let mut last_end = 0;

    for (pattern, _tag) in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            for cap in re.captures_iter(input) {
                if let Some(full) = cap.get(0) {
                    if full.start() > last_end {
                        nodes.push(AstNode::Text {
                            content: input[last_end..full.start()].to_string(),
                        });
                    }
                    if let Some(content) = cap.get(1) {
                        nodes.push(AstNode::Thinking {
                            content: content.as_str().to_string(),
                            collapsed: true,
                        });
                    }
                    last_end = full.end();
                }
            }
        }
    }

    if last_end < input.len() {
        nodes.push(AstNode::Text {
            content: input[last_end..].to_string(),
        });
    }

    if nodes.is_empty() {
        vec![AstNode::Text {
            content: input.to_string(),
        }]
    } else {
        nodes
    }
}

/// 工具调用规则：识别 <tool_call>JSON</tool_call> 和 <function_calls>
fn rule_tool_call_tags(input: &str) -> Vec<AstNode> {
    let patterns = [
        r"(?is)<tool_call\b[^>]*>(.*?)</tool_call>",
        r"(?is)<function_calls\b[^>]*>(.*?)</function_calls>",
    ];

    let mut nodes = Vec::new();
    let mut last_end = 0;

    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            for cap in re.captures_iter(input) {
                if let Some(full) = cap.get(0) {
                    if full.start() > last_end {
                        nodes.push(AstNode::Text {
                            content: input[last_end..full.start()].to_string(),
                        });
                    }
                    if let Some(content) = cap.get(1) {
                        // 尝试解析 JSON 提取工具名
                        let (name, args) = parse_tool_call_json(content.as_str());
                        nodes.push(AstNode::ToolCall {
                            name,
                            arguments: args,
                            result: None,
                            status: crate::rich_text::ast::ToolCallStatus::Pending,
                        });
                    }
                    last_end = full.end();
                }
            }
        }
    }

    if last_end < input.len() {
        nodes.push(AstNode::Text {
            content: input[last_end..].to_string(),
        });
    }

    if nodes.is_empty() {
        vec![AstNode::Text {
            content: input.to_string(),
        }]
    } else {
        nodes
    }
}

/// 从工具调用 JSON 中提取名称和参数
fn parse_tool_call_json(json_str: &str) -> (String, Option<String>) {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str.trim()) {
        let name = value
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let args = value
            .get("arguments")
            .map(|v| v.to_string());
        (name, args)
    } else {
        ("unknown".to_string(), Some(json_str.to_string()))
    }
}

/// 代码块语言分流规则：识别 ```lang 并根据语言分流
fn rule_code_block_language(input: &str) -> Vec<AstNode> {
    let pattern = r"(?s)```(\w+)?\n?(.*?)```";
    let mut nodes = Vec::new();
    let mut last_end = 0;

    if let Ok(re) = regex::Regex::new(pattern) {
        for cap in re.captures_iter(input) {
            if let Some(full) = cap.get(0) {
                if full.start() > last_end {
                    nodes.push(AstNode::Text {
                        content: input[last_end..full.start()].to_string(),
                    });
                }
                let lang = cap.get(1).map(|m| m.as_str().to_string());
                let content = cap.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();

                let node = match lang.as_deref() {
                    Some("diff") | Some("udiff") | Some("patch") => AstNode::FileEdit {
                        filename: extract_filename_from_diff(&content),
                        diff: content,
                    },
                    Some("plain") | Some("text") | Some("txt") => AstNode::PlainText { content },
                    Some("mermaid") => AstNode::Mermaid { content },
                    Some("bash") | Some("shell") | Some("sh") | Some("zsh")
                    | Some("powershell") | Some("ps") | Some("cmd") => {
                        if looks_like_terminal_command(&content) {
                            AstNode::TerminalCommand {
                                content,
                                language: lang.unwrap(),
                            }
                        } else {
                            AstNode::CodeBlock {
                                language: lang,
                                content,
                                filename: None,
                            }
                        }
                    }
                    _ => AstNode::CodeBlock {
                        language: lang,
                        content,
                        filename: None,
                    },
                };
                nodes.push(node);
                last_end = full.end();
            }
        }
    }

    if last_end < input.len() {
        nodes.push(AstNode::Text {
            content: input[last_end..].to_string(),
        });
    }

    if nodes.is_empty() {
        vec![AstNode::Text {
            content: input.to_string(),
        }]
    } else {
        nodes
    }
}

/// 从 diff 内容中提取文件名（--- a/xxx 或 +++ b/xxx）
fn extract_filename_from_diff(diff: &str) -> Option<String> {
    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("+++ b/") {
            return Some(rest.trim().to_string());
        }
        if let Some(rest) = line.strip_prefix("--- a/") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

/// 判断 bash 代码块是否像终端命令（行首是 $/>/PS>/#）
fn looks_like_terminal_command(content: &str) -> bool {
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .any(|l| {
            let t = l.trim_start();
            t.starts_with("$ ") || t.starts_with("> ") || t.starts_with("PS>") || t.starts_with("# ")
        })
}

// ── 构建各 APP 的规则集 ─────────────────────────────

fn build_openai_rules() -> AppRuleSet {
    let mut set = AppRuleSet::new(APP_OPENAI);
    set.add_rule(Box::new(FnRule::new("code_block_language", 100, rule_code_block_language)));
    set.add_rule(Box::new(FnRule::new("tool_call_tags", 90, rule_tool_call_tags)));
    set.add_rule(Box::new(FnRule::new("thinking_tags", 80, rule_thinking_tags)));
    set
}

fn build_claude_rules() -> AppRuleSet {
    let mut set = AppRuleSet::new(APP_CLAUDE);
    set.add_rule(Box::new(FnRule::new("code_block_language", 100, rule_code_block_language)));
    // Claude 用 XML 工具标签，tool_call 规则也能覆盖部分
    set.add_rule(Box::new(FnRule::new("tool_call_tags", 90, rule_tool_call_tags)));
    set.add_rule(Box::new(FnRule::new("thinking_tags", 80, rule_thinking_tags)));
    set
}

fn build_gemini_rules() -> AppRuleSet {
    let mut set = AppRuleSet::new(APP_GEMINI);
    set.add_rule(Box::new(FnRule::new("code_block_language", 100, rule_code_block_language)));
    set.add_rule(Box::new(FnRule::new("thinking_tags", 80, rule_thinking_tags)));
    set
}

fn build_qwen_rules() -> AppRuleSet {
    let mut set = AppRuleSet::new(APP_QWEN);
    set.add_rule(Box::new(FnRule::new("code_block_language", 100, rule_code_block_language)));
    set.add_rule(Box::new(FnRule::new("thinking_tags", 95, rule_thinking_tags)));
    set.add_rule(Box::new(FnRule::new("tool_call_tags", 90, rule_tool_call_tags)));
    set
}

fn build_deepseek_rules() -> AppRuleSet {
    let mut set = AppRuleSet::new(APP_DEEPSEEK);
    set.add_rule(Box::new(FnRule::new("code_block_language", 100, rule_code_block_language)));
    set.add_rule(Box::new(FnRule::new("thinking_tags", 95, rule_thinking_tags)));
    set
}

fn build_cursor_rules() -> AppRuleSet {
    let mut set = AppRuleSet::new(APP_CURSOR);
    set.add_rule(Box::new(FnRule::new("code_block_language", 100, rule_code_block_language)));
    set.add_rule(Box::new(FnRule::new("tool_call_tags", 90, rule_tool_call_tags)));
    set.add_rule(Box::new(FnRule::new("thinking_tags", 80, rule_thinking_tags)));
    set
}

fn build_ollama_rules() -> AppRuleSet {
    let mut set = AppRuleSet::new(APP_OLLAMA);
    set.add_rule(Box::new(FnRule::new("code_block_language", 100, rule_code_block_language)));
    set.add_rule(Box::new(FnRule::new("thinking_tags", 90, rule_thinking_tags)));
    set.add_rule(Box::new(FnRule::new("tool_call_tags", 85, rule_tool_call_tags)));
    set
}

fn build_generic_rules() -> AppRuleSet {
    let mut set = AppRuleSet::new(APP_GENERIC);
    set.add_rule(Box::new(FnRule::new("code_block_language", 100, rule_code_block_language)));
    set.add_rule(Box::new(FnRule::new("tool_call_tags", 90, rule_tool_call_tags)));
    set.add_rule(Box::new(FnRule::new("thinking_tags", 80, rule_thinking_tags)));
    set
}

// ── 全局注册表 ──────────────────────────────────────

/// APP 注册表 — 单例，启动时初始化所有内置 APP
pub struct AppRegistry {
    apps: HashMap<String, AppRuleSet>,
}

impl AppRegistry {
    /// 获取全局单例
    pub fn global() -> &'static AppRegistry {
        static REGISTRY: OnceLock<AppRegistry> = OnceLock::new();
        REGISTRY.get_or_init(Self::new)
    }

    fn new() -> Self {
        let mut apps = HashMap::new();
        apps.insert(APP_OPENAI.id.to_string(), build_openai_rules());
        apps.insert(APP_CLAUDE.id.to_string(), build_claude_rules());
        apps.insert(APP_GEMINI.id.to_string(), build_gemini_rules());
        apps.insert(APP_QWEN.id.to_string(), build_qwen_rules());
        apps.insert(APP_DEEPSEEK.id.to_string(), build_deepseek_rules());
        apps.insert(APP_CURSOR.id.to_string(), build_cursor_rules());
        apps.insert(APP_OLLAMA.id.to_string(), build_ollama_rules());
        apps.insert(APP_GENERIC.id.to_string(), build_generic_rules());
        Self { apps }
    }

    /// 查找 APP 规则集，找不到返回 generic
    pub fn get(&self, app_id: &str) -> &AppRuleSet {
        self.apps
            .get(app_id)
            .unwrap_or_else(|| self.apps.get("generic").expect("generic must exist"))
    }

    /// 检查 APP 是否在注册表中
    pub fn contains(&self, app_id: &str) -> bool {
        self.apps.contains_key(app_id)
    }

    /// 列出所有已注册 APP 的元信息
    pub fn list_apps(&self) -> Vec<&AppInfo> {
        self.apps.values().map(|s| &s.info).collect()
    }
}
