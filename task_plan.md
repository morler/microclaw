# 任务计划: 为 MicroClaw 添加 Zeroclaw 风格的 SQLite 记忆系统

## 状态: ✅ 已完成

## 完成的工作

### 阶段 1: 创建 memory 模块结构 ✅
- [x] 1.1 创建 `src/memory/` 目录结构
- [x] 1.2 创建 `traits.rs` - Memory trait 定义
- [x] 1.3 创建 `sqlite.rs` - SqliteMemory 实现
- [x] 1.4 创建 `mod.rs` - 模块入口 + MemoryManager
- [x] 1.5 创建 `vector.rs` - 向量操作
- [x] 1.6 创建 `embeddings.rs` - 嵌入提供者

### 阶段 2: SqliteMemory 核心实现 ✅
- [x] 2.1 数据库初始化与 schema 创建
- [x] 2.2 store/get/list/forget 方法实现
- [x] 2.3 recall 方法 - FTS5 搜索实现
- [x] 2.4 recall 方法 - 向量搜索实现
- [x] 2.5 recall 方法 - 混合搜索实现
- [x] 2.6 嵌入缓存实现

### 阶段 3: 配置集成 ✅
- [x] 3.1 在 config.rs 添加 memory 配置项
- [x] 3.2 在 AppState 中集成新的记忆系统
- [x] 3.3 更新 agent_engine 使用新记忆系统

### 阶段 4: 测试与验证 ✅
- [x] 4.1 所有单元测试通过
- [x] 4.2 集成测试通过

## 使用方法

在 `microclaw.config.yaml` 中添加以下配置来启用 SqliteMemory:

```yaml
enable_sqlite_memory: true
sqlite_memory_vector_weight: 0.7
sqlite_memory_keyword_weight: 0.3
```

## 关键文件
- `src/config.rs` - 配置定义
- `src/runtime.rs` - AppState 集成
- `src/agent_engine.rs` - 记忆加载逻辑
- `src/memory/` - 新的记忆模块
