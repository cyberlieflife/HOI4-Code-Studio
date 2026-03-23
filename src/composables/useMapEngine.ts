import { ref } from 'vue'
import {
  type ProvinceDefinition,
  type DefaultMap,
  type StateDefinition,
  type RGBColor,
  initializeMapContext,
  initializeMapContextWithFallback,
  getMapTileDirect,
  getMapPreview,
  getProvinceAtPoint,
  getProvinceOutline,
  getStateOutline,
  type MapMetadata,
  loadDefaultMap
} from '../api/tauri'
import { logMapEvent, measureMapAsync } from '../utils/mapPerformance'

/**
 * 地图覆盖模式
 * - 'fallback': 使用游戏目录和依赖作为fallback，项目文件优先（右键地图预览使用）
 * - 'project-only': 仅使用项目自身的文件，不包含game目录和依赖（编辑器内置预览使用）
 */
export type MapMergeMode = 'fallback' | 'project-only'

/**
 * 地图引擎组合式 API
 *
 * 该引擎负责加载HOI4地图数据并提供渲染服务，支持两种初始化模式：
 * 1. fallback 模式：从项目根目录、依赖目录、游戏目录搜索地图资源，后找到的会被先找到的覆盖（state/colors合并）
 * 2. project-only 模式：仅加载项目自身的map文件，不进行任何覆盖逻辑，适用于编辑器内置预览
 */
export function useMapEngine() {
  const definitions = ref<ProvinceDefinition[]>([])
  const defaultMap = ref<DefaultMap | null>(null)
  const mapData = ref<MapMetadata | null>(null)
  const states = ref<StateDefinition[]>([])
  const countryColors = ref<Record<string, RGBColor>>({})
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  async function initMap(
    projectPath: string,
    gameDirectory?: string,
    dependencyRoots: string[] = [],
    mode: MapMergeMode = 'fallback'
  ) {
    if (!projectPath) {
      error.value = '未指定项目路径'
      return
    }

    isLoading.value = true
    error.value = null

    const normalize = (p: string) => p.replace(/\\/g, '/').replace(/\/+$/, '')
    const rootPath = normalize(projectPath)

    try {
      logMapEvent('initMap:start', { projectPath: rootPath, mode })

      let initData

      if (mode === 'fallback') {
        // 使用 fallback 模式：搜索游戏目录和依赖
        initData = await measureMapAsync('frontend.initializeMapContextWithFallback', async () => (
          await initializeMapContextWithFallback(
            rootPath,
            gameDirectory,
            dependencyRoots
          )
        ))
      } else {
        // project-only 模式：仅使用项目自身的文件
        initData = await measureMapAsync('frontend.initializeMapContext', async () => {
          // 解析项目内的 default.map
          const mapDir = `${rootPath}/map`
          const defaultMapPath = `${mapDir}/default.map`

          // 读取并解析 default.map（使用 Tauri API）
          const defaultMapResult = await loadDefaultMap(defaultMapPath)
          if (!defaultMapResult.success || !defaultMapResult.data) {
            throw new Error(defaultMapResult.message || '无法加载项目 map/default.map')
          }

          const defaultMapConfig = defaultMapResult.data

          // 构造完整路径（全部相对于项目 map 目录）
          const definitionsPath = `${mapDir}/${defaultMapConfig.definitions}`
          const provincesPath = `${mapDir}/${defaultMapConfig.provinces}`
          const statesPath = `${rootPath}/history/states`
          const countryColorsPath = `${rootPath}/common/countries/colors.txt`

          // 直接初始化，不使用 fallback
          const result = await initializeMapContext(
            provincesPath,
            definitionsPath,
            statesPath,
            countryColorsPath
          )

          // 手动添加 defaultMap 以便前端使用
          result.defaultMap = defaultMapConfig
          return result
        })
      }

      defaultMap.value = initData.defaultMap ?? null
      mapData.value = initData.metadata
      definitions.value = initData.definitions
      states.value = initData.states

      logMapEvent('initMap:done', {
        width: initData.metadata.width,
        height: initData.metadata.height,
        definitions: definitions.value.length,
        states: states.value.length,
        mode
      })
    } catch (e: any) {
      error.value = e.message
      console.error('Map init error:', e)
    } finally {
      isLoading.value = false
    }
  }

  async function renderTile(x: number, y: number, zoom: number, mode: string): Promise<Uint8Array> {
    return await measureMapAsync(`frontend.getMapTileDirect(${mode})`, async () => (
      await getMapTileDirect(x, y, zoom, mode)
    ))
  }

  async function getPreview(width: number, height: number, mode: string): Promise<Uint8Array> {
    return await measureMapAsync(`frontend.getMapPreview(${mode})`, async () => (
      await getMapPreview(width, height, mode)
    ))
  }

  async function getProvinceId(x: number, y: number): Promise<number | null> {
    return await measureMapAsync('frontend.getProvinceAtPoint', async () => (
      await getProvinceAtPoint(x, y)
    ))
  }

  async function getOutline(provinceId: number): Promise<Uint32Array> {
    return await measureMapAsync('frontend.getProvinceOutline', async () => (
      await getProvinceOutline(provinceId)
    ))
  }

  async function getStateOutlineWrapper(stateId: number): Promise<Uint32Array> {
    return await measureMapAsync('frontend.getStateOutline', async () => (
      await getStateOutline(stateId)
    ))
  }

  return {
    definitions,
    defaultMap,
    mapData,
    states,
    countryColors,
    isLoading,
    error,
    initMap,
    renderTile,
    getPreview,
    getProvinceId,
    getOutline,
    getStateOutline: getStateOutlineWrapper
  }
}
