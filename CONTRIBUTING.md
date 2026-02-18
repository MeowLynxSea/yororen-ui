# Contributing to yororen-ui

感谢你对 yororen-ui 的兴趣！我们欢迎各种形式的贡献，包括 bug 修复、新功能、文档改进等。

## 开发环境设置

### 前置要求

- Rust 1.75+
- Cargo

### 本地开发

```bash
# 克隆项目
git clone https://github.com/MeowLynxSea/yororen-ui.git
cd yororen-ui

# 构建项目
cargo build

# 运行测试
cargo test

# 运行 clippy 检查
cargo clippy
```

## 代码规范

- 遵循 [Rust API 规范](https://rust-lang.github.io/api-guidelines/)
- 使用 `cargo clippy` 检查代码风格
- 确保 `cargo fmt` 格式化代码
- 所有公开 API 必须有文档注释

## 提交规范

我们使用 [Conventional Commits](https://www.conventionalcommits.org/) 提交格式：

```
<type>(<scope>): <description>

[可选的正文]

[可选的脚注]
```

### 类型 (type)

- `feat`: 新功能
- `fix`: bug 修复
- `docs`: 文档修改
- `style`: 代码格式调整（不影响功能）
- `refactor`: 代码重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建过程或辅助工具变动

### 示例

```bash
git commit -m "feat(button): add loading state support"
git commit -m "fix(toast): fix memory leak in notification queue"
git commit -m "docs(readme): add installation instructions"
```

## Pull Request 流程

1. Fork 仓库并创建分支
2. 进行开发并提交代码
3. 确保所有测试和检查通过
4. 提交 Pull Request
5. 等待代码审查

### PR 检查清单

- [ ] `cargo build` 成功
- [ ] `cargo test` 全部通过
- [ ] `cargo clippy` 无警告
- [ ] 代码已格式化 (`cargo fmt`)
- [ ] 已添加/更新相关文档

## 行为准则

请阅读并遵守我们的 [Code of Conduct](CODE_OF_CONDUCT.md)。

## 问题反馈

- 使用 GitHub Issues 报告 bug
- 使用 GitHub Issues 提出新功能请求
- 提交问题时，请提供复现步骤和环境信息
