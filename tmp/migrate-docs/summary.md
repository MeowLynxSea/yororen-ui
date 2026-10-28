# gpui-ce 迁移总结

完成日期：2026-10-28（Asia/Shanghai）。
分支：`codex/migrate-gpui-ce-0.3`；基线：`8043e1a8`。
本轮没有修改 `yororen-ui.wiki/`，没有上传 crate、推送分支或创建发布 tag。

## 目标与发布线

统一采用 crates.io 的 **gpui-ce 0.3.0 / gpui_ce_platform 0.3.0**，所有
GPUI 生态依赖使用正常 registry 解析。它们是 2026 年 10 月的新发布线；
历史 0.3.2/0.3.3 已 yank，不能按版本号大小判断发布时间或兼容性。
0.2.2 仍可用，但不是本轮选定的当前版本线。详细调研、备选取舍和 API
迁移表见 [plan.md](plan.md)，发布元数据见
[upstream-versions.json](upstream-versions.json)。

实际起始 manifests 与本地未跟踪的锁文件为 0.2.2；之前的提交已经吸收部分
0.3.3 → 0.2.2 的变化。因此本轮同时记录历史 API 差异和当前基线的增量修改，
没有把已经存在的改动算作本轮新实现。

- 工具链固定为 Rust **1.95.0**，workspace 和全部成员声明 MSRV 1.95。
- 九个可发布 Yororen crate 一起准备 **0.4.0**，内部依赖下限为 `"0.4"`。
  原 0.3.0 已发布，不能覆盖；公开 GPUI 类型身份变化也需要不兼容版本号。
- 平台 feature 显式启用 `font-kit`、`wayland`、`x11`。
- 根 `Cargo.lock` 已跟踪，无 patch、replace、git 或私有 registry 依赖。
  workspace 本地成员没有 `source` 是 Cargo 的标准表示。
- 所有 crates.io 接入示例指向准备发布的 0.4；README 明确说明正式发布前
  尚不能使用该版本/tag 接入。这次交付提供发布准备，不声称已修复线上旧包。

## 各层迁移结果

| 阶段 | crate | 修改与核验 |
| --- | --- | --- |
| core | yororen_ui_core | 三参 shape_line；按发布版 platform_layout 实现光标位置与点击命中；迁移 PaintQuad；单行 RTL 区间取 min/max；补 Unicode/边界几何测试；独立 cargo check 通过 |
| default renderer | yororen_ui_default_renderer | 矩形 builder、阴影 Background 转换、Deferred 显式优先级；保留 token 类型；独立 check 通过 |
| WinUI renderer | yororen_ui_winui_renderer | 同上，增加 shadow corner_smoothing=0.0、ContentMask 默认零 fade_out；保留硬裁剪和浮层顺序；独立 check 通过 |
| locales | en、zh_cn、ar | 逐包 check；语言数据无迁移所需结构变更，继续通过解析与翻译测试 |
| XML | yororen_ui_xml → yororen_ui_xml_macro | 按依赖顺序 check；parser/codegen、宏、tests/benches、独立 fuzz 目标均覆盖 |
| facade | yororen_ui | 最后检查门面、默认 XML 和可选 WinUI 集成；公开 re-export 保留 |
| demos | 全部九个 | 逐包 check 后全量 build/test/clippy；gallery 和 WinUI showcase 更新 Deferred 优先级 |
| 原生启动修正 | 全部九个及中英 README | App::spawn 在启动回调返回后初始化窗口；open_window 错误明确报出；创建后 activate(true)；真实 macOS 进程、屏幕窗口和截图逐个核验 |

所有实际调用的替代接口均先检查 crates.io 发布归档源码。特别区分主分支
CHANGELOG 与发布归档：未把主分支的 hover_style 等未发布重命名套到 0.3.0。
目标归档的关键源码位置已列在 plan.md；发布归档的校验和记录在版本元数据中。
原始下载源码留在本地忽略的 research/，不参与编译、不打入提交。

文本初始化的交互测试还发现 `text_input::init` 的旧 `static OnceLock` 会跨
App 跳过键盘绑定。改为目标版本真实支持的 `App::has_global/set_global`，
保证每个 App 注册一次。函数签名和调用方式保持不变。
既有 blink 测试同步改为 `window.blur(cx)`。

## macOS 原生启动死锁与修正

用户运行 demo 未见窗口后，直接运行 counter 复现：进程存在但首个窗口未创建。
`counter-sample.txt` 的线程采样显示主线程停在
`MacPlatform::open_window → Mutex::lock → parking_lot`，调用链来自
`did_finish_launching → Application::run callback → App::open_window`。
这条路径不在先前的 GPUI mock 测试窗口覆盖范围内。

目标发布归档 `gpui_ce_macos 0.3.0/src/platform.rs:1282`：

```rust
if let Some(callback) = platform.0.lock().finish_launching.take() {
    callback();
}
```

锁守卫在 if-let 主体结束前仍然存活；同文件 `open_window:776` 再次请求
`self.0.lock()`，发生同线程非重入锁死锁。不是构建产物缺失或隐藏窗口。
未修改任何 registry 缓存、上游源码、依赖来源或锁文件。

九个 demo 及下游接入示例统一采用目标版真实存在的前台任务 API：

```rust
app.run(|cx: &mut gpui::App| {
    cx.spawn(async |cx| cx.update(open_demo)).detach();
});
```

`open_demo` 是普通私有函数，包含原来的主题/locale/state/窗口初始化。
`App::spawn` 位于 gpui/src/app.rs:2871，`AsyncApp::update` 位于
gpui/src/app/async_context.rs:257，返回值直接为 R。任务在主线程后续调度时执行，
届时启动回调已经返回并释放平台锁；不能简单替换成同一 update 周期内的 defer。
窗口结果改为 expect，避免静默吞掉创建失败，随后按上游 examples 调用
`cx.activate(true)`。Yororen 库自身公开 API 不受影响。

这是该 registry 版本下公开、必要的启动顺序适配，README 明确告知下游，
不把它留作隐蔽临时代码。上游同步启动回调的锁生命周期缺陷本身仍在发布版中；
未来升级上游时应重新核对，再决定是否恢复同步初始化。

2026-10-28 18 点后的原生检查逐个启动全部九个最终 Debug 二进制，
通过 CoreGraphics 确认属于各自 PID 的 onscreen 普通窗口，并保存窗口截图。
九张截图均检查过，首帧含实际控件、文字、图标和主题；用户也确认 counter
窗口已经出现。证据见 [native-startup/report.json](native-startup/report.json)。
检查脚本 [check_native_startup.py](check_native_startup.py) 可复跑；它只关闭
自己启动的测试进程。

## 公开 API 变化清单

没有删除 Yororen 组件方法、修改组件 builder 参数、改动主题 token JSON、
翻译格式或 XML schema。下列兼容性变化必须随 0.4 说明：

1. **工具链与 GPUI 类型身份**：下游需 Rust ≥1.95，并同步升级所有 Yororen
   crate；直接依赖 GPUI 时使用同一 0.3.0 发布线。旧 GPUI 的 Entity、Window、
   Hsla 等类型不能与新发布线混用。入口采用
   `gpui_ce_platform::application()`，相关 trait 按新版导入。
2. **文本预绘制状态的 auto trait**：
   `headless::text_area_element::TextAreaPrepaintState` 与
   `headless::text_input_element::PrepaintState` 不再实现 `UnwindSafe` 和
   `RefUnwindSafe`，原因是上游文本布局内部类型变化。下游若用 catch_unwind
   捕获这些状态，应重构捕获边界，在失败后重新构造状态；不要假定布局缓存
   可以跨 panic 继续复用。本轮没有伪造这些 auto trait。
3. **renderer 快照中的颜色类型**：两种 renderer 所有返回 Hsla 的公共
   方法，快照从 `gpui::color::Hsla` 展开为 `palette::hsl::Hsla`。目标 GPUI
   的 `gpui::Hsla` 即该类型 re-export，可继续通过该路径引用；外部若直接
   访问旧颜色字段，应按 plan.md 更新。主题和 BorderSpec/ShadowSpec 仍保留
   Hsla，在实际绘制边界才转换成 Background。
4. **renderer 快照中的类型路径**：所有 FontWeight 返回值展开为
   `gpui::text_system::font_weight::FontWeight`，SharedString 返回值展开为
   `gpui_ce_shared_string::SharedString`，包括两 renderer 的
   `TokenLabelRenderer/WinUILabelRenderer::family_mono` 和 WinUI
   `themes::default_font`。下游继续使用 GPUI 的公开 re-export
   `gpui::FontWeight`、`gpui::SharedString`，无需跟随内部模块路径。
5. **继承方法增加**：ClickOutsideCapture、FocusTrap 的 IntoElement
   快照增加默认的 `into_any_element`；旧方法仍在。

五份快照通过 `scripts/regen-public-api.sh --all` 完整再生成。
[public-api-review.json](public-api-review.json) 逐 crate 记录差异：
default/WinUI renderer 的全部变化均可由上述三种类型路径替换解释；
core 剩余差异仅为两个继承方法和四条 unwind auto-trait 记录；
facade、XML 快照不变。renderer 的旧快照包含此前版本迁移尚未更新的路径，
所以不能把所有路径展开差异都归因于这次新增的源码修改。

## 验收结果

本机 macOS，Rust 1.95.0。下表命令均以 exit code 0 完成；
原始成功日志和 [validation.json](validation.json) 一起保存。

| 验收项 | 结果 | 证据 |
| --- | --- | --- |
| cargo build --workspace | 通过，含全部九个 demos，最终源码重新构建 | build-workspace.log |
| cargo test --workspace | 424 passed / 0 failed / 45 ignored，32 组结果 | test-workspace.log |
| cargo clippy --workspace -- -D warnings | 通过，demos 无豁免 | clippy-workspace.log |
| cargo check --workspace --all-targets --all-features | 通过，包含 tests、benches 和可选 features | check-all-targets-features.log |
| 独立 XML fuzz 工作区 | 无锁重新解析后 cargo check 通过；未运行 fuzz campaign | fuzz-check.log / fuzz-registry-audit.json |
| crates.io 来源、yank、校验和 | 778 个外部锁定版本，全为 crates.io，无 yank/校验和错误；18 个本地 workspace 成员 | registry-audit.json |
| 无锁 workspace | 临时复制 manifests/源码，不携带锁文件，正常解析；外部版本集合与已审计根锁完全一致 | downstream-check.json / downstream-check.log |
| 独立下游示例 | 无旧 lock、无 patch，默认 XML + WinUI 编译通过；仅有 gpui-ce 0.3.0 | downstream-check.json |
| 全部九个原生 demo | 修正启动顺序后，实际运行、onscreen 窗口检测、首帧截图检查通过 | native-startup/report.json / native-startup.log |
| 九个发布包 | 指定九个可发布包的 cargo package 成功；归一化 manifests 无 path/git/patch，内部版本均 0.4 | package-publishable.log / package-check.json |
| 公开 API 基线 | 全部五份生成成功，逐项差异已说明 | public-api-review.json / public-api.log |

45 个 ignored 是已有文档/示例的忽略项，本轮未新增 ignore。
独立 fuzz 锁有 397 个 registry 版本，复用根审计记录核对后无 yank 残留；
其锁文件仍按独立工作区惯例忽略，不是发行版根锁。

根锁 SHA-256：
`a3ae7799d2fb28d58a8a65b0a34de4c6c8662d919e5062f5495a7a6f42600e2f`。
yank 属于 registry 实时状态：此次结果以 registry-audit.json 的时间为准。
可重跑 `python3 tmp/migrate-docs/audit_registry.py --output /tmp/yororen-audit.json`。
单纯 grep source 无法证明未 yank，因此采用 sparse index 逐版本核查。

发布验证边界：`cargo package` 使用 `--no-verify --allow-dirty`，这里验证的是
归档和归一化 manifest；编译证据来自完整 workspace 和独立下游检查。
本轮未声称所有发布归档被 Cargo 独立重新编译。独立下游只对尚未发布的
Yororen 0.4 源码使用本地 path，所有外部依赖正常走 crates.io。发布到
crates.io 后，最终用户才能直接以 `yororen_ui = "0.4"` 使用。

过程中的两个验证配置问题均已纠正并重跑：`cargo package --workspace`
错误地包含了 publish=false 的 demo，改为显式选择九个发布包；独立临时目录
起初继承全局 Rust 1.92，检查脚本改为显式指定 1.95.0。没有用依赖替换规避。

## Demo 行为核验与剩余风险

WinUI showcase 使用真实页面结构、GPUI 测试窗口、模拟点击和键盘事件，
以下测试在最终 workspace test 中通过：

- `migrated_showcase_accepts_unicode_and_keyboard_edits`：点击输入框，
  输入中文和 emoji，按 Backspace 后 emoji 整体移除，App 文本同步。
- `migrated_showcase_dropdown_picks_through_deferred_layer`：点击打开下拉，
  等待动画，点击浮层选项，核对选中值、回调状态和关闭行为。
- `showcase_listbox_arrows_after_click`：点击列表后用方向键移动高亮。

core 的几何测试另覆盖空文本、ASCII、中文/emoji 字节边界和 UTF-16 末端。
既有输入、选区、光标闪烁及 XML 测试均通过。测试 debug selector 仅在 cfg(test)
中加入，不改变发行版 demo 布局。

首次迁移验收期间宿主 Mac 锁屏，尚不能人工视觉抽查。后续用户反馈启动问题，
在桌面可用后定位并修正上述原生启动死锁，已补齐九个原生窗口与首帧截图检查。
这仍不等同于真实系统 IME 候选窗、剪贴板、复杂 RTL 混排和所有浮层的完整
端到端验证；Linux/Windows 原生运行也未在本机验证。建议发布前补查这些交互
并在目标平台执行构建，不得把剩余项写成已通过。

public-api 使用现有 nightly 工具生成，出现既有文档 `ignore` 代码块中
省略号的 rustdoc 警告；生成成功，稳定工具链上的三条验收命令通过。
这些文档格式警告保留在日志，未扩展为本轮无关的源码清理。

## 发布顺序与交付

发布时顺序为 core → 两个 renderer → 三个 locale → XML → XML macro →
facade；仓库既有 `scripts/publish.sh` 已采用这一拓扑。需等各版本索引可见再
发布依赖它的 crate。完成发布后再创建 v0.4.0 tag 并更新 issue #10；这些外部
操作未在本轮执行。九个 demo 保持 publish=false，不上传。

分阶段提交包括依赖/工具链/计划、core、default renderer、WinUI renderer、
demos、输入初始化与交互测试、0.4 发布准备、API 基线/迁移总结，以及用户
反馈后的九个 demo 原生启动顺序修正和真实窗口检查。本轮没有遗留临时依赖
workaround；0.3.0 的必要启动适配与上游剩余缺陷已在本文明确说明。
