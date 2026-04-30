/**
 * 编辑器窗格分组管理测试
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useEditorGroups } from '../../../src/composables/useEditorGroups'

// Mock window.confirm
const mockConfirm = vi.fn()
Object.defineProperty(window, 'confirm', {
  value: mockConfirm,
  writable: true
})

// 模拟 notification
vi.mock('../../../src/utils/notification', () => ({
  toast: {
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
    success: vi.fn()
  }
}))

import { toast } from '../../../src/utils/notification'

describe('useEditorGroups', () => {
  let editorGroups: ReturnType<typeof useEditorGroups>

  beforeEach(() => {
    vi.clearAllMocks()
    editorGroups = useEditorGroups()
    mockConfirm.mockReturnValue(true) // 默认确认操作
  })

  it('应该正确初始化窗格状态', () => {
    expect(editorGroups.panes.value).toHaveLength(1)
    expect(editorGroups.activePaneId.value).toBeDefined()
    expect(editorGroups.activePane.value).toBeDefined()
    expect(editorGroups.activePaneIndex.value).toBe(0)
  })

  it('应该能够生成唯一的窗格ID', () => {
    // 通过分割窗格来测试窗格ID生成
    const initialPaneId = editorGroups.activePaneId.value
    const result = editorGroups.splitPane(initialPaneId)
    
    expect(result).toBe(true)
    expect(editorGroups.panes.value).toHaveLength(2)
    
    const newPaneId = editorGroups.activePaneId.value
    expect(newPaneId).toBeDefined()
    expect(newPaneId).not.toBe(initialPaneId)
    expect(newPaneId).toMatch(/^pane-\d+-[a-z0-9]+$/)
  })

  it('应该能够分割窗格', () => {
    const initialPaneId = editorGroups.activePaneId.value
    const initialPaneCount = editorGroups.panes.value.length
    
    const result = editorGroups.splitPane(initialPaneId)
    
    expect(result).toBe(true)
    expect(editorGroups.panes.value).toHaveLength(initialPaneCount + 1)
    expect(editorGroups.activePaneId.value).not.toBe(initialPaneId)
  })

  it('应该限制最多分割为3个窗格', () => {
    // 分割到最大数量
    const initialPaneId = editorGroups.activePaneId.value
    editorGroups.splitPane(initialPaneId)
    editorGroups.splitPane(initialPaneId)
    
    // 尝试分割第四个窗格
    vi.mocked(toast.warning).mockClear()
    const result = editorGroups.splitPane(initialPaneId)
    
    expect(result).toBe(false)
    expect(toast.warning).toHaveBeenCalledWith('最多只能分割为3个窗格')
    expect(editorGroups.panes.value).toHaveLength(3)
  })

  it('应该能够关闭窗格', () => {
    // 先分割一个窗格
    const initialPaneId = editorGroups.activePaneId.value
    editorGroups.splitPane(initialPaneId)
    
    const paneCountBeforeClose = editorGroups.panes.value.length
    const paneToClose = editorGroups.panes.value[1]
    
    const result = editorGroups.closePane(paneToClose.id)
    
    expect(result).toBe(true)
    expect(editorGroups.panes.value).toHaveLength(paneCountBeforeClose - 1)
  })

  it('应该阻止关闭最后一个窗格', () => {
    const initialPaneId = editorGroups.activePaneId.value
    const result = editorGroups.closePane(initialPaneId)
    
    expect(result).toBe(false)
    expect(editorGroups.panes.value).toHaveLength(1)
  })

  it('应该询问用户确认关闭有未保存更改的窗格', () => {
    // 先分割一个窗格
    const initialPaneId = editorGroups.activePaneId.value
    editorGroups.splitPane(initialPaneId)
    
    // 在第二个窗格中添加未保存的文件
    const secondPane = editorGroups.panes.value[1]
    secondPane.openFiles = [
      {
        node: {
          path: '/test/file.txt',
          name: 'file.txt',
          isDirectory: false,
          children: []
        },
        content: '',
        hasUnsavedChanges: true,
        cursorLine: 1,
        cursorColumn: 1
      }
    ]
    secondPane.activeFileIndex = 0
    
    // 设置用户拒绝关闭
    mockConfirm.mockReturnValue(false)
    const result = editorGroups.closePane(secondPane.id)
    
    expect(result).toBe(false)
    expect(mockConfirm).toHaveBeenCalledWith('该窗格中有文件包含未保存的更改，是否关闭？')
    expect(editorGroups.panes.value).toHaveLength(2)
  })

  it('应该能够调整窗格宽度', () => {
    // 先分割一个窗格
    const initialPaneId = editorGroups.activePaneId.value
    editorGroups.splitPane(initialPaneId)
    
    const paneId = editorGroups.panes.value[0].id
    const newWidth = 60
    
    const result = editorGroups.resizePaneWidth(paneId, newWidth)
    
    expect(result).toBe(true)
    expect(editorGroups.panes.value[0].width).toBe(newWidth)
    expect(editorGroups.panes.value[1].width).toBe(40) // 剩余宽度
  })

  it('应该阻止调整宽度到不合理范围', () => {
    const paneId = editorGroups.activePaneId.value
    
    // 尝试设置过小的宽度
    const result1 = editorGroups.resizePaneWidth(paneId, 5)
    expect(result1).toBe(false)
    
    // 尝试设置过大的宽度
    const result2 = editorGroups.resizePaneWidth(paneId, 95)
    expect(result2).toBe(false)
  })

  it('应该能够设置活动窗格', () => {
    // 先分割一个窗格
    const initialPaneId = editorGroups.activePaneId.value
    editorGroups.splitPane(initialPaneId)
    
    const newPaneId = editorGroups.panes.value[1].id
    editorGroups.setActivePane(newPaneId)
    
    expect(editorGroups.activePaneId.value).toBe(newPaneId)
    expect(editorGroups.activePane.value.id).toBe(newPaneId)
  })

  it('应该能够获取指定窗格', () => {
    const paneId = editorGroups.activePaneId.value
    const pane = editorGroups.getPane(paneId)
    
    expect(pane).toBeDefined()
    expect(pane?.id).toBe(paneId)
  })

  it('应该能够重置为单个窗格', () => {
    // 先分割几个窗格
    const initialPaneId = editorGroups.activePaneId.value
    editorGroups.splitPane(initialPaneId)
    editorGroups.splitPane(initialPaneId)
    
    expect(editorGroups.panes.value).toHaveLength(3)
    
    const result = editorGroups.resetToSinglePane()
    
    expect(result).toBe(true)
    expect(editorGroups.panes.value).toHaveLength(1)
    expect(editorGroups.activePaneId.value).toBe(editorGroups.panes.value[0].id)
  })

  it('应该询问用户确认重置有未保存更改的窗格', () => {
    // 先分割一个窗格
    const initialPaneId = editorGroups.activePaneId.value
    editorGroups.splitPane(initialPaneId)
    
    // 在第一个窗格中添加未保存的文件
    const firstPane = editorGroups.panes.value[0]
    firstPane.openFiles = [
      {
        node: {
          path: '/test/file.txt',
          name: 'file.txt',
          isDirectory: false,
          children: []
        },
        content: '',
        hasUnsavedChanges: true,
        cursorLine: 1,
        cursorColumn: 1
      }
    ]
    firstPane.activeFileIndex = 0
    
    // 设置用户拒绝重置
    mockConfirm.mockReturnValue(false)
    const result = editorGroups.resetToSinglePane()
    
    expect(result).toBe(false)
    expect(mockConfirm).toHaveBeenCalledWith('有文件包含未保存的更改，是否重置？')
    expect(editorGroups.panes.value).toHaveLength(2)
  })

  it('应该正确处理窗格宽度分配', () => {
    // 先分割一个窗格
    const initialPaneId = editorGroups.activePaneId.value
    editorGroups.splitPane(initialPaneId)
    
    // 设置窗格宽度
    editorGroups.panes.value[0].width = 40
    editorGroups.panes.value[1].width = 60
    
    // 关闭第一个窗格
    const result = editorGroups.closePane(editorGroups.panes.value[0].id)
    
    expect(result).toBe(true)
    expect(editorGroups.panes.value).toHaveLength(1)
    expect(editorGroups.panes.value[0].width).toBe(100) // 剩余窗格应该获得全部宽度
  })

  it('应该正确处理分割窗格时的文件移动', () => {
    const initialPaneId = editorGroups.activePaneId.value
    
    // 在源窗格中添加文件
    const sourcePane = editorGroups.getPane(initialPaneId)
    if (sourcePane) {
      sourcePane.openFiles = [
        {
          node: {
            path: '/test/file1.txt',
            name: 'file1.txt',
            isDirectory: false,
            children: []
          },
          content: '',
          hasUnsavedChanges: false,
          cursorLine: 1,
          cursorColumn: 1
        },
        {
          node: {
            path: '/test/file2.txt',
            name: 'file2.txt',
            isDirectory: false,
            children: []
          },
          content: '',
          hasUnsavedChanges: true,
          cursorLine: 1,
          cursorColumn: 1
        }
      ]
      sourcePane.activeFileIndex = 1
    }
    
    // 分割窗格并移动第二个文件
    const result = editorGroups.splitPane(initialPaneId, 1)
    
    expect(result).toBe(true)
    expect(editorGroups.panes.value).toHaveLength(2)
    
    // 检查文件移动
    const sourcePaneAfterSplit = editorGroups.getPane(initialPaneId)
    const newPane = editorGroups.panes.value[1]
    
    // 源窗格应该保留所有文件（实际实现是复制而不是移动）
    expect(sourcePaneAfterSplit?.openFiles).toHaveLength(2)
    expect(sourcePaneAfterSplit?.openFiles[0].node.name).toBe('file1.txt')
    expect(sourcePaneAfterSplit?.openFiles[1].node.name).toBe('file2.txt')
    
    // 新窗格应该包含复制的文件
    expect(newPane.openFiles).toHaveLength(1)
    expect(newPane.openFiles[0].node.name).toBe('file2.txt')
    expect(newPane.activeFileIndex).toBe(0)
  })

  it('应该正确处理活动窗格切换', () => {
    // 先分割一个窗格
    const initialPaneId = editorGroups.activePaneId.value
    editorGroups.splitPane(initialPaneId)
    
    const pane1 = editorGroups.panes.value[0]
    const pane2 = editorGroups.panes.value[1]
    
    // 设置活动窗格为第一个
    editorGroups.setActivePane(pane1.id)
    
    // 关闭活动窗格
    const result = editorGroups.closePane(pane1.id)
    
    expect(result).toBe(true)
    expect(editorGroups.activePaneId.value).toBe(pane2.id)
  })
})