/**
 * 地图引擎测试
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useMapEngine } from '../../../src/composables/useMapEngine'

// Mock Tauri API
vi.mock('../../../src/api/tauri', () => ({
  initializeMapContext: vi.fn(),
  initializeMapContextWithFallback: vi.fn(),
  getMapTileDirect: vi.fn(),
  getMapPreview: vi.fn(),
  getProvinceAtPoint: vi.fn(),
  getProvinceOutline: vi.fn(),
  getStateOutline: vi.fn(),
  loadDefaultMap: vi.fn()
}))

// Mock mapPerformance
vi.mock('../../../src/utils/mapPerformance', () => ({
  logMapEvent: vi.fn(),
  measureMapAsync: vi.fn((fn) => fn())
}))

describe('useMapEngine', () => {
  let mapEngine: ReturnType<typeof useMapEngine>

  beforeEach(() => {
    vi.clearAllMocks()
    mapEngine = useMapEngine()
  })

  it('应该正确初始化', () => {
    expect(mapEngine.definitions.value).toEqual([])
    expect(mapEngine.defaultMap.value).toBeNull()
    expect(mapEngine.mapData.value).toBeNull()
    expect(mapEngine.states.value).toEqual([])
    expect(mapEngine.countryColors.value).toEqual({})
    expect(mapEngine.isLoading.value).toBe(false)
    expect(mapEngine.error.value).toBeNull()
  })

  it('应该能够初始化地图', async () => {
    const { initializeMapContextWithFallback } = await import('../../../src/api/tauri')
    
    // Mock返回值
    vi.mocked(initializeMapContextWithFallback).mockResolvedValue({
      metadata: { width: 5632, height: 2048, province_count: 1000 },
      definitions: [],
      states: [],
      defaultMap: {
        definitions: 'definition.csv',
        provinces: 'provinces.bmp',
        adjacencies: 'adjacencies.csv',
        continent: 'continent.txt',
        rivers: 'rivers.bmp'
      }
    })

    await mapEngine.initMap('/test/project', '/test/game', [], 'fallback')

    expect(mapEngine.mapData.value).toEqual({
      width: 5632,
      height: 2048,
      province_count: 1000
    })
    expect(mapEngine.defaultMap.value).not.toBeNull()
    expect(mapEngine.isLoading.value).toBe(false)
    expect(mapEngine.error.value).toBeNull()
  })

  it('应该处理初始化错误', async () => {
    const { initializeMapContextWithFallback } = await import('../../../src/api/tauri')
    
    // Mock错误
    vi.mocked(initializeMapContextWithFallback).mockRejectedValue(new Error('初始化失败'))

    await mapEngine.initMap('/test/project', '/test/game', [], 'fallback')

    expect(mapEngine.error.value).toBe('初始化失败')
    expect(mapEngine.isLoading.value).toBe(false)
  })

  it('应该能够获取省份ID', async () => {
    const { getProvinceAtPoint } = await import('../../../src/api/tauri')
    
    vi.mocked(getProvinceAtPoint).mockResolvedValue(123)

    const result = await mapEngine.getProvinceId(100, 200)

    expect(result).toBe(123)
    expect(getProvinceAtPoint).toHaveBeenCalledWith(100, 200)
  })

  it('应该能够获取省份轮廓', async () => {
    const { getProvinceOutline } = await import('../../../src/api/tauri')
    
    const mockOutline = new Uint32Array([1, 2, 3, 4])
    vi.mocked(getProvinceOutline).mockResolvedValue(mockOutline)

    const result = await mapEngine.getOutline(123)

    expect(result).toEqual(mockOutline)
    expect(getProvinceOutline).toHaveBeenCalledWith(123)
  })

  it('应该能够获取地区轮廓', async () => {
    const { getStateOutline } = await import('../../../src/api/tauri')
    
    const mockOutline = new Uint32Array([5, 6, 7, 8])
    vi.mocked(getStateOutline).mockResolvedValue(mockOutline)

    const result = await mapEngine.getStateOutline(456)

    expect(result).toEqual(mockOutline)
    expect(getStateOutline).toHaveBeenCalledWith(456)
  })

  it('应该能够渲染地图瓦片', async () => {
    const { getMapTileDirect } = await import('../../../src/api/tauri')
    
    const mockTile = new Uint8Array([1, 2, 3])
    vi.mocked(getMapTileDirect).mockResolvedValue(mockTile)

    const result = await mapEngine.renderTile(0, 0, 1, 'provinces')

    expect(result).toEqual(mockTile)
    expect(getMapTileDirect).toHaveBeenCalledWith(0, 0, 1, 'provinces')
  })

  it('应该能够获取地图预览', async () => {
    const { getMapPreview } = await import('../../../src/api/tauri')
    
    const mockPreview = new Uint8Array([4, 5, 6])
    vi.mocked(getMapPreview).mockResolvedValue(mockPreview)

    const result = await mapEngine.getPreview(800, 600, 'provinces')

    expect(result).toEqual(mockPreview)
    expect(getMapPreview).toHaveBeenCalledWith(800, 600, 'provinces')
  })

  it('应该处理省份ID查询错误', async () => {
    const { getProvinceAtPoint } = await import('../../../src/api/tauri')
    
    vi.mocked(getProvinceAtPoint).mockRejectedValue(new Error('查询失败'))

    const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {})

    const result = await mapEngine.getProvinceId(100, 200)

    expect(result).toBeNull()
    expect(alertSpy).toHaveBeenCalledWith('获取省份ID失败: 查询失败')
  })

  it('应该处理省份轮廓查询错误', async () => {
    const { getProvinceOutline } = await import('../../../src/api/tauri')
    
    vi.mocked(getProvinceOutline).mockRejectedValue(new Error('轮廓计算失败'))

    const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {})

    const result = await mapEngine.getOutline(123)

    expect(result).toEqual(new Uint32Array())
    expect(alertSpy).toHaveBeenCalledWith('获取省份轮廓失败: 轮廓计算失败')
  })

  it('应该处理地区轮廓查询错误', async () => {
    const { getStateOutline } = await import('../../../src/api/tauri')
    
    vi.mocked(getStateOutline).mockRejectedValue(new Error('地区轮廓计算失败'))

    const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {})

    const result = await mapEngine.getStateOutline(456)

    expect(result).toEqual(new Uint32Array())
    expect(alertSpy).toHaveBeenCalledWith('获取地区轮廓失败: 地区轮廓计算失败')
  })

  it('应该在项目路径为空时返回错误', async () => {
    await mapEngine.initMap('', '/test/game', [], 'fallback')

    expect(mapEngine.error.value).toBe('未指定项目路径')
  })

  it('应该支持project-only模式', async () => {
    const { initializeMapContext, loadDefaultMap } = await import('../../../src/api/tauri')
    
    vi.mocked(loadDefaultMap).mockResolvedValue({
      success: true,
      message: 'success',
      data: {
        definitions: 'definition.csv',
        provinces: 'provinces.bmp',
        adjacencies: 'adjacencies.csv',
        continent: 'continent.txt',
        rivers: 'rivers.bmp'
      }
    })

    vi.mocked(initializeMapContext).mockResolvedValue({
      metadata: { width: 5632, height: 2048, province_count: 1000 },
      definitions: [],
      states: []
    })

    await mapEngine.initMap('/test/project', '/test/game', [], 'project-only')

    expect(mapEngine.mapData.value).toEqual({
      width: 5632,
      height: 2048,
      province_count: 1000
    })
    expect(mapEngine.defaultMap.value).not.toBeNull()
  })
})
