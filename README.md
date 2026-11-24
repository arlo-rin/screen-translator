<div align="center">

<img src="assets/icon.svg" width="112" alt="Screen Translator">

# Screen Translator

**轻量快速的屏幕 OCR 即时翻译工具 —— 基于原生 Rust 构建，采用 Material Design 3 界面设计。**

[简体中文](README.md) · [English](README.en.md)

[![CI](https://github.com/arlo-rin/screen-translator/actions/workflows/ci.yml/badge.svg)](https://github.com/arlo-rin/screen-translator/actions/workflows/ci.yml)
[![Release](https://github.com/arlo-rin/screen-translator/actions/workflows/release.yml/badge.svg)](https://github.com/arlo-rin/screen-translator/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform: Windows | macOS | Linux](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-555555)](https://github.com/arlo-rin/screen-translator/releases)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-CE422B?logo=rust&logoColor=white)](https://www.rust-lang.org/)

[**核心特性**](#核心特性) ·
[**快速上手**](#快速上手) ·
[**快捷键与命令行**](#快捷键与命令行) ·
[**翻译引擎与文字识别**](#翻译引擎与文字识别) ·
[**系统环境要求**](#系统环境要求) ·
[**从源码构建**](#从源码构建) ·
[**版本与自动发布**](#版本与自动发布) ·
[**免责声明**](#免责声明) ·
[**致谢与协议**](#致谢与协议)

</div>

---

## <a id="核心特性"></a>✨ 核心特性

- **五种译文展示方式**
  - **覆盖在原文上** *（默认）* —— 直接在原文字位置覆盖译文，自动匹配字号与背景色调，不抢占游戏或应用焦点。
  - **字幕模式** —— 在屏幕底部或顶部显示电影字幕条，可自定义字号、底板不透明度，并支持在上方同时显示原文。
  - **选区旁弹出卡片** —— 半透明毛玻璃卡片，支持一键复制、语音朗读与切换持续监视。
  - **单独窗口** —— 原文与译文左右双栏对比，支持始终置于顶层。
  - **仅复制到剪贴板** —— 不在屏幕上显示浮窗，识别翻译后直接写入剪贴板。
- **持续监视（实时翻译）** —— 使用 `Alt+Shift+Q` 框选对话框或窗口一次，画面文字变化时自动重新识别并更新译文，无需反复按键。
- **高速本机 OCR（Windows）** —— 默认调用系统内置 `Windows.Media.Ocr`，几十毫秒即可识别一段对话且图像无需离开本机；本地未安装对应语言包或遇到复杂艺术字时可自动或手动切换至在线识别。
- **五大翻译引擎顺序回退** —— 支持 `AI 翻译`、`DeepL`、`Yandex`、`Google` 与 `阿里云机器翻译`，按设置列表顺序尝试直到返回结果。
- **多 AI 预设独立隔离与识图直译** —— 支持 OpenAI 兼容、Anthropic Messages 与 Google Gemini 三种协议及 9 个一键预设（含本地 `Ollama` 与 `LM Studio`），每个预设独立保存密钥、模型、附加指令与参数；支持开启**直接识图（Vision）**让多模态模型直接读取截图。
- **Google Material Design 3 界面** —— 浅色 / 深色 / 跟随系统自适应主题、无边框 M3 自定义顶部栏与圆形窗口控制按钮、互不重复的 M3 色阶导航图标，支持 **14 种界面语言**并优化了简体中文 UI 字体排版。
- **低内存占用** —— 纯原生 Rust（`eframe`/`egui`）编写，无内置浏览器内核；Windows 端对中/日/韩/拉丁系统字体采用只读内存映射（`MapViewOfFile`）零拷贝按需加载，后台托盘空闲时低频唤醒。

> **定制版说明**：当前版本已移除上游自动更新检查与捐赠入口，如需获取新版请直接前往 [Releases](https://github.com/arlo-rin/screen-translator/releases) 下载。

---

## <a id="快速上手"></a>🚀 快速上手

前往 [**Releases**](https://github.com/arlo-rin/screen-translator/releases) 下载对应平台的可执行文件（绿色免安装，直接运行）：

- Windows：`screen-translator-windows-x86_64.exe`
- macOS（通用版）：`screen-translator-macos-universal`
- Linux：`screen-translator-linux-x86_64`

1. 启动程序后会在 **系统托盘**（Windows/Linux）或 **菜单栏**（macOS）静默驻留。
2. 在屏幕任意位置按下 **`Alt+Q`**。
3. 拖动框选要翻译的文字区域（或按 **`Tab`** 在 **区域 / 窗口 / 全屏** 之间切换）。
4. 松开鼠标即可显示译文。
5. 点击屏幕其他位置或按 **`Esc`** 隐藏译文。

---

## <a id="快捷键与命令行"></a>⌨️ 快捷键与命令行

### 全局快捷键

可在 *设置 → 键盘快捷键* 中自由修改或清除：

| 默认快捷键 | 功能 |
|---|---|
| `Alt+Q` | 翻译框选区域 |
| `Alt+Shift+W` | 翻译点击的窗口 |
| `Alt+Shift+F` | 翻译光标所在整个屏幕 |
| `Alt+Shift+R` | 重复翻译上次捕获的区域 |
| `Alt+Shift+Q` | 开启 / 停止**持续监视**指定区域 |
| *（默认未设置）* | 隐藏当前译文（浮窗显示时按 `Esc` 也可隐藏） |

**支持单独绑定（无需组合 `Ctrl`/`Alt`/`Shift`）：** `F1`–`F24`、`Insert`、`Delete`、`Home`、`End`、`Page Up`、`Page Down`，以及 Windows 下的鼠标侧键 **`Mouse4 (Back)`** / **`Mouse5 (Forward)`**。

### 框选截图时

| 按键 / 操作 | 说明 |
|---|---|
| `Tab` | 循环切换模式：区域 → 窗口 → 全屏 |
| 按住 `Space` | 整体移动当前选区框 |
| **鼠标右键** | 放弃当前选区并重新框选 |
| `Esc` | 取消本次捕获 |

### 命令行参数与便携模式

```bash
screen-translator --settings              # 启动时直接打开设置窗口
screen-translator --settings engine       # 直接跳转到指定设置页
screen-translator --region 100,200,800,160  # 立即翻译指定矩形区域 (x,y,w,h)
screen-translator --help                  # 查看帮助信息
```

- **可用设置页名称**：`general`、`keys`、`languages`、`engine`、`appearance`、`history`、`logs`、`about`
- **便携版 / 多开配置隔离**：通过设置环境变量 `SAKURA_CONFIG_DIR` 指定配置文件与日志目录；单实例互斥锁会按配置目录隔离，支持同时运行多个独立配置实例。

---

## <a id="翻译引擎与文字识别"></a>🌐 翻译引擎与文字识别

### 翻译引擎

按设置界面中从上到下的启用顺序依次请求，首个成功返回的结果生效：

1. **AI 翻译** —— 支持 OpenAI 兼容、Anthropic Messages 与 Google Gemini 协议，内置 Anthropic Claude、OpenAI、Google Gemini、OpenRouter、DeepSeek、Groq、Mistral、Ollama（本地）、LM Studio（本地）预设。切换预设时各自配置相互隔离保存（API 地址、模型、加密密钥、温度、附加提示词、超时与识图开关）。
2. **DeepL** —— 需填写 API 密钥（支持免费版 `:fx` 与 Pro 密钥）。
3. **Yandex** *（默认启用，无需密钥）*。
4. **Google** *（默认启用，无需密钥）*。
5. **阿里云机器翻译（Alibaba Cloud MT）** —— 填写 AccessKey ID / Secret，可选默认接入点（`mt.aliyuncs.com`）、杭州、新加坡、利雅得或迪拜节点。

Windows 下所有 API 密钥均通过 DPAPI 加密后写入配置。

### 文字识别（OCR）

- **自动（默认）**：Windows 端优先使用系统自带本机语言包识别；未安装对应语言包或未识别到内容时自动回退到在线识别。
- **仅在本机**：严格使用本机 OCR，截图不会发送到任何网络服务。
- **联网**：始终使用在线 OCR（适合本机未安装对应语言包或字体高度艺术化的场景）。

---

## <a id="系统环境要求"></a>📋 系统环境要求

<details>
<summary><b>Windows（10 / 11 x64）</b></summary>

- 无需额外运行时依赖。
- 在 *Windows 设置 → 时间和语言 → 语言和区域* 中安装对应语言包即可启用更多语言的本机 OCR。
- 游戏场景下如使用**独占全屏**导致翻译层无法显示在游戏之上，请将游戏切换为**无边框窗口化**或窗口模式。

</details>

<details>
<summary><b>macOS（13+，Apple Silicon 与 Intel 通用）</b></summary>

首次使用需在 *系统设置 → 隐私与安全性* 中授予：
- **录屏与系统录音（Screen Recording）** —— 用于截取屏幕文字。
- **辅助功能（Accessibility）** —— 若全局快捷键未触发时开启。

应用以菜单栏代理身份运行（不占 Dock 栏图标，不创建独立桌面空间），可直接悬浮覆盖全屏应用。

</details>

<details>
<summary><b>Linux（X11 / Wayland）</b></summary>

以下桌面服务为可选项，存在时自动启用：
- **Secret Service**（GNOME Keyring / KWallet）用于加密存储密钥
- 支持 **AppIndicator** 的系统托盘面板
- **speech-dispatcher**（`spd-say`）用于译文朗读

</details>

---

## <a id="从源码构建"></a>🛠️ 从源码构建

### 环境依赖

- [Rust](https://rustup.rs/) 稳定版工具链
- **Windows**：Visual Studio Build Tools（MSVC）与 Windows SDK
- **macOS**：Xcode 命令行工具（`xcode-select --install`）
- **Linux（Debian/Ubuntu）**：
  ```bash
  sudo apt install pkg-config clang libclang-dev libgtk-3-dev \
    libxcb1-dev libxcb-randr0-dev libxcb-shm0-dev libxcb-xfixes0-dev \
    libxkbcommon-dev libwayland-dev libgl1-mesa-dev \
    libayatana-appindicator3-dev libpipewire-0.3-dev libdbus-1-dev \
    libgbm-dev libxdo-dev
  ```

### 编译与测试

```bash
cargo build --release   # 输出位置: target/release/screen-translator(.exe)
cargo test              # 运行单元测试
cargo test -- --ignored # 运行依赖真实屏幕的捕获测试
```

---

## <a id="版本与自动发布"></a>🔖 版本与自动发布

`Cargo.toml` 为版本号的唯一权威来源。推送至 `main` 分支会触发 `.github/workflows/release.yml`，自动构建 Windows、macOS Universal 与 Linux 三端产物并发布对应 `vX.Y.Z` Release。

---

## <a id="免责声明"></a>⚠️ 免责声明

本软件按**“现状”（AS IS）**提供，不作任何明示或默示担保。第三方翻译、OCR 与 AI 服务均受各自服务条款及计费规则约束，非官方接口可能随时失效或变更。

---

## <a id="致谢与协议"></a>👤 致谢与协议

- 原项目作者：[@クシススミタ](https://github.com/Kushisusumita)
- 定制版维护：[@arlo-rin](https://github.com/arlo-rin)
- 开源协议：[MIT](LICENSE)
