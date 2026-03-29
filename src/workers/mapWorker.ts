/**
 * 地图预览Web Worker
 * 用于处理计算密集型任务，如省份查询、轮廓计算等
 * 实际的API调用仍在主线程进行（避免Worker中导入Tauri API的复杂性）
 */

// ==================== 类型定义 ====================

/**
 * 省份查询任务
 */
interface ProvinceQueryTask {
  id: string
  type: 'getProvinceId'
  x: number
  y: number
}

/**
 * 省份轮廓计算任务
 */
interface ProvinceOutlineTask {
  id: string
  type: 'getProvinceOutline'
  provinceId: number
}

/**
 * 地区轮廓计算任务
 */
interface StateOutlineTask {
  id: string
  type: 'getStateOutline'
  stateId: number
}

/**
 * 通用地图任务类型
 */
type MapTask = ProvinceQueryTask | ProvinceOutlineTask | StateOutlineTask

/**
 * 任务结果
 */
interface TaskResult {
  id: string
  success: boolean
  data?: number | Uint32Array
  error?: string
}

/**
 * Worker消息类型
 */
interface WorkerMessage {
  type: 'task' | 'result' | 'error' | 'cache'
  task?: MapTask
  result?: TaskResult
  cacheData?: { key: string; data: number | Uint32Array }
}

// ==================== 缓存管理 ====================

/**
 * 结果缓存（Worker内存）
 * 用于缓存频繁查询的结果，减少主线程调用
 */
const workerCache = new Map<string, number | Uint32Array>()

/**
 * 缓存大小限制
 */
const MAX_CACHE_SIZE = 1000

/**
 * 添加缓存数据
 */
function addCache(key: string, data: number | Uint32Array) {
  // 如果缓存已满，清理最早的一半
  if (workerCache.size >= MAX_CACHE_SIZE) {
    const keys = Array.from(workerCache.keys())
    const keysToDelete = keys.slice(0, Math.floor(MAX_CACHE_SIZE / 2))
    keysToDelete.forEach(k => workerCache.delete(k))
  }
  workerCache.set(key, data)
}

/**
 * 获取缓存数据
 */
function getCache(key: string): number | Uint32Array | undefined {
  return workerCache.get(key)
}

/**
 * 清理缓存
 */
function clearCache() {
  workerCache.clear()
}

/**
 * 获取缓存统计
 */
function getCacheStats() {
  let memoryUsage = 0
  workerCache.forEach((data) => {
    if (typeof data === 'number') {
      memoryUsage += 8
    } else {
      memoryUsage += data.byteLength
    }
  })
  return {
    size: workerCache.size,
    memoryUsage
  }
}

// ==================== 任务处理 ====================

/**
 * 生成缓存键
 */
function generateCacheKey(task: MapTask): string {
  switch (task.type) {
    case 'getProvinceId':
      return `province_${task.x}_${task.y}`
    case 'getProvinceOutline':
      return `outline_${task.provinceId}`
    case 'getStateOutline':
      return `state_${task.stateId}`
    default:
      return `unknown_${Date.now()}`
  }
}

/**
 * 处理省份查询任务
 * 注意：实际的API调用由主线程处理，这里只是任务调度
 */
async function handleProvinceQuery(task: ProvinceQueryTask): Promise<TaskResult> {
  const { id } = task
  
  try {
    // 检查缓存
    const cacheKey = generateCacheKey(task)
    const cached = getCache(cacheKey)
    if (cached !== undefined) {
      return {
        id,
        success: true,
        data: cached
      }
    }

    // 这里返回一个占位结果，实际的数据由主线程处理
    // 主线程会调用Tauri API获取真实数据
    return {
      id,
      success: false,
      error: 'Worker模式：请使用主线程API进行实际查询'
    }
  } catch (error) {
    return {
      id,
      success: false,
      error: error instanceof Error ? error.message : '查询异常'
    }
  }
}

/**
 * 处理省份轮廓计算任务
 */
async function handleProvinceOutline(task: ProvinceOutlineTask): Promise<TaskResult> {
  const { id } = task
  
  try {
    // 检查缓存
    const cacheKey = generateCacheKey(task)
    const cached = getCache(cacheKey)
    if (cached !== undefined) {
      return {
        id,
        success: true,
        data: cached
      }
    }

    // 这里返回一个占位结果，实际的数据由主线程处理
    return {
      id,
      success: false,
      error: 'Worker模式：请使用主线程API进行实际计算'
    }
  } catch (error) {
    return {
      id,
      success: false,
      error: error instanceof Error ? error.message : '轮廓计算异常'
    }
  }
}

/**
 * 处理地区轮廓计算任务
 */
async function handleStateOutline(task: StateOutlineTask): Promise<TaskResult> {
  const { id } = task
  
  try {
    // 检查缓存
    const cacheKey = generateCacheKey(task)
    const cached = getCache(cacheKey)
    if (cached !== undefined) {
      return {
        id,
        success: true,
        data: cached
      }
    }

    // 这里返回一个占位结果，实际的数据由主线程处理
    return {
      id,
      success: false,
      error: 'Worker模式：请使用主线程API进行实际计算'
    }
  } catch (error) {
    return {
      id,
      success: false,
      error: error instanceof Error ? error.message : '地区轮廓计算异常'
    }
  }
}

/**
 * 处理任务
 */
async function processTask(task: MapTask): Promise<TaskResult> {
  switch (task.type) {
    case 'getProvinceId':
      return await handleProvinceQuery(task)
    case 'getProvinceOutline':
      return await handleProvinceOutline(task)
    case 'getStateOutline':
      return await handleStateOutline(task)
    default:
      return {
        id: (task as MapTask).id,
        success: false,
        error: '未知任务类型'
      }
  }
}

// ==================== 消息处理 ====================

/**
 * 处理主线程消息
 */
self.addEventListener('message', async (event: MessageEvent<WorkerMessage>) => {
  const { type, task, cacheData } = event.data

  switch (type) {
    case 'task':
      if (task) {
        const result = await processTask(task)
        self.postMessage({
          type: 'result',
          result
        } as WorkerMessage)
      }
      break

    case 'cache':
      if (cacheData) {
        addCache(cacheData.key, cacheData.data)
      }
      break

    default:
      console.warn('Map Worker: 未知消息类型', type)
  }
})

// ==================== 导出接口 ====================

export { clearCache, getCacheStats }
