import { ref } from 'vue'
import {
  type ProvinceDefinition,
  type DefaultMap,
  type StateDefinition,
  type RGBColor,
  initializeMapContextWithFallback,
  getMapTileDirect,
  getMapPreview,
  getProvinceAtPoint,
  getProvinceOutline,
  getStateOutline,
  type MapMetadata
} from '../api/tauri'
import { logMapEvent, measureMapAsync } from '../utils/mapPerformance'

/**
 * 地图引擎组合式 API
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
    dependencyRoots: string[] = []
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
      logMapEvent('initMap:start', { projectPath: rootPath })

      const initData = await measureMapAsync('frontend.initializeMapContextWithFallback', async () => (
        await initializeMapContextWithFallback(
          rootPath,
          gameDirectory,
          dependencyRoots
        )
      ))

      defaultMap.value = initData.defaultMap ?? null
      mapData.value = initData.metadata
      definitions.value = initData.definitions
      states.value = initData.states

      logMapEvent('initMap:done', {
        width: initData.metadata.width,
        height: initData.metadata.height,
        definitions: definitions.value.length,
        states: states.value.length
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
