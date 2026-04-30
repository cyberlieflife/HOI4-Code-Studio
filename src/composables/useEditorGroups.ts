import { ref, computed } from 'vue'
import type { OpenFile } from './useFileManager'
import { toast } from '../utils/notification'

/**
 * 编辑器窗格接口
 */
export interface EditorPane {
  id: string
  openFiles: OpenFile[]
  activeFileIndex: number
  width: number // 百分比宽度 (0-100)
}

/**
 * 编辑器分组管理 Composable
 * 管理编辑器的分页功能，最多支持3个窗格
 */
export function useEditorGroups() {
  const panes = ref<EditorPane[]>([
    {
      id: generatePaneId(),
      openFiles: [],
      activeFileIndex: -1,
      width: 100
    }
  ])
  
  const activePaneId = ref<string>(panes.value[0].id)
  
  /**
   * 生成唯一的窗格ID
   */
  function generatePaneId(): string {
    return `pane-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
  }
  
  /**
   * 获取活动窗格
   */
  const activePane = computed(() => {
    return panes.value.find(p => p.id === activePaneId.value) || panes.value[0]
  })
  
  /**
   * 获取活动窗格索引
   */
  const activePaneIndex = computed(() => {
    return panes.value.findIndex(p => p.id === activePaneId.value)
  })
  
  /**
   * 分割窗格（向右分割）
   * @param sourcePaneId 源窗格ID
   * @param fileIndex 要移动到新窗格的文件索引（可选）
   */
  function splitPane(sourcePaneId: string, fileIndex?: number): boolean {
    if (panes.value.length >= 3) {
      toast.warning('最多只能分割为3个窗格')
      return false
    }
    
    const sourcePane = panes.value.find(p => p.id === sourcePaneId)
    if (!sourcePane) return false
    
    // 计算新的宽度分配
    const newWidth = sourcePane.width / 2
    sourcePane.width = newWidth
    
    // 创建新窗格
    const newPane: EditorPane = {
      id: generatePaneId(),
      openFiles: [],
      activeFileIndex: -1,
      width: newWidth
    }
    
    // 如果指定了文件索引，将该文件移动到新窗格
    if (fileIndex !== undefined && fileIndex >= 0 && fileIndex < sourcePane.openFiles.length) {
      const file = sourcePane.openFiles[fileIndex]
      newPane.openFiles = [{ ...file }]
      newPane.activeFileIndex = 0
    }
    
    // 在源窗格后面插入新窗格
    const sourceIndex = panes.value.findIndex(p => p.id === sourcePaneId)
    panes.value.splice(sourceIndex + 1, 0, newPane)
    
    // 激活新窗格
    activePaneId.value = newPane.id
    
    return true
  }
  
  /**
   * 关闭窗格
   * @param paneId 窗格ID
   */
  function closePane(paneId: string): boolean {
    if (panes.value.length <= 1) {
      return false // 至少保留一个窗格
    }
    
    const paneIndex = panes.value.findIndex(p => p.id === paneId)
    if (paneIndex === -1) return false
    
    const pane = panes.value[paneIndex]
    
    // 检查是否有未保存的更改
    const hasUnsaved = pane.openFiles.some(f => f.hasUnsavedChanges)
    if (hasUnsaved) {
      if (!confirm('该窗格中有文件包含未保存的更改，是否关闭？')) {
        return false
      }
    }
    
    // 将宽度分配给其他窗格
    const widthToDistribute = pane.width
    const remainingPanes = panes.value.filter(p => p.id !== paneId)
    const widthPerPane = widthToDistribute / remainingPanes.length
    
    remainingPanes.forEach(p => {
      p.width += widthPerPane
    })
    
    // 移除窗格
    panes.value.splice(paneIndex, 1)
    
    // 如果关闭的是活动窗格，切换到相邻窗格
    if (paneId === activePaneId.value) {
      const newActiveIndex = Math.min(paneIndex, panes.value.length - 1)
      activePaneId.value = panes.value[newActiveIndex].id
    }
    
    return true
  }
  
  /**
   * 调整窗格宽度
   * @param paneId 窗格ID
   * @param newWidth 新宽度（百分比）
   */
  function resizePaneWidth(paneId: string, newWidth: number): boolean {
    const paneIndex = panes.value.findIndex(p => p.id === paneId)
    if (paneIndex === -1) return false
    
    const pane = panes.value[paneIndex]
    const oldWidth = pane.width
    const widthDiff = newWidth - oldWidth
    
    // 确保宽度在合理范围内
    if (newWidth < 10 || newWidth > 90) return false
    
    // 调整相邻窗格的宽度
    if (paneIndex < panes.value.length - 1) {
      const nextPane = panes.value[paneIndex + 1]
      const newNextWidth = nextPane.width - widthDiff
      
      if (newNextWidth < 10 || newNextWidth > 90) return false
      
      pane.width = newWidth
      nextPane.width = newNextWidth
    }
    
    return true
  }
  
  /**
   * 设置活动窗格
   */
  function setActivePane(paneId: string) {
    if (panes.value.find(p => p.id === paneId)) {
      activePaneId.value = paneId
    }
  }
  
  /**
   * 获取指定窗格
   */
  function getPane(paneId: string): EditorPane | undefined {
    return panes.value.find(p => p.id === paneId)
  }
  
  /**
   * 重置为单个窗格
   */
  function resetToSinglePane(): boolean {
    const hasUnsaved = panes.value.some(pane => 
      pane.openFiles.some(f => f.hasUnsavedChanges)
    )
    
    if (hasUnsaved) {
      if (!confirm('有文件包含未保存的更改，是否重置？')) {
        return false
      }
    }
    
    panes.value = [
      {
        id: generatePaneId(),
        openFiles: [],
        activeFileIndex: -1,
        width: 100
      }
    ]
    activePaneId.value = panes.value[0].id
    
    return true
  }
  
  return {
    panes,
    activePaneId,
    activePane,
    activePaneIndex,
    splitPane,
    closePane,
    resizePaneWidth,
    setActivePane,
    getPane,
    resetToSinglePane
  }
}
