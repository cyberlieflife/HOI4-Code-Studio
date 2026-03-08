/**
 * 编辑器状态管理测试
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useEditorState } from '../../../src/composables/useEditorState'

describe('useEditorState', () => {
  let editorState: ReturnType<typeof useEditorState>

  beforeEach(() => {
    vi.clearAllMocks()
    editorState = useEditorState()
  })

  it('应该正确初始化编辑器状态', () => {
    expect(editorState.fileContent.value).toBe('')
    expect(editorState.hasUnsavedChanges.value).toBe(false)
    expect(editorState.currentLine.value).toBe(1)
    expect(editorState.currentColumn.value).toBe(1)
  })

  it('应该能够处理内容变化', () => {
    const content = 'Hello, World!'
    editorState.onContentChange(content)

    expect(editorState.fileContent.value).toBe(content)
    expect(editorState.hasUnsavedChanges.value).toBe(true)
  })

  it('应该能够重置未保存标记', () => {
    // 先修改内容
    editorState.onContentChange('test content')
    expect(editorState.hasUnsavedChanges.value).toBe(true)

    // 重置未保存标记
    editorState.resetUnsavedChanges()
    expect(editorState.hasUnsavedChanges.value).toBe(false)
  })

  it('应该能够更新光标位置 - 单行文本', () => {
    const mockTextarea = {
      value: 'Hello, World!',
      selectionStart: 5
    } as HTMLTextAreaElement

    editorState.updateCursorPosition(mockTextarea)

    expect(editorState.currentLine.value).toBe(1)
    expect(editorState.currentColumn.value).toBe(6) // 位置从1开始
  })

  it('应该能够更新光标位置 - 多行文本', () => {
    const mockTextarea = {
      value: 'Line 1\nLine 2\nLine 3',
      selectionStart: 10 // 在第二行
    } as HTMLTextAreaElement

    editorState.updateCursorPosition(mockTextarea)

    expect(editorState.currentLine.value).toBe(2)
    expect(editorState.currentColumn.value).toBe(4) // 'Line 1\n' 长度是7，第二行从位置7开始，光标在位置10，所以列号是4
  })

  it('应该能够处理空文本的光标位置', () => {
    const mockTextarea = {
      value: '',
      selectionStart: 0
    } as HTMLTextAreaElement

    editorState.updateCursorPosition(mockTextarea)

    expect(editorState.currentLine.value).toBe(1)
    expect(editorState.currentColumn.value).toBe(1)
  })

  it('应该能够处理光标在文本末尾的情况', () => {
    const mockTextarea = {
      value: 'Hello, World!',
      selectionStart: 13 // 在文本末尾
    } as HTMLTextAreaElement

    editorState.updateCursorPosition(mockTextarea)

    expect(editorState.currentLine.value).toBe(1)
    expect(editorState.currentColumn.value).toBe(14) // 位置从1开始
  })

  it('应该能够处理多行文本末尾的光标位置', () => {
    const mockTextarea = {
      value: 'Line 1\nLine 2',
      selectionStart: 12 // 在第二行末尾
    } as HTMLTextAreaElement

    editorState.updateCursorPosition(mockTextarea)

    expect(editorState.currentLine.value).toBe(2)
    expect(editorState.currentColumn.value).toBe(6) // 'Line 1\n' 长度是7，第二行从位置7开始，光标在位置12，所以列号是6
  })

  it('应该能够处理只有换行符的文本', () => {
    const mockTextarea = {
      value: '\n\n',
      selectionStart: 2 // 在第二个换行符后
    } as HTMLTextAreaElement

    editorState.updateCursorPosition(mockTextarea)

    expect(editorState.currentLine.value).toBe(3)
    expect(editorState.currentColumn.value).toBe(1)
  })

  it('应该能够处理混合内容的状态变化', () => {
    // 设置只读状态
    // 修改内容
    editorState.onContentChange('New content')
    expect(editorState.fileContent.value).toBe('New content')
    expect(editorState.hasUnsavedChanges.value).toBe(true)

    // 重置未保存标记
    editorState.resetUnsavedChanges()
    expect(editorState.hasUnsavedChanges.value).toBe(false)

  })

  it('应该能够处理连续的内容变化', () => {
    editorState.onContentChange('First change')
    expect(editorState.hasUnsavedChanges.value).toBe(true)

    editorState.onContentChange('Second change')
    expect(editorState.hasUnsavedChanges.value).toBe(true)
    expect(editorState.fileContent.value).toBe('Second change')

    editorState.resetUnsavedChanges()
    expect(editorState.hasUnsavedChanges.value).toBe(false)
  })

  it('应该能够处理边界情况 - 空内容', () => {
    editorState.onContentChange('')
    expect(editorState.fileContent.value).toBe('')
    expect(editorState.hasUnsavedChanges.value).toBe(true)

    editorState.resetUnsavedChanges()
    expect(editorState.hasUnsavedChanges.value).toBe(false)
  })

  it('应该能够处理边界情况 - 非常长的内容', () => {
    const longContent = 'A'.repeat(10000)
    editorState.onContentChange(longContent)
    
    expect(editorState.fileContent.value).toBe(longContent)
    expect(editorState.hasUnsavedChanges.value).toBe(true)
  })
})
