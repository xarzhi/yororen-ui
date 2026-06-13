# Yororen UI

<p align="center">
  <strong>中文版</strong> | <a href="README.md">English</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="许可证">
  <img src="https://img.shields.io/badge/rust-edition%202024-yellow.svg" alt="Rust 版本">
  <img src="https://img.shields.io/badge/gpui-based-2a2a2a.svg" alt="基于 gpui">
</p>

<p align="center">
  <strong>Yororen UI</strong> 是一个 headless 优先的 Rust UI 库，基于 <a href="https://github.com/zed-industries/zed"><code>gpui</code></a>（Zed）构建。它采用三层架构 —— headless 原语、JSON 主题、可替换的可视化渲染器 —— 让应用在不同设计风格之间自由切换。
</p>

<p align="center">
  它旨在被 <code>gpui</code> 应用程序 crate 使用，同时保持 UI 层的独立性（主题、组件、小部件和嵌入式图标资源）。
</p>

---

## 特性

<table>
  <tr>
    <td><strong>54 个组件</strong></td>
    <td>按钮、输入框、徽章、工具提示、图标、标题、卡片、模态框、树形控件、虚拟化列表等</td>
  </tr>
  <tr>
    <td><strong>三层架构</strong></td>
    <td>Headless 原语 + JSON 主题 + 可替换的可视化渲染器（默认 + brutalism）</td>
  </tr>
  <tr>
    <td><strong>主题系统</strong></td>
    <td>主题就是纯 JSON —— 通过一次 <code>install()</code> 调用即可在运行时切换配色</td>
  </tr>
  <tr>
    <td><strong>动画系统</strong></td>
    <td>可配置的动画，包含预设、缓动函数和编排器</td>
  </tr>
  <tr>
    <td><strong>国际化</strong></td>
    <td>多语言支持（英文、中文、阿拉伯文），文本方向支持（LTR/RTL）</td>
  </tr>
  <tr>
    <td><strong>无障碍</strong></td>
    <td>ARIA 支持、焦点管理、键盘导航、焦点陷阱</td>
  </tr>
  <tr>
    <td><strong>嵌入式资源</strong></td>
    <td>20+ 个 SVG 图标，通过 <code>rust-embed</code> 嵌入（<code>assets/icons/**</code>）</td>
  </tr>
  <tr>
    <td><strong>通知系统</strong></td>
    <td>Toast 通知，包含多种样式、队列管理和交互操作</td>
  </tr>
</table>

---

## 快速开始

### 1) 安装渲染器

```rust
use gpui::App;
use yororen_ui::renderer;

fn init_ui(cx: &mut App) {
    // 加载与系统外观匹配的默认主题（浅色/深色），
    // 并注册全部 54 个默认 TokenXxxRenderer 实现。
    renderer::install(cx, cx.window_appearance());
}
```

### 2) 使用 headless 组件

```rust
use gpui::{App, AppContext, Entity, Global};
use yororen_ui::headless::button::button;
use yororen_ui::headless::label::label;

#[derive(Default)]
pub struct Counter { pub value: i32 }
pub struct AppState { pub counter: Entity<Counter> }

impl AppState {
    pub fn new(cx: &mut App) -> Self {
        Self { counter: cx.new(|_| Counter::default()) }
    }
}
impl Global for AppState {}

fn render(cx: &mut gpui::Context<MyApp>) -> impl gpui::IntoElement {
    let count = cx.global::<AppState>().counter.read(cx).value;
    let inc = cx.global::<AppState>().counter.clone();

    gpui::div().size_full().flex().items_center().justify_center().gap_2()
        .child(label("count", count.to_string(), cx).render(cx))
        .child(
            button("inc", cx)
                .on_click(move |_, _, cx| {
                    inc.update(cx, |c, cx| { c.value += 1; cx.notify(); });
                })
                .render(cx)
                .child("+"),
        )
}
```

### 3) 安装 i18n（Locale + RTL）

```rust
use gpui::App;
use yororen_ui::locale_en;

fn init_i18n(cx: &mut App) {
    locale_en::install(cx);
}
```

### 4) 提供资源（图标）

```rust
use gpui::Application;
use yororen_ui::assets::UiAsset;

let app = Application::new().with_assets(UiAsset);
```

如果您的应用程序也有自己的资源，使用 <code>CompositeAssetSource</code> 将 Yororen UI 的图标作为后备层组合进来。

---

## 示例应用程序

我们在 <code>crates/yororen-ui-demos/</code> 下提供五个官方示例应用：

| 示例 | 展示内容 | 运行命令 |
|------|----------|----------|
| <a href="#counter">Counter（计数器）</a> | 最小启动模板，单一 <code>Entity&lt;T&gt;</code> 全局，三个按钮 | <code>cargo run -p counter-demo</code> |
| <a href="#layers-demo">Layers Demo（三层演示）</a> | 并排展示三种渲染路径：headless / 默认渲染 / 自定义绘制器 | <code>cargo run -p layers-demo</code> |
| <a href="#inputs-demo">Inputs Demo（输入演示）</a> | 七个文本输入组件，使用 <code>cx.entity().clone()</code> 闭包模式 | <code>cargo run -p inputs-demo</code> |
| <a href="#gallery-demo">Gallery Demo（画廊演示）</a> | 全套 54 组件展示、主题切换、国际化、通知、虚拟化列表 | <code>cargo run -p gallery-demo</code> |
| <a href="#theme-showcase">Theme Showcase（主题演示）</a> | 通过 <code>theme::install</code> 实现运行时主题切换 | <code>cargo run -p theme-showcase-demo</code> |

### Counter（计数器）

<!-- Screenshot slot: replace demo/screenshots/counter.png when refreshing -->
<img src="demo/screenshots/counter.png" alt="Counter Demo" width="600">

一个极简的计数器应用，演示 Yororen UI 最基础的核心概念。

**核心功能：**
- 单一 <code>Entity&lt;Counter&gt;</code> 全局状态
- 通过 <code>on_click</code> 处理按钮点击
- 通过 <code>cx.notify()</code> 实现响应式 UI 更新

**适用场景：** Yororen UI 初学者的第一个学习示例。

### Layers Demo（三层演示）

<!-- Screenshot slot: replace demo/screenshots/layers-demo.png when refreshing -->
<img src="demo/screenshots/layers-demo.png" alt="Layers Demo" width="600">

并排对比三种渲染路径：纯 headless（调用方自绘）、默认渲染器（主题 JSON）、手写的 <code>MaterialButton</code> 绘制器（带真正的涟漪动画）。

**核心功能：**
- 同一个 headless <code>button</code> 用三种方式渲染
- 自定义的 <code>gpui::Element</code> 绘制器（见 <code>material_button.rs</code>）
- 第四个面板展示使用 <code>.render(cx, window)</code> 的文本输入

**适用场景：** 理解 <code>.apply</code>（仅 a11y）和 <code>.render(cx)</code>（完整视觉）的区别。

### Inputs Demo（输入演示）

<!-- Screenshot slot: replace demo/screenshots/inputs-demo.png when refreshing -->
<img src="demo/screenshots/inputs-demo.png" alt="Inputs Demo" width="600">

七个面板，每个展示一个文本输入组件。展示规范的 <code>cx.entity().clone()</code> 模式来连接 <code>on_change</code> 闭包。

**核心功能：**
- 全套七个文本输入：TextInput, PasswordInput, NumberInput, TextArea, SearchInput, FilePathInput, KeybindingInput
- <code>number_input</code> 的步进回调
- <code>KeybindingInputMode</code> 状态切换（Idle / Capturing）

**适用场景：** 构建表单、设置页等输入密集型界面。

### Gallery Demo（画廊演示）

<!-- Screenshot slot: replace demo/screenshots/gallery-demo.png when refreshing -->
<img src="demo/screenshots/gallery-demo.png" alt="Gallery Demo" width="600">

厨房水槽式参考。一个窗口渲染所有组件，带有可用的主题切换器（默认 vs brutalism，浅色 vs 深色）和语言切换器（en / zh-CN / ar）。

**核心功能：**
- 一个窗口展示全部 54 个组件
- <code>cell()</code> / <code>input_cell()</code> 辅助函数将每个组件包裹在带状态行的标签卡中
- 通过工具栏实时切换主题和语言
- 使用 <code>gpui::deferred(...).with_priority(3)</code> 渲染的 <code>NotificationCenter</code> 宿主

**适用场景：** 任何非平凡模式 —— 构建真实应用时从这里开始。

### Theme Showcase（主题演示）

<!-- Screenshot slot: replace demo/screenshots/theme-showcase.png when refreshing -->
<img src="demo/screenshots/theme-showcase.png" alt="Theme Showcase" width="600">

单个窗口演示运行时主题切换：复用同一个 headless 按钮和 <code>TokenButtonRenderer</code>，每次点击 "Next" 时替换渲染器读取的 JSON。

**核心功能：**
- 内置 <code>system-light</code> / <code>system-dark</code> 主题
- 内联 CATPPUCCIN 和 MATERIAL 主题（用户自定义 JSON）
- 一键轮换主题

**适用场景：** 构建 "Next theme" 工具栏，A/B 主题测试。

---

## 基于 Yororen UI 构建

使用 Yororen UI 构建的项目和应用程序。

### Yororen Accelerator

<img src="demo/screenshots/accelerator-1.png" alt="Yororen Accelerator" width="380">
<img src="demo/screenshots/accelerator-2.png" alt="Yororen Accelerator" width="380">

<img src="demo/screenshots/accelerator-3.png" alt="Yororen Accelerator" width="380">
<img src="demo/screenshots/accelerator-4.png" alt="Yororen Accelerator" width="380">

一个网络加速工具，具备本机透明 TCP 导流 + relay 透传功能，使用 Yororen UI 构建。

**核心亮点：**
- 复杂的数据仪表板，实时统计显示
- 自定义窗口边框，接近原生体验
- 丰富的数据表格和虚拟化列表
- 服务器管理和配置界面

---

## 包含内容

### Crate 列表

<table>
  <tr>
    <td><code>yororen-ui-core</code></td>
    <td>Headless 原语、主题 JSON 访问、i18n、a11y、RTL、动画、资源、通知中心</td>
  </tr>
  <tr>
    <td><code>yororen-ui-default-renderer</code></td>
    <td>54 个 <code>TokenXxxRenderer</code> 默认实现 + 内置 <code>system-light.json</code> / <code>system-dark.json</code> 主题 + <code>renderer::install</code> 引导</td>
  </tr>
  <tr>
    <td><code>yororen-ui-brutalism-renderer</code></td>
    <td>备选渲染器 —— 锐角、粗黑边框、硬偏移阴影、等宽字体 + 内置 brutalism 主题</td>
  </tr>
  <tr>
    <td><code>yororen-ui</code></td>
    <td>聚合 crate，重新导出三个层 + 语言目录（<code>en</code>、<code>zh-CN</code>、<code>ar</code>）</td>
  </tr>
</table>

### 三层架构

```
headless 组件  ──▶  主题（JSON）  ──▶  渲染器（视觉）
       ▲                                          │
       └──────── 安装时注册 ─────────────────────┘
```

- **Headless**：数据 + 控制 + a11y。无视觉。
- **主题**：单个 <code>serde_json::Value</code>，可在运行时切换。
- **渲染器**：每个组件一个 trait，读取主题并生成可视化 div。

自定义渲染器只需实现全部 54 个 <code>XxxRenderer</code> trait —— 完全不需要触碰 headless 层。

### 组件分类

| 分类 | 组件 |
|------|------|
| **基础** | Button, IconButton, Icon, Label, Text, Heading, Spacer, Divider, Card, FocusRing |
| **输入** | TextInput, PasswordInput, NumberInput, TextArea, SearchInput, FilePathInput, KeybindingInput |
| **选择** | Checkbox, Radio, RadioGroup, Switch, Slider, Select, ComboBox |
| **展示** | Badge, Avatar, Image, ProgressBar, Skeleton, Tag, EmptyState |
| **浮层** | Tooltip, Popover, Modal, Toast, DropdownMenu, Menu, Disclosure, Overlay |
| **表面** | Panel, Card, Tooltip, Avatar, Image |
| **列表 / 表格** | ListItem, TreeItem, Tree, Table, Form, FormField, VirtualList, UniformVirtualList |
| **交互** | ToggleButton, SplitButton, ButtonGroup, ShortcutHint, KeybindingDisplay, DragHandle |

### 图标

```rust
use yororen_ui::icon::IconName;
// 作为任何接受 icon 的组件的图标源。
```

图标路径映射到 <code>assets/icons/</code> 下嵌入的 SVG 资源。业务专属图标可以放入您自己的 <code>AssetSource</code>，通过 icon API 引用。

---

## 环境要求

<ul>
  <li><strong>Rust edition：</strong> 2024</li>
  <li><code>gpui</code> 通过 crates.io 上的 <a href="https://crates.io/crates/gpui-ce"><code>gpui-ce</code></a> crate 提供</li>
</ul>

---

## 安装

### 从 crates.io 使用（推荐）

```toml
[dependencies]
yororen_ui = "0.3"
```

### 从 GitHub 使用（最新开发版）

```toml
[dependencies]
yororen_ui = { git = "https://github.com/MeowLynxSea/yororen-ui.git", tag = "v0.3.0" }
```

### 从本地路径使用（开发时）

```toml
[dependencies]
yororen_ui = { path = "../yororen-ui" }
```

---

## 依赖项

<code>gpui-ce</code> 通过 crates.io 进行分发，使用语义版本控制。请确保您的应用程序使用兼容的版本：

```toml
[dependencies]
gpui = { package = "gpui-ce", version = "0.3" }
```

在本仓库中，<code>gpui-ce</code> 已在 <code>Cargo.toml</code> 中指定。

---

## 许可证

<ul>
  <li>Yororen UI 采用 <strong>Apache License, Version 2.0</strong> 授权。参见 <code>LICENSE</code>。</li>
  <li>本项目基于 <code>gpui</code>（Zed Industries）构建，同样采用 Apache-2.0 许可证。</li>
</ul>

归属详情请参见 <code>NOTICE</code>。

---

## 贡献

欢迎提交 Issue 和 PR。

修改视觉效果时：

<ul>
  <li>请提供截图或简短录制</li>
  <li>保持代码符合 <code>rustfmt</code> 规范</li>
</ul>

---

## Wiki

参见 <a href="https://github.com/MeowLynxSea/yororen-ui/wiki" target="_blank">Yororen UI Wiki</a> 获取详细文档、指南和组件参考。