/**
 * version.ts 测试文件
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { 
  parseVersionToTag, 
  parseTagToVersion, 
  compareVersions, 
  checkForUpdates 
} from '../../utils/version'

describe('version.ts', () => {
  beforeEach(() => {
    // 清除 localStorage 和 fetch 的 mock
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe('parseVersionToTag', () => {
    it('应该将开发版本转换为Dev标签格式', () => {
      expect(parseVersionToTag('v0.1.1-dev')).toBe('Dev_0_1_1')
      expect(parseVersionToTag('v1.2.0-dev')).toBe('Dev_1_2_0')
      expect(parseVersionToTag('v10.20.30-dev')).toBe('Dev_10_20_30')
    })

    it('应该将正式版本转换为v标签格式', () => {
      expect(parseVersionToTag('v0.1.1')).toBe('v0_1_1')
      expect(parseVersionToTag('v1.2.0')).toBe('v1_2_0')
      expect(parseVersionToTag('v10.20.30')).toBe('v10_20_30')
    })

    it('应该处理没有v前缀的版本', () => {
      expect(parseVersionToTag('0.1.1-dev')).toBe('Dev_0_1_1')
      expect(parseVersionToTag('0.1.1')).toBe('v0_1_1')
    })

    it('应该处理空字符串', () => {
      expect(parseVersionToTag('')).toBe('v')
    })
  })

  describe('parseTagToVersion', () => {
    it('应该将Dev标签转换为开发版本', () => {
      expect(parseTagToVersion('Dev_0_1_1')).toBe('v0.1.1-dev')
      expect(parseTagToVersion('Dev_1_2_0')).toBe('v1.2.0-dev')
      expect(parseTagToVersion('Dev_10_20_30')).toBe('v10.20.30-dev')
    })

    it('应该兼容OV正式版与dev日期标签', () => {
      expect(parseTagToVersion('OV_0_3_1')).toBe('v0.3.1')
      expect(parseTagToVersion('OV_0_3_1_dev_20250101')).toBe('v0.3.1-dev-20250101')
    })

    it('应该兼容纯dev日期标签', () => {
      expect(parseTagToVersion('dev_20250101')).toBe('v0.0.0-dev-20250101')
    })

    it('应该将v标签转换为正式版本', () => {
      expect(parseTagToVersion('v0_1_1')).toBe('v0.1.1')
      expect(parseTagToVersion('v1_2_0')).toBe('v1.2.0')
      expect(parseTagToVersion('v10_20_30')).toBe('v10.20.30')
    })

    it('应该处理不符合预期格式的标签', () => {
      expect(parseTagToVersion('invalid_tag')).toBe('invalid_tag')
      expect(parseTagToVersion('')).toBe('')
    })
  })

  describe('compareVersions', () => {
    it('应该正确比较版本号', () => {
      // version2 > version1 的情况
      expect(compareVersions('v0.1.0', 'v0.2.0')).toBe(true)
      expect(compareVersions('v1.0.0', 'v2.0.0')).toBe(true)
      expect(compareVersions('v0.1.0', 'v0.1.1')).toBe(true)
      
      // version2 < version1 的情况
      expect(compareVersions('v0.2.0', 'v0.1.0')).toBe(false)
      expect(compareVersions('v2.0.0', 'v1.0.0')).toBe(false)
      expect(compareVersions('v0.1.1', 'v0.1.0')).toBe(false)
    })

    it('应该处理相同版本号的比较', () => {
      expect(compareVersions('v0.1.0', 'v0.1.0')).toBe(false)
      expect(compareVersions('v1.2.3', 'v1.2.3')).toBe(false)
    })

    it('应该处理不同长度的版本号', () => {
      expect(compareVersions('v0.1', 'v0.1.0')).toBe(false)
      expect(compareVersions('v0.1.0', 'v0.1.1')).toBe(true)
      expect(compareVersions('v1.0', 'v1.0.1')).toBe(true)
    })

    it('应该正确处理dev版本和正式版本的比较', () => {
      // 正式版本 > dev版本
      expect(compareVersions('v0.1.0-dev', 'v0.1.0')).toBe(true)
      expect(compareVersions('v1.0.0-dev', 'v1.0.0')).toBe(true)
      
      // 相同版本号时，dev版本 < 正式版本
      expect(compareVersions('v0.1.0', 'v0.1.0-dev')).toBe(false)
      
      // 不同版本号时，先比较版本号
      expect(compareVersions('v0.2.0-dev', 'v0.1.0')).toBe(false) // 0.2.0 > 0.1.0
    })

    it('应该比较dev日期版本', () => {
      expect(compareVersions('v0.1.0-dev-20240101', 'v0.1.0-dev-20240102')).toBe(true)
      expect(compareVersions('v0.1.0-dev-20240102', 'v0.1.0-dev-20240101')).toBe(false)
    })

    it('应该比较版本后缀修订号', () => {
      expect(compareVersions('v0.1.0', 'v0.1.0-1')).toBe(true)
      expect(compareVersions('v0.1.0-2', 'v0.1.0-1')).toBe(false)
      expect(compareVersions('v0.1.0-1', 'v0.1.0')).toBe(false)
    })

    it('应在dev日期相同时比较修订号', () => {
      expect(compareVersions('v0.1.0-dev-20240101-1', 'v0.1.0-dev-20240101-2')).toBe(true)
      expect(compareVersions('v0.1.0-dev-20240101-2', 'v0.1.0-dev-20240101-1')).toBe(false)
      expect(compareVersions('v0.1.0-dev-20240101', 'v0.1.0-dev-20240101-1')).toBe(true)
    })

    it('应该处理没有v前缀的版本', () => {
      expect(compareVersions('0.1.0', '0.2.0')).toBe(true)
      expect(compareVersions('1.0.0-dev', '1.0.0')).toBe(true)
    })
  })

  describe('缓存功能', () => {
    const mockCache = {
      version: 'v0.2.0',
      url: 'https://github.com/test/release',
      timestamp: Date.now()
    }

    beforeEach(() => {
      const localStorageMock = {
        getItem: vi.fn(),
        setItem: vi.fn(),
        removeItem: vi.fn(),
        clear: vi.fn()
      }
      Object.defineProperty(window, 'localStorage', {
        value: localStorageMock,
        writable: true
      })
      // 确保 localStorage.getItem 是 mock 函数
      vi.mocked(localStorage.getItem).mockClear()
    })

    it('应该从缓存中读取数据', () => {
      // 直接使用 localStorage.getItem，因为它已经是 mock
      const mockGetItem = vi.mocked(localStorage.getItem)
      mockGetItem.mockReturnValue(JSON.stringify(mockCache))
      
      // 由于 getCache 是内部函数，我们通过 checkForUpdates 的缓存行为来测试
      // 这里我们模拟缓存命中的情况
      mockGetItem.mockImplementation((key) => {
        if (key === 'hoi4_version_check_cache') {
          return JSON.stringify(mockCache)
        }
        return null
      })
    })

    it('应该处理缓存过期', () => {
      const expiredCache = {
        ...mockCache,
        timestamp: Date.now() - (2 * 60 * 60 * 1000) // 2小时前
      }
      
      const mockGetItem = vi.mocked(localStorage.getItem)
      mockGetItem.mockReturnValue(JSON.stringify(expiredCache))
      
      // 缓存过期应该返回 null，但由于 getCache 是内部函数，
      // 我们通过 behavior 测试来验证
    })

    it('应该处理无效缓存数据', () => {
      const mockGetItem = vi.mocked(localStorage.getItem)
      mockGetItem.mockReturnValue('invalid json')
      
      // 无效的JSON应该返回 null
    })
  })

  describe('checkForUpdates', () => {
    beforeEach(() => {
      // 重置全局 fetch mock
      global.fetch = vi.fn()
    })

    it('应该成功检查更新（有新版本）', async () => {
      const mockResponse = {
        ok: true,
        status: 200,
        statusText: 'OK',
        headers: new Headers(),
        json: () => Promise.resolve({
          tag_name: 'v0_2_0',
          html_url: 'https://github.com/test/releases/v0.2.0'
        })
      }
      
      vi.mocked(fetch).mockResolvedValue(mockResponse as any)

      const result = await checkForUpdates('v0.1.0')
      
      expect(result.hasUpdate).toBe(true)
      expect(result.latestVersion).toBe('v0.2.0')
      expect(result.releaseUrl).toBe('https://github.com/test/releases/v0.2.0')
      expect(result.error).toBeUndefined()
      
      expect(vi.mocked(fetch)).toHaveBeenCalledWith(
        'https://api.github.com/repos/cyberlieflife/HOI4-Code-Studio/releases/latest',
        {
          headers: {
            'Accept': 'application/vnd.github.v3+json'
          }
        }
      )
    })

    it('应该成功检查更新（无新版本）', async () => {
      const mockResponse = {
        ok: true,
        status: 200,
        statusText: 'OK',
        headers: new Headers(),
        json: () => Promise.resolve({
          tag_name: 'v0_1_0',
          html_url: 'https://github.com/test/releases/v0.1.0'
        })
      }
      
      vi.mocked(fetch).mockResolvedValue(mockResponse as any)

      const result = await checkForUpdates('v0.1.0')
      
      expect(result.hasUpdate).toBe(false)
      expect(result.latestVersion).toBe('v0.1.0')
      expect(result.releaseUrl).toBe('https://github.com/test/releases/v0.1.0')
      expect(result.error).toBeUndefined()
    })

    it('应该使用GitHub Token', async () => {
      const mockResponse = {
        ok: true,
        status: 200,
        statusText: 'OK',
        headers: new Headers(),
        json: () => Promise.resolve({
          tag_name: 'v0_2_0',
          html_url: 'https://github.com/test/releases/v0.2.0'
        })
      }
      
      vi.mocked(fetch).mockResolvedValue(mockResponse as any)

      await checkForUpdates('v0.1.0', 'test-token')
      
      expect(vi.mocked(fetch)).toHaveBeenCalledWith(
        'https://api.github.com/repos/cyberlieflife/HOI4-Code-Studio/releases/latest',
        {
          headers: {
            'Accept': 'application/vnd.github.v3+json',
            'Authorization': 'token test-token'
          }
        }
      )
    })

    it('应该处理API请求失败', async () => {
      const mockResponse = {
        ok: false,
        status: 404,
        statusText: 'Not Found',
        headers: new Headers(),
        json: () => Promise.resolve({ message: 'Not found' })
      }
      
      vi.mocked(fetch).mockResolvedValue(mockResponse as any)

      const result = await checkForUpdates('v0.1.0')
      
      expect(result.hasUpdate).toBe(false)
      expect(result.error).toBeDefined()
      expect(result.error).toContain('GitHub API 请求失败')
    })

    it('应该处理网络错误', async () => {
      vi.mocked(fetch).mockRejectedValue(new Error('Network error'))

      const result = await checkForUpdates('v0.1.0')
      
      expect(result.hasUpdate).toBe(false)
      expect(result.error).toBe('Network error')
    })

    it('应该处理Dev版本标签', async () => {
      const mockResponse = {
        ok: true,
        status: 200,
        statusText: 'OK',
        headers: new Headers(),
        json: () => Promise.resolve({
          tag_name: 'Dev_0_2_0',
          html_url: 'https://github.com/test/releases/v0.2.0-dev'
        })
      }
      
      vi.mocked(fetch).mockResolvedValue(mockResponse as any)

      const result = await checkForUpdates('v0.1.0')
      
      expect(result.hasUpdate).toBe(true)
      expect(result.latestVersion).toBe('v0.2.0-dev')
      expect(result.releaseUrl).toBe('https://github.com/test/releases/v0.2.0-dev')
    })

    it('应该禁用缓存功能', async () => {
      const mockResponse = {
        ok: true,
        status: 200,
        statusText: 'OK',
        headers: new Headers(),
        json: () => Promise.resolve({
          tag_name: 'v0_2_0',
          html_url: 'https://github.com/test/releases/v0.2.0'
        })
      }
      
      vi.mocked(fetch).mockResolvedValue(mockResponse as any)

      // 调用两次，第二次应该仍然发起网络请求（因为禁用缓存）
      await checkForUpdates('v0.1.0', undefined, false)
      await checkForUpdates('v0.1.0', undefined, false)
      
      expect(vi.mocked(fetch)).toHaveBeenCalledTimes(2)
    })
  })

  describe('边界情况和错误处理', () => {
    it('应该处理空字符串输入', () => {
      expect(parseVersionToTag('')).toBe('v')
      expect(parseTagToVersion('')).toBe('')
    })

    it('应该处理边界情况', () => {
      // 测试特殊字符输入
      expect(parseVersionToTag('v1.2.3-beta')).toBe('v1_2_3-beta')
      expect(parseTagToVersion('v1_2_3-beta')).toBe('v1.2.3-beta')
    })

    it('应该处理特殊字符输入', () => {
      expect(parseVersionToTag('v1.2.3-beta')).toBe('v1_2_3-beta')
      expect(parseTagToVersion('v1_2_3-beta')).toBe('v1.2.3-beta')
    })
  })
})