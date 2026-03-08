# 自动保存功能 (Auto Save)

## 概述
自动保存功能允许在编辑文件时自动保存更改，无需手动按下 `Ctrl+S` 或点击保存按钮。

## 核心功能
- **自动保存开关**: 在设置页面和编辑器工具栏都可以切换自动保存功能
- **快速响应**: 停止输入0.1秒后自动保存文件
- **防抖机制**: 避免频繁的文件写入操作，提高性能
- **视觉反馈**: 
  - 工具栏按钮启用时显示绿色，禁用时显示灰色
  - 文件标签页上的红点表示未保存状态
- **持久化配置**: 自动保存设置会保存到配置文件，重启应用后保留

## 使用方式

### 启用/禁用自动保存
1. **在设置页面**:
   - 打开设置页面
   - 在"应用设置"区域找到"启用自动保存"复选框
   - 勾选或取消勾选即可切换

2. **在编辑器工具栏**:
   - 点击工具栏中的保存图标按钮
   - 绿色表示已启用，灰色表示已禁用
   - 点击即可切换状态

### 编辑文件
- 启用自动保存后，打开任意文件开始编辑
- 文件修改后，标签页会显示红点（未保存标记）
- 停止输入0.1秒后，文件会自动保存
- 红点消失，表示文件已保存成功

### 查看日志
在 Tauri 后端日志中可以看到文件保存操作：
```
写入文件成功: <文件路径>
```

## 技术实现

### 前端组件
- **Settings.vue**: 设置页面，提供自动保存复选框
- **EditorToolbar.vue**: 编辑器工具栏，提供自动保存切换按钮
- **Editor.vue**: 编辑器页面，管理自动保存状态
- **EditorGroup.vue**: 编辑器组，实现自动保存逻辑

### 核心逻辑 (EditorGroup.vue)
```typescript
// 自动保存防抖计时器
const autoSaveTimers = ref<Map<string, number>>(new Map())
const AUTO_SAVE_DELAY = 100 // 0.1秒防抖

// 处理内容变化
function handleContentChange(paneId: string, content: string) {
  const pane = panes.value.find(p => p.id === paneId)
  if (!pane || pane.activeFileIndex === -1) return
  
  const file = pane.openFiles[pane.activeFileIndex]
  if (file) {
    if (file.content !== content) {
      file.content = content
      file.hasUnsavedChanges = true
      
      // 如果启用了自动保存，设置防抖计时器
      if (props.autoSave) {
        // 清除旧的计时器
        const existingTimer = autoSaveTimers.value.get(paneId)
        if (existingTimer) {
          clearTimeout(existingTimer)
        }
        
        // 设置新的计时器
        const timer = window.setTimeout(() => {
          handleSaveFile(paneId)
          autoSaveTimers.value.delete(paneId)
        }, AUTO_SAVE_DELAY)
        
        autoSaveTimers.value.set(paneId, timer)
      }
    }
  }
}
```

### 配置持久化
配置保存在系统配置文件中：
```json
{
  "gameDirectory": "...",
  "autoSave": true,
  "checkForUpdates": true,
  ...
}
```

### 后端 API
- `loadSettings()`: 加载配置，包含 `autoSave` 字段
- `saveSettings(settings)`: 保存配置
- `writeFileContent(path, content)`: 写入文件内容

## 设计考量

### 防抖延迟
- **延迟时间**: 0.1秒（100毫秒）
- **优势**: 平衡了响应速度和性能
  - 足够短，用户几乎感觉不到延迟
  - 足够长，避免每次按键都触发保存
- **机制**: 连续输入只会在最后一次输入后0.1秒保存一次

### 用户体验
- **双重访问**: 在设置页面和编辑器都可以切换，方便快捷
- **视觉反馈**: 绿色/灰色按钮清晰表示状态
- **即时生效**: 切换开关后立即生效，无需重启
- **无干扰**: 自动保存在后台静默进行，不打断用户工作流

### 性能优化
- **防抖机制**: 避免频繁的文件 I/O 操作
- **计时器管理**: 使用 Map 管理多个窗格的计时器
- **按需保存**: 只有内容真正改变时才触发保存

## 注意事项
- 自动保存会按当前编辑内容正常写入，包括从游戏目录打开的文件
- 禁用自动保存后，需要手动保存文件（`Ctrl+S`）
- 关闭文件或应用时，未保存的更改会提示用户
