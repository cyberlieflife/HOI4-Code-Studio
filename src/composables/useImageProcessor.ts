/**
 * 图片处理管理器
 * 管理Web Worker池，实现多线程图片处理和渐进式加载
 */

import { ref, shallowRef } from 'vue'
import { loadFocusIcon as loadFocusIconApi, readIconCache, writeIconCache } from '../api/tauri'

// 定义接口
interface ImageLoadTask {
  id: string
  iconName: string
  projectPath?: string
  gameDirectory?: string
  priority: 'high' | 'normal' | 'low'
  onSuccess?: (dataUrl: string) => void
  onError?: (error: string) => void
  onProgress?: (loaded: number, total: number) => void
}



interface ProcessingStats {
  totalTasks: number
  completedTasks: number
  loadingTasks: number
  cacheHits: number
  cacheMisses: number
}

interface WorkerMessage {
  type: 'load' | 'result' | 'error' | 'cache'
  task?: ImageLoadTask
  result?: {
    id: string
    success: boolean
    dataUrl?: string
    error?: string
  }
  cacheData?: { iconName: string; dataUrl: string }
}

// Worker池管理
class WorkerPool {
  private workers: Worker[] = []
  private availableWorkers: Worker[] = []
  private busyWorkers = new Set<Worker>()
  private taskQueue: ImageLoadTask[] = []
  private pendingTasks = new Map<string, ImageLoadTask>()
  private readonly maxWorkers: number

  constructor(maxWorkers: number = 4) {
    this.maxWorkers = Math.min(maxWorkers, navigator.hardwareConcurrency || 4)
    this.initWorkers()
  }

  /**
   * 初始化Worker池
   */
  private async initWorkers() {
    for (let i = 0; i < this.maxWorkers; i++) {
      try {
        const worker = new Worker(new URL('../workers/imageWorker.ts', import.meta.url), {
          type: 'module'
        })

        worker.addEventListener('message', (event) => {
          this.handleWorkerMessage(worker, event.data)
        })

        this.workers.push(worker)
        this.availableWorkers.push(worker)
      } catch (error) {
        console.error(`创建Worker失败 #${i}:`, error)
      }
    }
  }

  /**
   * 处理Worker消息
   */
  private handleWorkerMessage(worker: Worker, message: WorkerMessage) {
    const { type, result } = message

    if (type === 'result' && result) {
      const task = this.pendingTasks.get(result.id)
      if (task) {
        // 清理任务
        this.pendingTasks.delete(result.id)
        
        // 释放Worker
        this.busyWorkers.delete(worker)
        this.availableWorkers.push(worker)

        // 执行回调
        if (result.success && task.onSuccess) {
          task.onSuccess(result.dataUrl ?? '')
        } else if (!result.success && task.onError) {
          task.onError(result.error ?? '未知错误')
        }

        // 处理队列中的下一个任务
        this.processNextTask()
      }
    }
  }

  /**
   * 添加任务
   */
  addTask(task: ImageLoadTask) {
    // 检查是否已在处理
    if (this.pendingTasks.has(task.id)) {
      return
    }

    this.taskQueue.push(task)
    this.processNextTask()
  }

  /**
   * 处理下一个任务
   */
  private processNextTask() {
    if (this.availableWorkers.length === 0 || this.taskQueue.length === 0) {
      return
    }

    // 按优先级排序
    this.taskQueue.sort((a, b) => {
      const priorityOrder = { high: 0, normal: 1, low: 2 }
      return priorityOrder[a.priority] - priorityOrder[b.priority]
    })

    const task = this.taskQueue.shift()!
    const worker = this.availableWorkers.shift()!

    // 标记为忙碌
    this.busyWorkers.add(worker)
    this.pendingTasks.set(task.id, task)

    // 发送任务给Worker
    worker.postMessage({
      type: 'load',
      task
    })
  }

  /**
   * 获取统计信息
   */
  getStats(): ProcessingStats {
    return {
      totalTasks: this.taskQueue.length + this.busyWorkers.size,
      completedTasks: this.workers.length * 10 - this.busyWorkers.size, // 粗略估算
      loadingTasks: this.busyWorkers.size,
      cacheHits: 0, // 需要从Worker获取
      cacheMisses: 0
    }
  }

  /**
   * 清理资源
   */
  dispose() {
    this.workers.forEach(worker => worker.terminate())
    this.workers = []
    this.availableWorkers = []
    this.busyWorkers.clear()
    this.taskQueue = []
    this.pendingTasks.clear()
  }
}

// 全局Worker池实例
let globalWorkerPool: WorkerPool | null = null

/**
 * 使用图片处理器
 */
export function useImageProcessor() {
  const isProcessing = ref(false)
  const stats = ref<ProcessingStats>({
    totalTasks: 0,
    completedTasks: 0,
    loadingTasks: 0,
    cacheHits: 0,
    cacheMisses: 0
  })

  // 主线程缓存（与Worker缓存同步）
  const mainThreadCache = shallowRef(new Map<string, string>())

  /**
   * 初始化Worker池
   */
  function initWorkerPool(maxWorkers?: number) {
    if (!globalWorkerPool) {
      globalWorkerPool = new WorkerPool(maxWorkers)
    }
    return globalWorkerPool
  }

/**
 * 加载单个图标（异步非阻塞）
 */
function loadIconAsync(
  iconName: string,
  options: {
    projectPath?: string
    gameDirectory?: string
    priority?: 'high' | 'normal' | 'low'
    onSuccess?: (dataUrl: string) => void
    onError?: (error: string) => void
  } = {}
): Promise<string | null> {
  return new Promise((resolve, reject) => {
    // 检查主线程缓存
    if (mainThreadCache.value.has(iconName)) {
      const cachedUrl = mainThreadCache.value.get(iconName)!
      options.onSuccess?.(cachedUrl)
      resolve(cachedUrl)
      return
    }

    // 异步加载图标（不阻塞主线程）
    loadIconMainThread({
      id: `icon_${iconName}`,
      iconName,
      projectPath: options.projectPath,
      gameDirectory: options.gameDirectory,
      priority: options.priority || 'normal'
    }).then(dataUrl => {
      options.onSuccess?.(dataUrl ?? '')
      resolve(dataUrl)
    }).catch(error => {
      options.onError?.(error)
      reject(error)
    })
  })
}

  /**
   * 主线程加载图标（备用方案）
   */
  async function loadIconMainThread(task: ImageLoadTask): Promise<string | null> {
    const { iconName, projectPath, gameDirectory } = task

    try {
      // 检查主线程缓存
      if (mainThreadCache.value.has(iconName)) {
        return mainThreadCache.value.get(iconName)!
      }

      // 尝试从磁盘缓存读取
      const cacheResult = await readIconCache(iconName)
      const cacheMime = cacheResult.mimeType || cacheResult.mime_type
      if (cacheResult.success && cacheResult.base64 && cacheMime) {
        console.info(`[icon] cache hit: ${iconName}`)
        const dataUrl = `data:${cacheMime};base64,${cacheResult.base64}`
        
        // 更新主线程缓存
        const newCache = new Map(mainThreadCache.value)
        newCache.set(iconName, dataUrl)
        mainThreadCache.value = newCache
        
        return dataUrl
      }

      console.info(`[icon] cache miss: ${iconName}, loading via load_focus_icon`)

      // 调用API获取
      const result = await loadFocusIconApi(iconName, projectPath, gameDirectory)
      
      if (result.success && result.base64 && result.mimeType) {
        const dataUrl = `data:${result.mimeType};base64,${result.base64}`
        
        // 更新缓存
        const newCache = new Map(mainThreadCache.value)
        newCache.set(iconName, dataUrl)
        mainThreadCache.value = newCache
        
        // 异步写入磁盘缓存
        writeIconCache(iconName, result.base64, result.mimeType)
          .then((resp) => {
            if (resp?.success) {
              console.info(`[icon] cache write ok: ${iconName}`)
            } else {
              console.warn(`[icon] cache write fail: ${iconName}`, resp?.message)
            }
          })
          .catch(error => {
            console.warn(`[icon] cache write error: ${iconName}`, error)
          })
        
        return dataUrl
      }

      throw new Error(result.message || '加载失败')

    } catch (error) {
      console.warn(`主线程加载图标失败: ${iconName}`, error)
      return null
    }
  }

  /**
   * 批量加载图标（渐进式）
   */
  async function loadIconsBatch(
    iconNames: string[],
    options: {
      projectPath?: string
      gameDirectory?: string
      onProgress?: (loaded: number, total: number) => void
      onItemLoaded?: (iconName: string, dataUrl: string) => void
      priority?: 'high' | 'normal' | 'low'
    } = {}
  ) {
    let loaded = 0
    const total = iconNames.length

    // 并发加载，但不使用Promise.all以避免阻塞
    const loadPromises = iconNames.map(async (iconName) => {
      try {
        const dataUrl = await loadIconAsync(iconName, {
          ...options,
          onSuccess: (dataUrl) => {
            loaded++
            options.onProgress?.(loaded, total)
            options.onItemLoaded?.(iconName, dataUrl)
          }
        })
        return { iconName, dataUrl, success: true }
      } catch (error) {
        loaded++
        options.onProgress?.(loaded, total)
        return { iconName, error: error instanceof Error ? error.message : '未知错误', success: false }
      }
    })

    return Promise.all(loadPromises)
  }

  /**
   * 预加载图标（低优先级）
   */
  function preloadIcons(
    iconNames: string[],
    options: {
      projectPath?: string
      gameDirectory?: string
    } = {}
  ) {
    iconNames.forEach(iconName => {
      loadIconAsync(iconName, {
        ...options,
        priority: 'low'
      }).catch(() => {
        // 预加载失败不影响主流程
      })
    })
  }

  /**
   * 获取缓存的图标
   */
  function getCachedIcon(iconName: string): string | null {
    return mainThreadCache.value.get(iconName) || null
  }

  /**
   * 清理缓存
   */
  function clearCache() {
    mainThreadCache.value.clear()
    // 可以添加清理Worker缓存的逻辑
  }

  /**
   * 获取缓存大小
   */
  function getCacheSize(): number {
    return mainThreadCache.value.size
  }

  /**
   * 销毁处理器
   */
  function dispose() {
    if (globalWorkerPool) {
      globalWorkerPool.dispose()
      globalWorkerPool = null
    }
    clearCache()
  }

  return {
    // 状态
    isProcessing,
    stats,
    
    // 方法
    initWorkerPool,
    loadIconAsync,
    loadIconsBatch,
    preloadIcons,
    getCachedIcon,
    clearCache,
    getCacheSize,
    dispose
  }
}