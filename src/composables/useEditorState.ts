import { ref } from 'vue'

/**
 * 编辑器状态 Composable
 * 管理编辑器的内容、光标位置、保存状态等
 */
export function useEditorState() {
  const fileContent = ref('')
  const hasUnsavedChanges = ref(false)
  const currentLine = ref(1)
  const currentColumn = ref(1)
  
  /**
   * 更新光标位置
   */
  function updateCursorPosition(textarea: HTMLTextAreaElement) {
    const text = textarea.value
    const cursorPos = textarea.selectionStart
    
    // 计算行号
    const textBeforeCursor = text.substring(0, cursorPos)
    const lines = textBeforeCursor.split('\n')
    currentLine.value = lines.length
    
    // 计算列号
    const currentLineText = lines[lines.length - 1]
    currentColumn.value = currentLineText.length + 1
  }
  
  /**
   * 内容变化处理
   */
  function onContentChange(content: string) {
    fileContent.value = content
    hasUnsavedChanges.value = true
  }
  
  /**
   * 重置未保存标记
   */
  function resetUnsavedChanges() {
    hasUnsavedChanges.value = false
  }
  
  return {
    fileContent,
    hasUnsavedChanges,
    currentLine,
    currentColumn,
    updateCursorPosition,
    onContentChange,
    resetUnsavedChanges
  }
}
