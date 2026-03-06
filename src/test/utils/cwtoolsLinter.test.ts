/**
 * cwtools Linter 集成测试
 */

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { createCWToolsLinter } from '@/utils/cwtoolsLinter'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

describe('cwtoolsLinter', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('应该创建 linter 扩展数组', () => {
    const linter = createCWToolsLinter({
      getFilePath: () => '/test/file.txt',
      getVersion: () => 1,
      delay: 100
    })

    expect(Array.isArray(linter)).toBe(true)
    expect(linter.length).toBeGreaterThan(0)
  })

  it('应该接受所有必需的配置选项', () => {
    const options = {
      getFilePath: () => '/test/file.txt',
      getVersion: () => 1,
      getProjectRoot: () => '/test/project',
      getGameRoot: () => '/test/game',
      delay: 500,
      enableErrorLens: true,
      enableLineDecoration: true
    }

    const linter = createCWToolsLinter(options)
    expect(linter).toBeDefined()
  })

  it('应该支持禁用 Error Lens', () => {
    const linter = createCWToolsLinter({
      getFilePath: () => '/test/file.txt',
      getVersion: () => 1,
      enableErrorLens: false
    })

    expect(linter).toBeDefined()
  })

  it('应该支持禁用行级装饰', () => {
    const linter = createCWToolsLinter({
      getFilePath: () => '/test/file.txt',
      getVersion: () => 1,
      enableLineDecoration: false
    })

    expect(linter).toBeDefined()
  })
})
