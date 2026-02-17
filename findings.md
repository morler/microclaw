# 研究发现: Zeroclaw SQLite 记忆系统

## Zeroclaw 项目核心实现

### SqliteMemory 架构
位置: `src/memory/sqlite.rs`

**核心特性:**
1. 独立的 brain.db 数据库文件
2. 完整的表结构:
   - `memories` - 核心记忆表 (id, key, content, category, embedding, timestamps)
   - `memories_fts` - FTS5 虚拟表用于全文搜索
   - `embedding_cache` - 嵌入缓存表 (LRU)
3. 索引:
   - `idx_memories_category`
   - `idx_memories_key`
   - `idx_cache_accessed`

**搜索能力:**
- FTS5 BM25 关键词搜索
- 向量余弦相似度搜索 (存储为 BLOB)
- 混合搜索: weighted fusion of vector + keyword

**性能优化:**
- WAL 模式
- Memory-mapped I/O
- 嵌入缓存 LRU 淘汰

### Memory Trait
位置: `src/memory/traits.rs`

```rust
pub trait Memory: Send + Sync {
    fn name(&self) -> &str;
    async fn store(&self, key: &str, content: &str, category: MemoryCategory) -> anyhow::Result<()>;
    async fn recall(&self, query: &str, limit: usize) -> anyhow::Result<Vec<MemoryEntry>>;
    async fn get(&self, key: &str) -> anyhow::Result<Option<MemoryEntry>>;
    async fn list(&self, category: Option<&MemoryCategory>) -> anyhow::Result<Vec<MemoryEntry>>;
    async fn forget(&self, key: &str) -> anyhow::Result<bool>;
    async fn count(&self) -> anyhow::Result<usize>;
    async fn health_check(&self) -> bool;
}
```

## MicroClaw 当前实现

### 现有架构
- 同一个数据库 `microclaw.db`
- memories 表存在但结构不同
- 已有 sqlite-vec 支持向量搜索
- 关键词搜索使用 LIKE

### 差异对比

| 特性 | Zeroclaw | MicroClaw |
|------|----------|-----------|
| 数据库 | 独立 brain.db | 共享 microclaw.db |
| FTS5 | 有 | 无 (需要 sqlite-vec) |
| 向量存储 | BLOB | sqlite-vec 扩展 |
| 混合搜索 | 有 | 需要实现 |
| 嵌入缓存 | 有 | 无 |
| Key-Value | 有 (UNIQUE key) | 无 (自增 ID) |

## 实现方案

将 Zeroclaw 的 SqliteMemory 完整移植到 MicroClaw:
1. 创建 `src/memory/` 模块
2. 实现完整的 SqliteMemory
3. 添加配置选项启用
4. 集成到 AppState
