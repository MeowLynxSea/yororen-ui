# gpui-ce 迁移计划

调研日期：2026-10-28（Asia/Shanghai）。基线提交：8043e1a8。
迁移分支：codex/migrate-gpui-ce-0.3。禁止修改 yororen-ui.wiki/。

## 基线与版本选择

开始时 `git status --short --branch` 干净，dev 比 origin/dev 超前两个提交；
保留这些提交，在其基础上创建迁移分支。任务背景中的 0.3.3 已不再是当前工作区
锁定版本：基线 manifests 与本地未跟踪的 Cargo.lock 均为 0.2.2，提交
9c9020c0 已做第一轮降版本迁移，ebd0f9f6 补过焦点 API。仍从发布归档 0.3.3
对比，明确哪些变动已经被基线吸收。

crates.io API 实时结果（完整响应保存在 research/）：

| crate | 版本 | yanked | 声明 MSRV |
| --- | --- | --- | --- |
| gpui-ce | 0.3.3 / 0.3.2（2025 年旧版本线） | 是 | 未声明 |
| gpui-ce | 0.2.2（2026-08-28） | 否 | 未声明 |
| gpui-ce | 0.3.0（2026-10-20） | 否 | 1.95 |
| gpui_ce_platform | 0.1.0 / 0.3.0 | 否 | 未单独声明；受 gpui 限制 |

选择 gpui-ce 0.3.0 与 gpui_ce_platform 0.3.0：它们是调研当天最新的正常发布线。
0.2.2 也可正常解析、迁移量较小，但不是当前上游版本线；0.3.3 虽数字较大，
实际早于这次重新发布，且已 yank，不能使用。已向用户说明两种可用版本的取舍。
保持正常的 0.3 semver 范围，不使用 git、patch、replace、私有 registry。
所有内部 gpui 依赖统一在 workspace.dependencies 定义；Cargo.lock 提交入库。
应用平台依赖显式启用 font-kit、wayland、x11，对齐目标归档 examples 的平台功能。

目标源码以 crates.io 发布归档为准，不能把 main 分支新增 API 当作已发布 API：

- 归档：https://static.crates.io/crates/gpui-ce/gpui-ce-0.3.0.crate
- 对应提交：58ff79777198b35e78467795eb50eb258d640eb9（.cargo_vcs_info.json）
- 仓库：https://github.com/gpui-ce/gpui-ce
- issue：https://github.com/MeowLynxSea/yororen-ui/issues/10
- 元数据：https://crates.io/api/v1/crates/gpui-ce
- 生态元数据：https://crates.io/api/v1/crates/gpui_ce_platform 等，research/*.json
- 主分支 README/CHANGELOG/Cargo.toml 均另存 research/；发布归档声明的 VCS
  SHA 对 raw.githubusercontent.com 的 CHANGELOG/Cargo.toml 请求返回 404，
  因此不能把 main 的这些文件伪称为发布提交文件。
  README 的 crates/gpui/examples/learn 路径不代表发布归档包含相同目录。
- 发布结构：crates/gpui 为核心；独立 gpui_platform 选择平台，macos/apple、
  linux、windows、web 等后端单独发布；collections/macros/refineable/
  scheduler/shared_string/sum_tree/util 等通过 gpui_ce_* 发布名解析。

## 工具链（先于代码迁移）

当前 rustc/cargo 1.92.0，不能满足目标。安装固定的 Rust 1.95.0（minimal、
clippy、rustfmt），增加 rust-toolchain.toml；根 workspace.package 声明
rust-version = "1.95"，所有成员继承，独立 fuzz 工作区单独声明。
后续首先确认 rustc/cargo 版本，再迁移核心代码。不依赖本机 nightly 或默认工具链。

## 旧 → 新 API 与行为差异（按影响面排序）

以下路径均相对于 research/gpui-ce-0.3.0/。实际修改每个调用点前仍须读取
目标签名及相邻实现；新增发现补入本表与总结。

| 影响面 | 0.3.3 → 0.3.0 的变化 | 目标源码 / 迁移方式 |
| --- | --- | --- |
| 所有 crate / 类型身份 | 历史 0.3.x 与重新发布线不兼容，不能混用 | 统一 workspace 依赖；无锁解析后检查 gpui 单一版本及所有 registry 元数据 |
| 应用初始化 | Application::new() → gpui_ce_platform::application() | 平台归档 src/gpui_platform.rs:64；基线 demos 已迁移，README 仍旧 |
| AppContext | notify、emit、global entity 操作由 inherent 方法移入 AppContext；借用接口调整 | src/gpui.rs:258、src/app/context.rs；显式导入 trait / prelude；发布版仍为 AppContext，不猜测 main 的分域 trait |
| 焦点 | FocusHandle::focus(window) → focus(window, cx)；Window::focus/focus_next/focus_prev 同样需要 App | src/window.rs:701、3119、3166、3180；基线大多已吸收 |
| 自绘 Element | request_layout/prepaint/paint 增加 inspector 参数；命中测试 boolean → HitboxBehavior | src/element.rs、src/window.rs:6965；基线已部分吸收，核查全部自绘控件 |
| 颜色 | 自有 Rgba/Hsla → palette 类型与转换/ColorExt 扩展；字段与方法不同 | src/color.rs、src/style.rs；检查所有直接字段访问、blend/alpha/conversion，不能仅换版本 |
| 绘制矩形 | PaintQuad 新增 corner_smoothing、border_dashed_length/gap；border_color 改为 Background | src/window.rs:9810；采用 PaintQuad::new(bounds) 及已验证的 builder，保留原边框/背景/圆角 |
| 旧绘制 helper | fill/quad/outline 被标记 deprecated | src/window.rs:9886-9920；改 PaintQuad builder，避免 -D warnings 失败 |
| 边框与阴影 | BoxShadow::color、Style::border_color、RingColor 与 PaintQuad border 由 Hsla 改为 Background | src/style.rs:516、src/window.rs；在 GPUI 边界转换，保留 yororen 的 Hsla token API |
| 阴影/图片/滤镜 | 绘制阴影等接口引入 corner_smoothing；部分低层参数改变 | src/window.rs:5843 等；逐个对照签名，默认 0.0 保持旧圆角 |
| 浮层 | Deferred::with_priority(usize) → derive_setters 生成的 priority(impl Into<DeferredPriority>)；新自动优先级 | src/elements/deferred.rs；保留原显式优先级，避免改变 popover/modal/notification 层次 |
| 长度/布局 | DefiniteLength::Fraction(f32) → Relative(Relative)；方向、inline 等新布局模型 | src/geometry.rs、src/style.rs；对照 relative helper；未使用的变动只记录 |
| 文本 / IME | Parley 文本系统；Font 新增 width，TextRun 新增 letter_spacing；行布局、换行、字节位置与方向语义变化 | src/text_system/、examples/text_input.rs；核查单行、多行、UTF-8/UTF-16、点击选区和焦点测试 |
| 资源 | with_assets 接受 Into<AssetRegistry>；App::asset_source → assets | src/assets.rs、src/app.rs；AssetSource 有 From 实现，UiAsset 可继续传入；核查图标加载 |
| 拖放 | Drag API / DropListener；on_drop 调用模型变化 | src/elements/div.rs；检索实际使用，未使用不引入替代代码 |
| 窗口 | activate_window → activate；WindowOptions 分平台背景与 builder | src/platform.rs、src/window.rs；保留有效字面量写法，核对实际用到字段 |
| 动画 | use_transition/use_keyed_transition 使用 Motion，新增转换与样式动画 | src/window.rs、src/motion.rs；核查是否使用，不改 yororen 自有动画模型 |
| 测试 | TestApp、TestAppContext、test-support/leak detection、平台测试支持变化 | src/app/test_app.rs、src/app/test_context.rs、examples/tests；保留可用旧测试 context，按错误核对 |
| 图片与后端 | image-default-formats、WGPU/WGSL、平台包重组、透明组与 RTL 行为变化 | 目标 Cargo.toml、scene/text_system；显式平台功能，运行 demo 核查关键交互 |

基线 9c9020c0 已吸收、从 0.3.3 迁移时仍必须列出的变化：
BoxShadow 新增 inset；ShapedLine::paint 增加 TextAlign/align_width 两参；
TextRun 新增 letter_spacing；flex_grow 接收 f32；AsyncApp::update 不再返回
Result；颜色字段 h/s/l/a 改为 hue/saturation/lightness/alpha（hue 按角度）；
rust-embed 需要 include-exclude feature。对照归档确认，不再次反向改动。

主分支 CHANGELOG 还列出 hover_style/active_style 等重命名；发布归档
div.rs 中仍使用 hover/active，并无对应弃用标记。迁移不能盲从主分支变更。
同理，所有“破坏性变更”必须区分“发布版本真实变化”“基线已适配”“本项目未用到”。

## 受影响 crate 与顺序

每完成一个 crate 立即 cargo check -p <实际包名>，修到通过才推进下一层；
全量 build/test/clippy 仅在逐层通过后进行。依赖版本统一是解析准备工作，不等于
提前修改下游代码。core 检查不启用下游 test-support，避免依赖 feature 掩盖问题。

1. 工具链、workspace 依赖、lock 策略与调研文档（第一阶段提交）。
2. yororen_ui_core：颜色、输入绘制、文本系统、a11y、动画、theme/renderer 边界。
3. yororen_ui_default_renderer，再 yororen_ui_winui_renderer：所有自绘、阴影、
   浮层与输入；检查内置 themes，保持 JSON token 兼容（渲染层阶段提交）。
4. yororen_ui_locale_en、yororen_ui_locale_zh_cn、yororen_ui_locale_ar：
   逐一查询包名并检查；translations 数据保持语义不变。
5. yororen_ui（门面）及 yororen_ui_xml_macro、yororen_ui_xml：依赖顺序由
   Cargo metadata 确认；编译期宏、运行时 XML、tests/benches/fuzz 同步检查。
6. 所有九个 demos：counter、showcase_xml、gallery_demo、gallery_xml、
   inputs_demo、layers_demo、theme_showcase、variant_showcase、winui-showcase。
   每个包完成后 check；不豁免 clippy（XML/demos 阶段提交）。
7. README 中英接入示例、发布验证与迁移总结（最终阶段提交）。

## 验收

- 移除 Cargo.lock 忽略并提交新 lock。临时目录内从无 lock 的 manifest 副本
  正常解析，再核对元数据；不删除唯一已验证的最终 lock。
- 检查 lock 每个外部 package source 均为 crates.io registry；本 workspace
  package 没有 source 是 Cargo 标准行为，并非 path workaround。
- 对 lock 中每个外部版本查 crates.io sparse registry 的 yanked 字段；
  grep 只能确认来源，无法证明未 yank。记录完整机器可读审计结果。
- cargo build --workspace、cargo test --workspace、
  cargo clippy --workspace -- -D warnings，全量包含 demos。
- 在 macOS 运行 demos，抽查输入、下拉与列表以及图标/层次，保存截图/日志；
  用实际 UI 结果和已有 GPUI 测试区分人工交互与自动化断言覆盖。
- 最后确认没有 patch/replace/git dependency，wiki diff 为空，提交分阶段。

## 风险与不确定项

- 新 0.3.0 发布仅八天，主分支 CHANGELOG 标为 Unreleased；发布归档为唯一 API
  基准，不能承诺 main 修复已包含在该版。目标生态所有包需分别查 yank 状态。
- Rust 1.95 是明确的下游升级要求。此前未声明 MSRV，需在 README 说明。
- `gpui` 类型出现在 yororen 公开接口；即使自身方法不改，依赖类型身份仍不兼容，
  下游必须统一使用新版本；确实的签名变化列在总结「公开 API 变化清单」。
- 本机可验证 macOS；Linux/Windows/Web 的原生工具链与运行时不在本轮机器条件内。
- 自绘输入不等同于上游 EditableText：需特别关注 Unicode、IME、光标与多行。
- 当前只有约 15 GiB 可用空间，旧 target 为约 22 GiB；必要时清理构建缓存，
  不把磁盘失败当源码错误，也不省略最终验收。
- README、上游 API 快照、独立 fuzz/历史嵌套 lock 可能保留旧路径；逐项处理，
  不改 yororen-ui.wiki。

## 临时 workaround 登记

尚未引入。调研用下载源码只在 tmp/migrate-docs/research，不参与 Cargo 构建。
任何后续临时处理在此登记并在完成前清除；不接受依赖替换作为交付物。

## 公开 API 变化清单

待逐层编译确认后补全到 summary.md。目标是保留 yororen 自身组件、token、
locale、XML schema 的公开接口；gpui 升级与 MSRV 变化必须明确披露。
