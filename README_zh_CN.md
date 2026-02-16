<h1>Yororen UI</h1>

<p><a href="README.md">English Version</a></p>

<p>Yororen UI 是一个基于 <a href="https://github.com/zed-industries/zed" target="_blank"><code>gpui</code></a>（Zed）构建的可复用 UI 组件和小部件库。</p>

<p>它旨在被 <code>gpui</code> 应用程序 crate 使用，同时保持 UI 层的独立性（主题、组件、小部件和嵌入式图标资源）。</p>

<hr>

<h2>特性</h2>

<table>
<tr>
  <td><strong>60+ 组件</strong></td>
  <td>按钮、输入框、徽章、工具提示、图标、标题、卡片、模态框、树形控件等</td>
</tr>
<tr>
  <td><strong>小部件</strong></td>
  <td>TitleBar、VirtualList 等高级小部件</td>
</tr>
<tr>
  <td><strong>主题系统</strong></td>
  <td><code>GlobalTheme</code> + <code>ActiveTheme</code> 辅助工具，支持浅色/深色模式</td>
</tr>
<tr>
  <td><strong>动画系统</strong></td>
  <td>可配置的动画，包含预设、缓动函数和编排器</td>
</tr>
<tr>
  <td><strong>国际化</strong></td>
  <td>多语言支持（英文、中文），文本方向支持（LTR/RTL）</td>
</tr>
<tr>
  <td><strong>嵌入式资源</strong></td>
  <td>29+ 个 SVG 图标，通过 <code>rust-embed</code> 嵌入（<code>assets/icons/**</code>）</td>
</tr>
</table>

<h2>环境要求</h2>

<ul>
  <li><strong>Rust edition:</strong> 2024（与您的 <code>gpui</code> 应用程序使用的工具链兼容）</li>
  <li><code>gpui</code> 是 git 依赖，需要固定到特定提交</li>
</ul>

<h2>安装</h2>

<h3>从 GitHub 使用（推荐）</h3>

<p>通过 git 添加此 crate 作为依赖，并使用发布标签固定：</p>

<pre><code>[dependencies]
yororen_ui = { git = "https://github.com/MeowLynxSea/yororen-ui.git", tag = "v0.1.0" }
</code></pre>

<h3>从本地路径使用（开发时）</h3>

<pre><code>[dependencies]
yororen_ui = { path = "../yororen-ui" }
</code></pre>

<h2>固定 <code>gpui</code></h2>

<p><code>gpui</code> 更新频繁，且通过 git 依赖进行使用。如果您的应用程序和 <code>yororen_ui</code> 使用了<em>不同</em>的 <code>gpui</code> 版本，您将看到如下错误：</p>

<blockquote>
  "multiple different versions of crate <code>gpui</code> in the dependency graph"
</blockquote>

<p>以及许多 trait/类型不匹配的问题。</p>

<p><strong>推荐方法：</strong></p>

<ul>
  <li>在您的应用程序工作区中，将 <code>gpui</code> 固定到相同的 <code>rev</code></li>
  <li>将 <code>yororen_ui</code> 和您的应用程序都固定到相同的 <code>gpui</code> 版本</li>
</ul>

<p>在本仓库中，<code>gpui</code> 已在 <code>Cargo.toml</code> 中固定。</p>

<h2>快速开始</h2>

<h3>1) 注册组件</h3>

<p>某些组件需要一次性注册/初始化。在应用程序启动时调用 <code>component::init</code>：</p>

<pre><code>use gpui::App;
use yororen_ui::component;

fn init_ui(cx: &amp;mut App) {
    component::init(cx);
}
</code></pre>

<h3>2) 安装全局主题</h3>

<p>Yororen UI 提供了一个 <code>GlobalTheme</code>，可根据 <code>WindowAppearance</code> 自动选择浅色/深色配色方案。</p>

<pre><code>use gpui::App;
use yororen_ui::theme::GlobalTheme;

fn init_theme(cx: &amp;mut App) {
    cx.set_global(GlobalTheme::new(cx.window_appearance()));
}
</code></pre>

<p>在渲染函数中，您可以通过 <code>ActiveTheme</code> 访问主题颜色：</p>

<pre><code>use gpui::{Render, div};
use yororen_ui::theme::ActiveTheme;

// 在 render(..., cx: &amp;mut gpui::Context&lt;Self&gt;) 中
let theme = cx.theme();
let _ = div().bg(theme.surface.base).text_color(theme.content.primary);
</code></pre>

<h3>3) 提供资源（图标）</h3>

<p>此 crate 将其图标嵌入到 <code>assets/icons/**</code> 下，并作为 <code>gpui::AssetSource</code>（<code>yororen_ui::assets::UiAsset</code>）公开。</p>

<p>如果您的应用程序只需要 Yororen UI 的图标，可以直接安装：</p>

<pre><code>use gpui::Application;
use yororen_ui::assets::UiAsset;

let app = Application::new().with_assets(UiAsset);
</code></pre>

<p>如果您的应用程序也有自己的资源，可以组合资源源以同时使用两组资源。Yororen UI 提供了一个小助手 <code>CompositeAssetSource</code>：</p>

<pre><code>use gpui::Application;
use yororen_ui::assets::{CompositeAssetSource, UiAsset};

// `MyAsset` 是您自己的 AssetSource 实现
let app = Application::new().with_assets(CompositeAssetSource::new(MyAsset, UiAsset));
</code></pre>

<p><strong>重要：</strong> 您的主要 <code>AssetSource</code> 在路径不存在时应返回 <code>Ok(None)</code>。如果它在缺失路径时返回错误，可能会阻止回退到 <code>UiAsset</code>。</p>

<h2>包含内容</h2>

<h3>模块</h3>

<table>
<tr>
  <td><code>yororen_ui::theme</code></td>
  <td>
    <ul>
      <li><code>Theme</code>（配色方案）</li>
      <li><code>GlobalTheme</code>（<code>gpui::Global</code>）</li>
      <li><code>ActiveTheme</code> trait（在 <code>App</code> 和渲染上下文中提供 <code>theme()</code>）</li>
    </ul>
  </td>
</tr>
<tr>
  <td><code>yororen_ui::assets</code></td>
  <td>
    <ul>
      <li><code>UiAsset</code>（嵌入图标的 <code>gpui::AssetSource</code>）</li>
      <li><code>CompositeAssetSource</code>（组合两个资源源并支持回退）</li>
    </ul>
  </td>
</tr>
<tr>
  <td><code>yororen_ui::component</code></td>
  <td>
    <ul>
      <li>常用构建块：<code>button</code>、<code>icon_button</code>、<code>text_input</code>、<code>password_input</code>、<code>tooltip</code>、<code>badge</code>、<code>divider</code> 等</li>
      <li><code>component::init(cx)</code> 用于任何注册</li>
    </ul>
  </td>
</tr>
<tr>
  <td><code>yororen_ui::widget</code></td>
  <td>由组件组合的高级小部件。目前包括：<code>TitleBar</code> 和辅助构造函数。</td>
</tr>
</table>

<h3>组件概览</h3>

<table>
<tr>
  <td><strong>基础</strong></td>
  <td>Button, IconButton, Icon, Label, Text, Heading, Spacer, Divider</td>
</tr>
<tr>
  <td><strong>输入</strong></td>
  <td>TextInput, PasswordInput, NumberInput, TextArea, SearchInput, FilePathInput, KeybindingInput</td>
</tr>
<tr>
  <td><strong>选择</strong></td>
  <td>Checkbox, Radio, RadioGroup, Switch, Slider, Select, ComboBox</td>
</tr>
<tr>
  <td><strong>展示</strong></td>
  <td>Badge, Avatar, Image, Progress, Skeleton, Tag</td>
</tr>
<tr>
  <td><strong>浮层</strong></td>
  <td>Tooltip, Popover, Modal, Toast, DropdownMenu</td>
</tr>
<tr>
  <td><strong>布局</strong></td>
  <td>Card, ListItem, EmptyState, Disclosure</td>
</tr>
<tr>
  <td><strong>交互</strong></td>
  <td>ClickableSurface, ToggleButton, SplitButton, DragHandle, ButtonGroup</td>
</tr>
<tr>
  <td><strong>树形/层级</strong></td>
  <td>Tree, TreeNode, TreeItem, TreeData, TreeDrag</td>
</tr>
<tr>
  <td><strong>表单</strong></td>
  <td>Form, ContextMenuTrigger</td>
</tr>
<tr>
  <td><strong>导航</strong></td>
  <td>TitleBar 小部件, VirtualList, VirtualRow</td>
</tr>
<tr>
  <td><strong>工具</strong></td>
  <td>FocusRing, ShortcutHint, KeybindingDisplay</td>
</tr>
</table>

<h3>图标</h3>

<p>组件图标 API 使用强类型名称：</p>

<pre><code>use yororen_ui::component::{icon, IconName};

let _ = icon(IconName::Minecraft);
</code></pre>

<p>图标路径映射到嵌入的 SVG 资源，如 <code>icons/minecraft.svg</code>。</p>

<h2>许可证</h2>

<ul>
  <li>Yororen UI 采用 <strong>Apache License, Version 2.0</strong> 授权。参见 <code>LICENSE</code>。</li>
  <li>本项目基于 <code>gpui</code>（Zed Industries）构建，同样采用 Apache-2.0 许可证。</li>
</ul>

<p>归属详情请参见 <code>NOTICE</code>。</p>

<h2>贡献</h2>

<p>欢迎提交 Issue 和 PR。</p>

<p>修改视觉效果时：</p>

<ul>
  <li>请提供截图或简短录制</li>
  <li>保持代码符合 <code>rustfmt</code> 规范</li>
</ul>

<h2>Wiki</h2>

<p>参见 <a href="https://github.com/MeowLynxSea/yororen-ui/wiki" target="_blank">Yororen UI Wiki</a></p>
