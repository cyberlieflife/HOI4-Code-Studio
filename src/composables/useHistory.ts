import { ref, nextTick } from 'vue'

/**
 * 历史状态接口
 */
export interface HistoryState {
  content: string
  cursorStart: number
  cursorEnd: number
}

/**
 * 撤销/重做 Composable
 * 管理编辑器的撤销和重做功能
 */
export function useHistory() {
  const undoStack = ref<HistoryState[]>([])
  const redoStack = ref<HistoryState[]>([])
  const isApplyingHistory = ref(false)
  
  /**
   * 保存当前状态到撤销栈
   */
  function saveHistory(state: HistoryState) {
    if (isApplyingHistory.value) return
    
    undoStack.value.push(state)
    // 限制栈大小为 100
    if (undoStack.value.length > 100) {
      undoStack.value.shift()
    }
    // 清空重做栈
    redoStack.value = []
  }
  
  /**
   * 撤销操作
   */
  function undo(
    textarea: HTMLTextAreaElement,
    onContentChange: (content: string) => void,
    onHighlight?: () => void
  ) {
    if (undoStack.value.length === 0) return
    
    // 保存当前状态到重做栈
    const currentState: HistoryState = {
      content: textarea.value,
      cursorStart: textarea.selectionStart,
      cursorEnd: textarea.selectionEnd
    }
    redoStack.value.push(currentState)
    
    // 恢复上一个状态
    const previousState = undoStack.value.pop()
    if (previousState) {
      isApplyingHistory.value = true
      onContentChange(previousState.content)
      
      nextTick(() => {
        textarea.value = previousState.content
        textarea.setSelectionRange(previousState.cursorStart, previousState.cursorEnd)
        textarea.focus()
        isApplyingHistory.value = false
        onHighlight?.()
      })
    }
  }
  
  /**
   * 重做操作
   */
  function redo(
    textarea: HTMLTextAreaElement,
    onContentChange: (content: string) => void,
    onHighlight?: () => void
  ) {
    if (redoStack.value.length === 0) return
    
    // 保存当前状态到撤销栈
    const currentState: HistoryState = {
      content: textarea.value,
      cursorStart: textarea.selectionStart,
      cursorEnd: textarea.selectionEnd
    }
    undoStack.value.push(currentState)
    
    // 恢复下一个状态
    const nextState = redoStack.value.pop()
    if (!nextState) return
    
    isApplyingHistory.value = true
    onContentChange(nextState.content)
    
    nextTick(() => {
      textarea.value = nextState.content
      textarea.setSelectionRange(nextState.cursorStart, nextState.cursorEnd)
      textarea.focus()
      isApplyingHistory.value = false
      onHighlight?.()
    })
  }
  
  /**
   * 清空历史记录
   */
  function clearHistory() {
    undoStack.value = []
    redoStack.value = []
  }
  
  return {
    undoStack,
    redoStack,
    isApplyingHistory,
    saveHistory,
    undo,
    redo,
    clearHistory
  }
}
