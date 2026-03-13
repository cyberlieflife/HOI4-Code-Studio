/**
 * useKeyboardShortcuts composable 的单元测试
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useKeyboardShortcuts } from '../../../src/composables/useKeyboardShortcuts'

describe('useKeyboardShortcuts', () => {
  let mockHandlers: {
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
  }

  beforeEach(() => {
    // 重置所有mock
    vi.clearAllMocks()
    
    // 创建mock handlers
    mockHandlers = {
      save: vi.fn(),
      undo: vi.fn(),
      redo: vi.fn(),
      search: vi.fn(),
      copy: vi.fn(() => true),
      cut: vi.fn(() => true),
      paste: vi.fn(() => true),
      nextError: vi.fn(),
      previousError: vi.fn(),
      toggleTheme: vi.fn(),
      toggleIconPanel: vi.fn()
    }
  })

  it('应该正确创建键盘快捷键处理器', () => {
    const { handleKeyDown } = useKeyboardShortcuts(mockHandlers)
    expect(typeof handleKeyDown).toBe('function')
  })

  it('应该处理Ctrl+S保存快捷键', () => {
    const { handleKeyDown } = useKeyboardShortcuts(mockHandlers)
    
    // 创建一个模拟的键盘事件
    const mockEvent = new KeyboardEvent('keydown', {
      ctrlKey: true,
      key: 's'
    })
    
    // 阻止默认行为的mock
    mockEvent.preventDefault = vi.fn()
    
    handleKeyDown(mockEvent)
    
    expect(mockHandlers.save).toHaveBeenCalledTimes(1)
    expect(mockEvent.preventDefault).toHaveBeenCalledTimes(1)
  })

  it('应该处理Ctrl+Z撤销快捷键', () => {
    const { handleKeyDown } = useKeyboardShortcuts(mockHandlers)
    
    // 创建一个模拟的键盘事件
    const mockEvent = new KeyboardEvent('keydown', {
      ctrlKey: true,
      key: 'z'
    })
    
    // 阻止默认行为的mock
    mockEvent.preventDefault = vi.fn()
    
    handleKeyDown(mockEvent)
    
    expect(mockHandlers.undo).toHaveBeenCalledTimes(1)
    // 注意：撤销操作不应该阻止默认行为
    expect(mockEvent.preventDefault).not.toHaveBeenCalled()
  })

  it('应该处理Ctrl+Shift+Z重做快捷键', () => {
    const { handleKeyDown } = useKeyboardShortcuts(mockHandlers)
    
    // 创建一个模拟的键盘事件
    const mockEvent = new KeyboardEvent('keydown', {
      ctrlKey: true,
      shiftKey: true,
      key: 'z'
    })
    
    // 阻止默认行为的mock
    mockEvent.preventDefault = vi.fn()
    
    handleKeyDown(mockEvent)
    
    expect(mockHandlers.redo).toHaveBeenCalledTimes(1)
    // 注意：重做操作不应该阻止默认行为
    expect(mockEvent.preventDefault).not.toHaveBeenCalled()
  })

  it('应该处理Ctrl+Y重做快捷键', () => {
    const { handleKeyDown } = useKeyboardShortcuts(mockHandlers)
    
    // 创建一个模拟的键盘事件
    const mockEvent = new KeyboardEvent('keydown', {
      ctrlKey: true,
      key: 'y'
    })
    
    // 阻止默认行为的mock
    mockEvent.preventDefault = vi.fn()
    
    handleKeyDown(mockEvent)
    
    expect(mockHandlers.redo).toHaveBeenCalledTimes(1)
    // 注意：重做操作不应该阻止默认行为
    expect(mockEvent.preventDefault).not.toHaveBeenCalled()
  })

  it('应该处理Ctrl+F搜索快捷键', () => {
    const { handleKeyDown } = useKeyboardShortcuts(mockHandlers)
    
    // 创建一个模拟的键盘事件
    const mockEvent = new KeyboardEvent('keydown', {
      ctrlKey: true,
      key: 'f'
    })
    
    // 阻止默认行为的mock
    mockEvent.preventDefault = vi.fn()
    
    handleKeyDown(mockEvent)
    
    expect(mockHandlers.search).toHaveBeenCalledTimes(1)
    expect(mockEvent.preventDefault).toHaveBeenCalledTimes(1)
  })

  it('应该处理Ctrl+E下一个错误快捷键', () => {
    const { handleKeyDown } = useKeyboardShortcuts(mockHandlers)
    
    // 创建一个模拟的键盘事件
    const mockEvent = new KeyboardEvent('keydown', {
      ctrlKey: true,
      key: 'e'
    })
    
    // 阻止默认行为的mock
    mockEvent.preventDefault = vi.fn()
    
    handleKeyDown(mockEvent)
    
    expect(mockHandlers.nextError).toHaveBeenCalledTimes(1)
    expect(mockEvent.preventDefault).toHaveBeenCalledTimes(1)
  })

  it('应该处理Ctrl+R上一个错误快捷键', () => {
    const { handleKeyDown } = useKeyboardShortcuts(mockHandlers)
    
    // 创建一个模拟的键盘事件
    const mockEvent = new KeyboardEvent('keydown', {
      ctrlKey: true,
      key: 'r'
    })
    
    // 阻止默认行为的mock
    mockEvent.preventDefault = vi.fn()
    
    handleKeyDown(mockEvent)
    
    expect(mockHandlers.previousError).toHaveBeenCalledTimes(1)
    expect(mockEvent.preventDefault).toHaveBeenCalledTimes(1)
  })

  it('应该处理Ctrl+Shift+T切换主题面板快捷键', () => {
    const { handleKeyDown } = useKeyboardShortcuts(mockHandlers)
    
    // 创建一个模拟的键盘事件
    const mockEvent = new KeyboardEvent('keydown', {
      ctrlKey: true,
      shiftKey: true,
      key: 'T'
    })
    
    // 阻止默认行为的mock
    mockEvent.preventDefault = vi.fn()
    
    handleKeyDown(mockEvent)
    
    expect(mockHandlers.toggleTheme).toHaveBeenCalledTimes(1)
    expect(mockEvent.preventDefault).toHaveBeenCalledTimes(1)
  })

  it('应该处理Ctrl+Shift+Y切换图标面板快捷键', () => {
    const { handleKeyDown } = useKeyboardShortcuts(mockHandlers)
    
    // 创建一个模拟的键盘事件
    const mockEvent = new KeyboardEvent('keydown', {
      ctrlKey: true,
      shiftKey: true,
      key: 'Y'
    })
    
    // 阻止默认行为的mock
    mockEvent.preventDefault = vi.fn()
    
    handleKeyDown(mockEvent)
    
    expect(mockHandlers.toggleIconPanel).toHaveBeenCalledTimes(1)
    expect(mockEvent.preventDefault).toHaveBeenCalledTimes(1)
  })

  it('应该忽略不匹配的快捷键', () => {
    const { handleKeyDown } = useKeyboardShortcuts(mockHandlers)
    
    // 创建一个模拟的键盘事件
    const mockEvent = new KeyboardEvent('keydown', {
      ctrlKey: false,
      key: 'x'
    })
    
    // 阻止默认行为的mock
    mockEvent.preventDefault = vi.fn()
    
    handleKeyDown(mockEvent)
    
    // 验证没有任何handler被调用
    Object.values(mockHandlers).forEach(handler => {
      if (handler) {
        expect(handler).not.toHaveBeenCalled()
      }
    })
    
    // 验证没有阻止默认行为
    expect(mockEvent.preventDefault).not.toHaveBeenCalled()
  })
})
