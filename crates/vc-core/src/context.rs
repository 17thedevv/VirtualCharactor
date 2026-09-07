use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ContextPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextSource {
    Memory,
    System,
    User,
    Environment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextItem {
    pub source: ContextSource,
    pub content: String,
    pub priority: ContextPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub items: Vec<ContextItem>,
}
