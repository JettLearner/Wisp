//! 统一富文本引擎
//!
//! 三段式架构：原始文本 → 归一化器（APP 专属规则）→ 统一 AST → 渲染器
//!
//! - [`ast`]：统一 AST 节点定义（25 种节点类型）
//! - [`app_registry`]：APP 专属规则注册表（8 个内置 APP + generic 兜底）
//! - [`normalizer`]：归一化器，根据 app_id 选择规则集执行

pub mod app_registry;
pub mod ast;
pub mod normalizer;

pub use ast::{AstNode, RichTextDoc};
pub use normalizer::Normalizer;
