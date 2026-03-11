# 版本号管理脚本

## 概述

本项目使用统一的版本号管理方式，通过根目录下的 `Version` 文件控制多个文件中的版本号。

## 使用方法

### 1. 更新版本号

只需要修改项目根目录下的 `Version` 文件：

```text
0.1.3-dev
```

### 2. 运行更新脚本

在项目根目录执行：

```bash
npm run version:update
```

### 3. 自动更新的文件

脚本会自动同步以下文件中的版本号：

- `package.json`
- `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`
- `src/views/Home.vue`
- `src/views/Settings.vue`

## 版本号格式

- `Version` 文件：`0.1.2-dev`
- Vue 页面显示：`v0.1.2-dev`
- 其他配置文件：`0.1.2-dev`

## 注意事项

1. `Version` 文件只写版本号本身，不要带 `v`
2. 如果 `Cargo.lock` 更新失败，可以在下次 Rust 构建时重新生成
3. 推送标签后，GitHub Actions 会自动构建并发布版本

## 示例流程

```bash
# 1. 修改 Version 文件
echo "0.2.0" > Version

# 2. 同步版本号
npm run version:update

# 3. 提交代码
git add .
git commit -m "chore:更新版本号至v0.2.0"

# 4. 创建并推送标签
git tag v0.2.0
git push origin main --tags
```

## 脚本输出示例

```text
🚀 开始更新版本号...

📖 读取版本号: 0.1.2-dev
✅ 更新 package.json: 0.1.2-dev
✅ 更新 package-lock.json: 0.1.2-dev
✅ 更新 Cargo.toml: 0.1.2-dev
✅ 更新 Cargo.lock: 0.1.2-dev
✅ 更新 tauri.conf.json: 0.1.2-dev
✅ 更新 Home.vue: v0.1.2-dev
✅ 更新 Settings.vue: v0.1.2-dev

✨ 版本号更新完成！
📌 当前版本: 0.1.2-dev
```

## 故障排除

### 脚本执行失败

- 确认命令在项目根目录执行
- 确认 `Version` 文件存在且格式正确
- 确认目标文件存在且当前用户有写入权限

### `Cargo.lock` 更新失败

- 可以在 `src-tauri` 目录执行 `cargo build`
- 或直接运行 `npm run tauri build`
