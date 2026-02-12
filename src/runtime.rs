use std::sync::Arc;

use teloxide::prelude::*;

use crate::config::Config;
use crate::db::Database;
use crate::llm::LlmProvider;
use crate::memory::MemoryManager;
use crate::skills::SkillManager;
use crate::tools::ToolRegistry;

pub struct AppState {
    pub config: Config,
    pub bot: Bot,
    pub db: Arc<Database>,
    pub memory: MemoryManager,
    pub skills: SkillManager,
    pub llm: Box<dyn LlmProvider>,
    pub tools: ToolRegistry,
}
