# MyWordsTool

我的程序员英语翻译练习工具：输入中文，得到贴合 Java 全栈程序员习惯的英文翻译和关键词讲解，然后**自己把英文手动敲进 Cursor**。手动敲就是练习。

文档：

- `docs/需求背景文档.md` — 为什么做、做什么、不做什么
- `docs/产品PRD.md` — 界面、交互、每一步发生什么
- `docs/视觉规范.md` — 颜色 / 字号 / 间距 token，`docs/result-area-sample.html` 是翻译结果区的静态样例

## 技术栈

Tauri 2 + Vue 3 + TypeScript + Naive UI（前端），Rust + rusqlite + reqwest（后端）。一期只出 Windows 包，代码不引入平台专属依赖，为 macOS 预留。

## 开发环境

1. [Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/)，勾选「使用 C++ 的桌面开发」
2. [Rust](https://rustup.rs/)，装完 `cargo --version` 能出版本号
3. Node.js 20+
4. WebView2 运行时（Win10/11 一般自带）

```powershell
npm install
npm run tauri dev      # 开发：热更新
npm run tauri build    # 打包：src-tauri/target/release/bundle/nsis/
```

首次运行 `tauri dev` 会编译全部 Rust 依赖，约 3~8 分钟；之后增量编译很快。

## 首次使用

1. 托盘图标或 `Ctrl+Shift+E` 唤起窗口
2. 底栏「设置」→「模型」→ 编辑 DeepSeek → 粘贴 API Key → 测试连接 → 保存
3. 回到翻译页，输入中文，`Ctrl+Enter`

API Key 存在 Windows 凭据管理器，不在任何文件里。

## 目录

```
src/                    前端
  api/                  与 Rust 命令一一对应的调用封装与类型
  stores/               pinia：settings（本机设置）、translate（翻译状态）
  composables/          useTheme / useTts / useHotkeys
  components/           TopBar / BottomBar / PageHeader / ResultArea
  views/                Translate / History / Favorites / Settings
  styles/tokens.css     视觉规范里的 CSS 变量
src-tauri/src/          后端
  lib.rs                Tauri 启动、插件、命令注册
  commands.rs           全部 #[tauri::command]
  db/                   SQLite：schema.sql、models、查询
  llm/                  LlmProvider trait + OpenAI 兼容适配器
  translate.rs          提示词拼装、模型输出解析、分句
  secrets.rs            API Key 存系统钥匙串
  tray.rs               托盘菜单
```

## 数据

- SQLite：`%APPDATA%\com.jackliu.codeenglish\codeenglish.db`（设置页可打开目录）
- 本机设置：同目录 `settings.json`
- 导出：设置页「导出数据」→ 下载目录 `codeenglish-export-*.json`
