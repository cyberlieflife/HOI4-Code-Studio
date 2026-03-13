import { describe, expect, it, vi } from 'vitest'
import { useKeyboardShortcuts } from '../../../src/composables/useKeyboardShortcuts'

describe('useKeyboardShortcuts clipboard', () => {
  it('应处理 Ctrl+C 复制快捷键', () => {
    const copy = vi.fn(() => true)
    const { handleKeyDown } = useKeyboardShortcuts({ copy })
    const event = new KeyboardEvent('keydown', { ctrlKey: true, key: 'c' })
    event.preventDefault = vi.fn()

    handleKeyDown(event)

    expect(copy).toHaveBeenCalledTimes(1)
    expect(event.preventDefault).toHaveBeenCalledTimes(1)
  })

  it('应处理 Ctrl+X 剪切快捷键', () => {
    const cut = vi.fn(() => true)
    const { handleKeyDown } = useKeyboardShortcuts({ cut })
    const event = new KeyboardEvent('keydown', { ctrlKey: true, key: 'x' })
    event.preventDefault = vi.fn()

    handleKeyDown(event)

    expect(cut).toHaveBeenCalledTimes(1)
    expect(event.preventDefault).toHaveBeenCalledTimes(1)
  })

  it('应处理 Ctrl+V 粘贴快捷键', () => {
    const paste = vi.fn(() => true)
    const { handleKeyDown } = useKeyboardShortcuts({ paste })
    const event = new KeyboardEvent('keydown', { ctrlKey: true, key: 'v' })
    event.preventDefault = vi.fn()

    handleKeyDown(event)

    expect(paste).toHaveBeenCalledTimes(1)
    expect(event.preventDefault).toHaveBeenCalledTimes(1)
  })

  it('当处理器返回 false 时不阻止默认行为', () => {
    const copy = vi.fn(() => false)
    const { handleKeyDown } = useKeyboardShortcuts({ copy })
    const event = new KeyboardEvent('keydown', { ctrlKey: true, key: 'c' })
    event.preventDefault = vi.fn()

    handleKeyDown(event)

    expect(copy).toHaveBeenCalledTimes(1)
    expect(event.preventDefault).not.toHaveBeenCalled()
  })
})
