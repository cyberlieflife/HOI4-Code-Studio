import { ref, onUnmounted } from 'vue'
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

function normalizePath(path?: string): string {
  return path?.replace(/\\/g, '/').replace(/\/+$/, '') || ''
}

function normalizeRoots(paths: string[] = []): string[] {
  return Array.from(new Set(paths.map((path) => normalizePath(path)).filter(Boolean)))
}

function inferRootFromPreviewSource(path?: string): string {
  const normalized = normalizePath(path)
  if (!normalized) return ''
  const lower = normalized.toLowerCase()

  // 1. 如果直接指向 map/default.map，返回其所属根目录
  if (lower.endsWith('/map/default.map')) {
    return normalized.slice(0, -'/map/default.map'.length)
  }
  // 2. 如果指向 map 目录，返回其父目录
  if (lower.endsWith('/map')) {
    return normalized.slice(0, -'/map'.length)
  }
  // 3. 尝试从路径中查找 map 关键字并推断根目录（例如在 map/terrain/ 下的文件）
  const mapIdx = lower.lastIndexOf('/map/')
  if (mapIdx !== -1) {
    return normalized.slice(0, mapIdx)
  }

  return ''
}

function normalizeProjectOnlyMapPath(mapDir: string, relativePath: string): string {
  const normalized = relativePath.trim().replace(/\\/g, '/').replace(/^\/+/, '')
  if (!normalized) return mapDir
  return normalized.includes('/') ? `${mapDir}/${normalized}`.replace(/\/+/g, '/') : `${mapDir}/${normalized}`
}

/**
 * 地图引擎组合式 API
 *
 * 该引擎负责加载HOI4地图数据并提供渲染服务，支持两种初始化模式：
 * 1. fallback 模式：从项目根目录、依赖目录、游戏目录搜索地图资源，后找到的会被先找到的覆盖（state/colors合并）
 * 2. project-only 模式：仅加载项目自身的map文件，不进行任何覆盖逻辑，适用于编辑器内置预览
 */
export function useMapEngine(options?: { workerTimeout?: number }) {
  const workerTimeout = options?.workerTimeout ?? 5000
  const definitions = ref<ProvinceDefinition[]>([])
  const defaultMap = ref<DefaultMap | null>(null)
  const mapData = ref<MapMetadata | null>(null)
  const states = ref<StateDefinition[]>([])
  const countryColors = ref<Record<string, RGBColor>>({})
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // ==================== Web Worker 相关 ====================
  
  /**
   * Worker实例
   */
  let mapWorker: Worker | null = null
  
  /**
   * Worker是否可用
   */
  const workerAvailable = ref(false)
  
  /**
   * 待处理的任务队列
   */
  const pendingTasks = new Map<string, {
    resolve: (value: Uint32Array | number | null) => void
    reject: (reason: Error) => void
    timer: ReturnType<typeof setTimeout>
  }>()
  
  /**
   * 任务ID计数器
   */
  let taskIdCounter = 0
  
  /**
   * 初始化Worker
   */
  function initWorker() {
    try {
      // 动态导入Worker
      mapWorker = new Worker(
        new URL('../workers/mapWorker.ts', import.meta.url),
        { type: 'module' }
      )
      
      // 监听Worker消息
      mapWorker.onmessage = (event: MessageEvent) => {
        const { type, result } = event.data
        
        if (type === 'result' && result) {
          const task = pendingTasks.get(result.id)
          if (task) {
            clearTimeout(task.timer)
            if (result.success) {
              task.resolve(result.data)
            } else {
              task.reject(new Error(result.error || 'Worker任务失败'))
            }
            pendingTasks.delete(result.id)
          }
        }
      }
      
      // 监听Worker错误
      mapWorker.onerror = (error) => {
        console.error('Map Worker错误:', error)
        workerAvailable.value = false
        
        // 拒绝所有待处理任务
        pendingTasks.forEach((task) => {
          task.reject(new Error('Worker发生错误'))
        })
        pendingTasks.clear()
      }
      
      workerAvailable.value = true
      logMapEvent('worker:initialized')
    } catch (e) {
      console.warn('Worker初始化失败，将使用主线程:', e)
      workerAvailable.value = false
    }
  }
  
  /**
   * 销毁Worker
   */
  function destroyWorker() {
    if (mapWorker) {
      mapWorker.terminate()
      mapWorker = null
      workerAvailable.value = false
      
      // 拒绝所有待处理任务
      pendingTasks.forEach((task) => {
        task.reject(new Error('Worker已销毁'))
      })
      pendingTasks.clear()
      
      logMapEvent('worker:destroyed')
    }
  }
  
interface WorkerTask {
  type: 'getProvinceId' | 'getProvinceOutline' | 'getStateOutline' | 'addCache'
  id?: string
  x?: number
  y?: number
  provinceId?: number
  stateId?: number
  cacheKey?: string
  cacheData?: unknown
}

  /**
   * 向Worker发送任务
   */
  function sendTaskToWorker(task: WorkerTask): Promise<unknown> {
    return new Promise((resolve, reject) => {
      if (!mapWorker || !workerAvailable.value) {
        reject(new Error('Worker不可用'))
        return
      }
      
      const taskId = `task_${++taskIdCounter}_${Date.now()}`
      
      // 设置超时
      const timer = setTimeout(() => {
        if (pendingTasks.has(taskId)) {
          pendingTasks.delete(taskId)
          // 通知 Worker 取消任务
          if (mapWorker && workerAvailable.value) {
            mapWorker.postMessage({ type: 'cancel', taskId })
          }
          reject(new Error(`Worker任务超时 (${workerTimeout}ms)`))
        }
      }, workerTimeout)
      
      pendingTasks.set(taskId, { resolve, reject, timer })
      
      mapWorker.postMessage({
        type: 'task',
        task: { ...task, id: taskId }
      })
    })
  }
  
  /**
   * 向Worker添加缓存
   */
  function addCacheToWorker(key: string, data: number | Uint32Array) {
    if (mapWorker && workerAvailable.value) {
      mapWorker.postMessage({
        type: 'cache',
        cacheData: { key, data }
      })
    }
  }
  
  // 初始化Worker
  initWorker()
  
  // 组件卸载时销毁Worker
  onUnmounted(() => {
    destroyWorker()
  })

  async function initMap(
    projectPath: string,
    gameDirectory?: string,
    dependencyRoots: string[] = [],
    mode: MapMergeMode = 'fallback',
    previewSourcePath?: string
  ) {
    if (!projectPath) {
      error.value = '未指定项目路径'
      return
    }

    isLoading.value = true
    error.value = null

    const rootPath = normalizePath(projectPath)
    const normalizedGameDirectory = normalizePath(gameDirectory)
    const normalizedDependencyRoots = normalizeRoots(dependencyRoots)
    const normalizedPreviewSourcePath = normalizePath(previewSourcePath)
    const safeGameDirectory = normalizedGameDirectory || undefined

    try {
      logMapEvent('initMap:start', { projectPath: rootPath, mode })

      let initData

      if (mode === 'fallback') {
        // 使用 fallback 模式：搜索游戏目录和依赖
        initData = await measureMapAsync('frontend.initializeMapContextWithFallback', async () => (
          await initializeMapContextWithFallback(
            rootPath,
            safeGameDirectory,
            normalizedDependencyRoots
          )
        ))
      } else {
        // project-only 模式：仅使用项目自身的文件
        initData = await measureMapAsync('frontend.initializeMapContext', async () => {
          // 搜索优先级：previewSourcePath > projectPath > dependencyRoots > gameDirectory
          // 1. inferRootFromPreviewSource: 从预览源文件路径推断出的根目录
          // 2. rootPath: 当前项目的根目录
          // 3. normalizedDependencyRoots: 已启用的依赖项目录
          // 4. normalizedGameDirectory: 游戏安装目录
          const candidateRoots = Array.from(new Set([
            inferRootFromPreviewSource(normalizedPreviewSourcePath),
            rootPath,
            ...normalizedDependencyRoots,
            normalizedGameDirectory
          ].filter(Boolean)))

          let effectiveRoot = ''
          let defaultMapResult = null as Awaited<ReturnType<typeof loadDefaultMap>> | null

          for (const candidateRoot of candidateRoots) {
            const candidateMapPath = `${candidateRoot}/map/default.map`
            const result = await loadDefaultMap(candidateMapPath)
            if (result.success && result.data) {
              effectiveRoot = candidateRoot
              defaultMapResult = result
              break
            }
          }

          if (!defaultMapResult?.data || !effectiveRoot) {
            throw new Error('无法定位可预览的 map/default.map')
          }

          const defaultMapConfig = defaultMapResult.data
          const mapDir = `${effectiveRoot}/map`

          // 构造完整路径（全部相对于项目 map 目录）
          const definitionsPath = normalizeProjectOnlyMapPath(mapDir, defaultMapConfig.definitions)
          const provincesPath = normalizeProjectOnlyMapPath(mapDir, defaultMapConfig.provinces)
          const statesPath = `${effectiveRoot}/history/states`
          const countryColorsPath = `${effectiveRoot}/common/countries`

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
    } catch (e: unknown) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      error.value = errorMessage
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

  /**
   * 获取省份ID（支持Worker回退）
   */
  async function getProvinceId(x: number, y: number): Promise<number | null> {
    try {
      // 优先使用Worker
      if (workerAvailable.value) {
        try {
          const result = await sendTaskToWorker({
            type: 'getProvinceId',
            x,
            y
          }) as number | null
          
          // 缓存结果
          if (result !== null) {
            addCacheToWorker(`province_${x}_${y}`, result)
          }
          
          return result
        } catch (workerError) {
          console.warn('Worker查询失败，回退到主线程:', workerError)
        }
      }
      
      // 回退到主线程
      return await measureMapAsync('frontend.getProvinceAtPoint', async () => (
        await getProvinceAtPoint(x, y)
      ))
    } catch (e: unknown) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      console.error('获取省份ID失败:', e)
      alert(`获取省份ID失败: ${errorMessage}`)
      return null
    }
  }

  /**
   * 获取省份轮廓（支持Worker回退）
   */
  async function getOutline(provinceId: number): Promise<Uint32Array> {
    try {
      // 优先使用Worker
      if (workerAvailable.value) {
        try {
          const result = await sendTaskToWorker({
            type: 'getProvinceOutline',
            provinceId
          }) as Uint32Array | null
          
          // 缓存结果
          if (result) {
            addCacheToWorker(`outline_${provinceId}`, result)
          }
          
          return result ?? new Uint32Array()
        } catch (workerError) {
          console.warn('Worker计算失败，回退到主线程:', workerError)
        }
      }
      
      // 回退到主线程
      return await measureMapAsync('frontend.getProvinceOutline', async () => (
        await getProvinceOutline(provinceId)
      ))
    } catch (e: unknown) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      console.error('获取省份轮廓失败:', e)
      alert(`获取省份轮廓失败: ${errorMessage}`)
      return new Uint32Array()
    }
  }

  /**
   * 获取地区轮廓（支持Worker回退）
   */
  async function getStateOutlineWrapper(stateId: number): Promise<Uint32Array> {
    try {
      // 优先使用Worker
      if (workerAvailable.value) {
        try {
          const result = await sendTaskToWorker({
            type: 'getStateOutline',
            stateId
          }) as Uint32Array | null
          
          // 缓存结果
          if (result) {
            addCacheToWorker(`state_${stateId}`, result)
          }
          
          return result ?? new Uint32Array()
        } catch (workerError) {
          console.warn('Worker计算失败，回退到主线程:', workerError)
        }
      }
      
      // 回退到主线程
      return await measureMapAsync('frontend.getStateOutline', async () => (
        await getStateOutline(stateId)
      ))
    } catch (e: unknown) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      console.error('获取地区轮廓失败:', e)
      alert(`获取地区轮廓失败: ${errorMessage}`)
      return new Uint32Array()
    }
  }

  return {
    definitions,
    defaultMap,
    mapData,
    states,
    countryColors,
    isLoading,
    error,
    workerAvailable,
    initMap,
    renderTile,
    getPreview,
    getProvinceId,
    getOutline,
    getStateOutline: getStateOutlineWrapper
  }
}
