pub mod embeddings;
pub mod sqlite;
pub mod traits;
pub mod vector;

pub use embeddings::EmbeddingProvider;
pub use sqlite::SqliteMemory;
pub use traits::{Memory, MemoryCategory, MemoryEntry};

use std::path::{Path, PathBuf};
use std::sync::Arc;

/// File-based memory manager (AGENTS.md)
pub struct MemoryManager {
    data_dir: PathBuf,
}

impl MemoryManager {
    pub fn new(data_dir: &str) -> Self {
        MemoryManager {
            data_dir: PathBuf::from(data_dir).join("groups"),
        }
    }

    fn global_memory_path(&self) -> PathBuf {
        self.data_dir.join("AGENTS.md")
    }

    fn chat_memory_path(&self, chat_id: i64) -> PathBuf {
        self.data_dir.join(chat_id.to_string()).join("AGENTS.md")
    }

    pub fn read_global_memory(&self) -> Option<String> {
        let path = self.global_memory_path();
        std::fs::read_to_string(path).ok()
    }

    pub fn read_chat_memory(&self, chat_id: i64) -> Option<String> {
        let path = self.chat_memory_path(chat_id);
        std::fs::read_to_string(path).ok()
    }

    #[allow(dead_code)]
    pub fn write_global_memory(&self, content: &str) -> std::io::Result<()> {
        let path = self.global_memory_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)
    }

    #[allow(dead_code)]
    pub fn write_chat_memory(&self, chat_id: i64, content: &str) -> std::io::Result<()> {
        let path = self.chat_memory_path(chat_id);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)
    }

    pub fn build_memory_context(&self, chat_id: i64) -> String {
        let mut context = String::new();

        if let Some(global) = self.read_global_memory() {
            if !global.trim().is_empty() {
                context.push_str("<global_memory>\n");
                context.push_str(&global);
                context.push_str("\n</global_memory>\n\n");
            }
        }

        if let Some(chat) = self.read_chat_memory(chat_id) {
            if !chat.trim().is_empty() {
                context.push_str("<chat_memory>\n");
                context.push_str(&chat);
                context.push_str("\n</chat_memory>\n\n");
            }
        }

        context
    }

    #[allow(dead_code)]
    pub fn groups_dir(&self) -> &Path {
        &self.data_dir
    }
}

/// Create memory backend based on configuration
pub fn create_memory(
    data_dir: &str,
    embedding_provider: Option<&str>,
    embedding_api_key: Option<&str>,
    embedding_model: &str,
    embedding_dim: usize,
) -> anyhow::Result<Box<dyn Memory>> {
    let workspace_dir = Path::new(data_dir);

    let embedder = if let Some(provider) = embedding_provider {
        if provider.is_empty() {
            Arc::new(embeddings::NoopEmbedding) as Arc<dyn EmbeddingProvider>
        } else {
            embeddings::create_provider(provider, embedding_api_key, embedding_model, embedding_dim)
        }
    } else {
        Arc::new(embeddings::NoopEmbedding) as Arc<dyn EmbeddingProvider>
    };

    let mem = SqliteMemory::with_embedder(workspace_dir, embedder, 0.7, 0.3, 10_000)?;
    Ok(Box::new(mem))
}
