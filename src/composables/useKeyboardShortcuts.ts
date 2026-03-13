import { onMounted, onBeforeUnmount } from 'vue'

/**
 * 键盘快捷键 Composable
 * 处理全局快捷键事件
 */
export function useKeyboardShortcuts(handlers: {
  save?: () => void
  undo?: () => void
  redo?: () => void
  search?: () => void
  copy?: () => boolean | void
  cut?: () => boolean | void
  paste?: () => boolean | void
  nextError?: () => void
  previousError?: () => void
  toggleTheme?: () => void
  toggleIconPanel?: () => void
}) {
  function shouldHandleClipboardShortcut(handler?: () => boolean | void) {
    if (!handler) {
      return false
    }

    return handler() !== false
  }

  function handleKeyDown(e: KeyboardEvent) {
    // Ctrl+S 保存
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault()
      handlers.save?.()
    }
    
    // Ctrl+E 下一个错误
    if ((e.ctrlKey || e.metaKey) && e.key === 'e') {
      e.preventDefault()
      handlers.nextError?.()
    }

    // Ctrl+R 上一个错误
    if ((e.ctrlKey || e.metaKey) && e.key === 'r') {
      e.preventDefault()
      handlers.previousError?.()
    }

    // Ctrl+F 搜索
    if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key === 'f') {
      e.preventDefault()
      handlers.search?.()
    }

    if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key === 'c') {
      if (shouldHandleClipboardShortcut(handlers.copy)) {
        e.preventDefault()
      }
    }

    if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key === 'x') {
      if (shouldHandleClipboardShortcut(handlers.cut)) {
        e.preventDefault()
      }
    }

    if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key === 'v') {
      if (shouldHandleClipboardShortcut(handlers.paste)) {
        e.preventDefault()
      }
    }

    // Ctrl+Shift+T 切换主题面板
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'T') {
      e.preventDefault()
      handlers.toggleTheme?.()
    }
    
    // Ctrl+Shift+Y 切换图标面板
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'Y') {
      e.preventDefault()
      handlers.toggleIconPanel?.()
    }
    
    // Ctrl+Z 撤销
    if ((e.ctrlKey || e.metaKey) && e.key === 'z') {
      if (e.shiftKey) {
        // Ctrl+Shift+Z 与 Ctrl+Y 交给重做处理
        handlers.redo?.()
      } else {
        handlers.undo?.()
      }
      // 不调用 preventDefault，让浏览器保留默认行为
    }
    
    // Ctrl+Y 重做
    if ((e.ctrlKey || e.metaKey) && e.key === 'y') {
      handlers.redo?.()
      // 同样不调用 preventDefault
    }
  }
  
  onMounted(() => {
    document.addEventListener('keydown', handleKeyDown)
  })
  
  onBeforeUnmount(() => {
    document.removeEventListener('keydown', handleKeyDown)
  })
  
  return {
    handleKeyDown
  }
}
