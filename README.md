# Hearts of Iron IV - Code Studio(停止维护)

<div align="center">
  <img src="app-icon.png" alt="HOI4 Code Studio" width="96" />
  <p>面向《钢铁雄心 IV》Mod 制作的桌面开发工具</p>
  <p>
    <img src="https://img.shields.io/badge/Vue-3.5.13-4FC08D?style=flat-square&logo=vue.js" alt="Vue" />
    <img src="https://img.shields.io/badge/Tauri-2.x-FFC131?style=flat-square&logo=tauri" alt="Tauri" />
    <img src="https://img.shields.io/badge/Rust-1.90+-000000?style=flat-square&logo=rust" alt="Rust" />
    <img src="https://img.shields.io/badge/TypeScript-5.x-3178C6?style=flat-square&logo=typescript" alt="TypeScript" />
  </p>
</div>

HOI4 Code Studio 是一个基于 Tauri + Vue 3 + Rust 的Windows桌面应用，目标是提供接近 IDE 的 HOI4 Mod 开发体验：项目管理、脚本编辑、预览工具、依赖管理、插件扩展与 AI 辅助在同一工作区内完成。

## 核心能力

- 项目工作流：创建项目、打开项目、初始化普通目录为 HOICS 项目、最近项目历史与统计信息。
- 多面板编辑器：文件树、标签页、多窗格分栏、右键菜单、自动保存。
- 搜索与定位：项目/游戏目录/依赖三种范围搜索，支持正则与大小写选项，并支持结果定位。
- 语法与校验：CodeMirror 高亮、HOI4 脚本错误收集、括号匹配、Tag/Idea 索引。
- 预览工具：
  - 事件关系图（Event Graph）
  - 国策树预览与可视化编辑（Focus Tree）
  - GUI/GFX 预览
  - MIO 预览
  - 地图预览（default.map + 省份/州数据）
  - 图片与 DDS/TGA 资源读取
- 游戏与发布：一键启动游戏、依赖项管理、项目打包导出。
- 可扩展性：插件系统（`About.hoics` + iframe + command 白名单）与插件面板/工具栏贡献。
- AI 集成：施工中...
- 错误提示：不完善...

## 技术架构

### 前端

- Vue 3 + TypeScript + Vue Router
- Vite 构建
- CodeMirror 6 编辑器
- Cytoscape.js 图形渲染（事件图、国策树）
- Tailwind CSS 4

### 后端（Tauri / Rust）

- Tauri 2 命令桥接
- 项目与文件系统服务（创建、读取、重命名、删除、搜索、打包）
- JSON 工具链（解析、校验、路径访问、深度合并）
- HOI4 专用解析与索引（Tag、Idea、国策本地化、GUI/GFX、MIO、地图）
- 插件安装与校验、主题管理、图标缓存

## 目录说明

```text
.
├─ src/                # Vue 前端
├─ src-tauri/          # Rust 后端与 Tauri 配置
├─ docs/               # 项目文档与 API 文档
├─ sample-plugins/     # 插件示例
├─ scripts/            # 构建与版本脚本
└─ README.md
```

## 快速开始

### 环境要求

- Node.js 18+
- npm
- Rust 1.90+

### 安装依赖

```bash
npm install
```

### 开发运行

```bash
npm run tauri dev
```

### 构建应用

```bash
npm run tauri build
```

Windows 构建产物常见位置：

- 可执行文件：`src-tauri/target/release/HOI4 Code Studio.exe`
- 安装包：`src-tauri/target/release/bundle/msi/HOI4_Code_Studio_x64_en-US.msi`

## 测试

### 前端测试

```bash
npm run test:run
npm run test:coverage
npm run test:ui
```

### 后端测试

```bash
cd src-tauri
cargo test
```

## 文档入口

- 总览文档：`docs/README.md`
- API 文档：`docs/API/README.md`
- 前端功能文档：`docs/Frontend/`
- 后端说明：`docs/Backend/`

## 贡献

1. 提交 Issue（缺陷、需求、文档问题）
2. Fork 并创建分支开发
3. 保持代码与文档同步更新
4. 提交 Pull Request

推荐开发环境：

- VS Code
- Vue - Official（Volar）
- rust-analyzer
- Tauri VS Code 插件

## 许可证

[MIT License](LICENSE)

## 仓库链接

- 项目主页：[https://github.com/cybercyberlieflife/HOI4-Code-Studio](https://github.com/cybercyberlieflife/HOI4-Code-Studio)
- Issues：[https://github.com/cybercyberlieflife/HOI4-Code-Studio/issues](https://github.com/cybercyberlieflife/HOI4-Code-Studio/issues)
- Discussions：[https://github.com/cybercyberlieflife/HOI4-Code-Studio/discussions](https://github.com/cybercyberlieflife/HOI4-Code-Studio/discussions)

## 免责声明

本项目为非官方第三方工具，与 Paradox Interactive 无关。Hearts of Iron IV 为 Paradox Interactive 的注册商标。
