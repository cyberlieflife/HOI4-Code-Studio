<template>
  <div class="w-full h-full overflow-hidden relative bg-hoi4-gray/50 flex flex-col">
    <!-- 顶部工具栏 -->
    <div class="flex-none p-2 bg-hoi4-border/50 border-b border-hoi4-border flex items-center justify-between z-10">
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-2 bg-hoi4-gray rounded px-2 py-1">
          <span class="text-hoi4-text text-sm font-semibold">地图模式:</span>
          <div class="flex gap-1">
            <button 
              v-for="m in modes" 
              :key="m.id"
              class="px-3 py-1 text-xs rounded transition-colors"
              :class="currentMode === m.id ? 'bg-hoi4-accent text-white' : 'bg-hoi4-border text-hoi4-text hover:bg-hoi4-border/80'"
              @click="setMode(m.id)"
            >
              {{ m.name }}
            </button>
          </div>
        </div>
        
        <div v-if="isLoading" class="flex items-center gap-2 text-hoi4-accent">
          <svg class="animate-spin h-4 w-4" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <span class="text-xs">加载中...</span>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <span v-if="error" class="text-red-400 text-xs px-2">{{ error }}</span>
        
        <!-- 高亮模式切换 -->
        <div v-if="isHighlightEnabled" class="flex items-center bg-hoi4-gray rounded px-1 py-1">
          <button 
            class="px-2 py-0.5 text-[10px] rounded transition-colors"
            :class="highlightMode === 'tile' ? 'bg-hoi4-accent text-white' : 'text-hoi4-text hover:bg-hoi4-border/80'"
            @click="highlightMode = 'tile'"
          >
            地块
          </button>
          <button 
            class="px-2 py-0.5 text-[10px] rounded transition-colors"
            :class="highlightMode === 'province' ? 'bg-hoi4-accent text-white' : 'text-hoi4-text hover:bg-hoi4-border/80'"
            @click="highlightMode = 'province'"
          >
            省份
          </button>
        </div>

        <!-- 高亮切换 -->
        <label class="flex items-center gap-2 px-2 py-1 bg-hoi4-gray rounded cursor-pointer hover:bg-hoi4-border transition-colors">
          <input 
            type="checkbox" 
            v-model="isHighlightEnabled" 
            class="w-3 h-3 accent-hoi4-accent"
          >
          <span class="text-hoi4-text text-xs">高亮</span>
        </label>

        <!-- 性能模式切换 -->
        <label class="flex items-center gap-2 px-2 py-1 bg-hoi4-gray rounded cursor-pointer hover:bg-hoi4-border transition-colors">
          <input 
            type="checkbox" 
            v-model="isPerformanceMode" 
            class="w-3 h-3 accent-hoi4-accent"
          >
          <span class="text-hoi4-text text-xs">性能模式</span>
        </label>

        <button 
          class="px-3 py-1 bg-hoi4-border text-hoi4-text text-xs rounded hover:bg-hoi4-border/80"
          @click="refreshMap"
        >
          刷新地图
        </button>
      </div>
    </div>

    <!-- 地图渲染区域 -->
    <div 
      ref="containerRef"
      class="flex-grow overflow-hidden relative"
      @wheel="handleWheel"
      @mousedown="handleMouseDown"
      @mousemove="handleMouseMove"
      @mouseup="handleMouseUp"
      @mouseleave="handleMouseUp"
      @contextmenu.prevent
    >
      <div
        :style="mapWrapperStyle"
        class="absolute top-0 left-0"
      >
        <canvas ref="canvasRef" class="block w-full h-full absolute top-0 left-0"></canvas>
        <canvas ref="overlayCanvasRef" class="block w-full h-full absolute top-0 left-0 pointer-events-none"></canvas>
      </div>
      
      <!-- 悬停信息 Tooltip -->
      <div 
        v-if="hoverInfo"
        class="absolute pointer-events-none ui-island rounded-2xl backdrop-blur-sm px-4 py-3 min-w-[180px] max-w-[320px] shadow-2xl z-30 transition-all duration-75 border border-hoi4-border/40"
        :style="tooltipStyle"
      >
        <!-- 州视图 (省份视图) -->
        <template v-if="hoverInfo.isState">
          <div class="text-hoi4-text font-bold text-sm tracking-wide mb-2 flex items-center justify-between">
            <span>{{ hoverInfo.name || '未命名州' }}</span>
            <span class="text-[10px] bg-hoi4-accent/20 text-hoi4-accent px-1.5 py-0.5 rounded uppercase tracking-tighter">STATE</span>
          </div>
          <div class="h-px bg-hoi4-border/30 mb-3"></div>
          <div class="space-y-3">
            <div class="flex justify-between items-center text-xs">
              <span class="text-hoi4-text-dim">ID:</span>
              <span class="text-hoi4-text font-mono">{{ hoverInfo.id }}</span>
            </div>
            
            <div class="flex justify-between items-start text-xs">
              <span class="text-hoi4-text-dim mt-0.5">拥有者:</span>
              <span class="text-hoi4-accent font-bold bg-hoi4-accent/10 px-1.5 py-0.5 rounded">{{ hoverInfo.owner || '无' }}</span>
            </div>

            <div class="flex flex-col gap-1.5">
              <span class="text-hoi4-text-dim text-[10px] uppercase tracking-wider">核心国家 (Cores):</span>
              <div class="flex flex-wrap gap-1">
                <template v-if="hoverInfo.cores.length">
                  <span 
                    v-for="core in hoverInfo.cores" 
                    :key="core"
                    class="text-[10px] bg-hoi4-border/60 text-hoi4-text px-1.5 py-0.5 rounded font-mono border border-hoi4-border/40"
                  >
                    {{ core }}
                  </span>
                </template>
                <span v-else class="text-hoi4-comment text-[10px] italic">无</span>
              </div>
            </div>

            <div class="flex flex-col gap-1.5">
              <span class="text-hoi4-text-dim text-[10px] uppercase tracking-wider">宣称国家 (Claims):</span>
              <div class="flex flex-wrap gap-1">
                <template v-if="hoverInfo.claims.length">
                  <span 
                    v-for="claim in hoverInfo.claims" 
                    :key="claim"
                    class="text-[10px] bg-hoi4-border/40 text-hoi4-text-dim px-1.5 py-0.5 rounded font-mono"
                  >
                    {{ claim }}
                  </span>
                </template>
                <span v-else class="text-hoi4-comment text-[10px] italic">无</span>
              </div>
            </div>
          </div>
        </template>

        <!-- 地块视图 -->
        <template v-else-if="!hoverInfo.isState">
          <div class="text-hoi4-text font-bold text-sm tracking-wide mb-2 flex items-center justify-between">
            <span>地块 #{{ hoverInfo.id }}</span>
            <span class="text-[10px] bg-hoi4-comment/20 text-hoi4-comment px-1.5 py-0.5 rounded uppercase tracking-tighter">PROVINCE</span>
          </div>
          <div class="h-px bg-hoi4-border/30 mb-3"></div>
          <div class="grid grid-cols-2 gap-x-4 gap-y-2 text-xs">
            <div class="flex flex-col">
              <span class="text-hoi4-text-dim text-[10px] uppercase">类型</span>
              <span class="text-hoi4-text">{{ hoverInfo.type }}</span>
            </div>
            <div class="flex flex-col">
              <span class="text-hoi4-text-dim text-[10px] uppercase">地形</span>
              <span class="text-hoi4-text">{{ hoverInfo.terrain }}</span>
            </div>
            <div class="flex flex-col">
              <span class="text-hoi4-text-dim text-[10px] uppercase">沿海</span>
              <span class="text-hoi4-text">{{ hoverInfo.coastal ? '是' : '否' }}</span>
            </div>
            <div v-if="hoverInfo.owner" class="flex flex-col">
              <span class="text-hoi4-text-dim text-[10px] uppercase">所有者</span>
              <span class="text-hoi4-accent font-bold">{{ hoverInfo.owner }}</span>
            </div>
          </div>
          <div v-if="hoverInfo.stateName" class="mt-3 pt-2 border-t border-hoi4-border/20">
            <div class="flex flex-col">
              <span class="text-hoi4-text-dim text-[10px] uppercase">所属州</span>
              <span class="text-hoi4-text">{{ hoverInfo.stateName }}</span>
            </div>
          </div>
        </template>
      </div>

      <!-- 加载遮罩 -->
      <div v-if="isLoading" class="absolute inset-0 bg-black/20 flex items-center justify-center backdrop-blur-[1px] z-50">
        <div class="bg-hoi4-gray/90 p-4 rounded-lg border border-hoi4-border shadow-xl flex flex-col items-center gap-3 w-64">
          <div class="text-hoi4-accent font-bold">{{ loadingStatus }}</div>
          
          <!-- 进度条 -->
          <div class="w-full h-1.5 bg-black/40 rounded-full overflow-hidden">
            <div 
              class="h-full bg-hoi4-accent transition-all duration-300 ease-out"
              :style="{ width: `${loadingProgress}%` }"
            ></div>
          </div>
          
          <div class="text-xs text-hoi4-comment flex justify-between w-full">
            <span>{{ loadingDetail }}</span>
            <span>{{ loadingProgress }}%</span>
          </div>
        </div>
      </div>

      <!-- 导航器 (Minimap) -->
      <div 
        v-if="mapData"
        class="absolute bottom-4 right-4 ui-island rounded-2xl backdrop-blur-sm p-1.5 border border-hoi4-border/40 shadow-2xl z-20 overflow-hidden group select-none transition-all duration-300 hover:scale-[1.02]"
        :style="{ width: MINIMAP_SIZE + 12 + 'px' }"
      >
        <div class="relative w-full h-full cursor-pointer" @mousedown="handleMinimapClick">
          <canvas ref="minimapCanvasRef" class="rounded-xl w-full h-auto block bg-black/20"></canvas>
          
          <!-- 视口指示框 -->
          <div 
            class="absolute border-2 border-hoi4-accent shadow-[0_0_0_1px_rgba(0,0,0,0.5)] pointer-events-none transition-[width,height,left,top] duration-75"
            :style="{
              left: viewportRect.x + 'px',
              top: viewportRect.y + 'px',
              width: viewportRect.w + 'px',
              height: viewportRect.h + 'px'
            }"
          ></div>
        </div>
        
        <!-- 导航器标题 -->
        <div class="absolute top-2 left-3 text-[10px] font-bold text-white/40 group-hover:text-hoi4-accent transition-colors pointer-events-none">
          NAVIGATOR
        </div>
      </div>
    </div>

    <!-- 底部状态栏 -->
    <div class="flex-none px-4 py-1 bg-hoi4-border/50 border-t border-hoi4-border text-[10px] text-hoi4-comment flex justify-between">
      <div class="flex gap-4">
        <span>缩放: {{ Math.round(scale * 100) }}%</span>
        <span v-if="mapData">分辨率: {{ mapData.width }} × {{ mapData.height }}</span>
      </div>
      <div v-if="hoverInfo" class="flex gap-4 text-hoi4-accent font-mono">
        <span>X: {{ mouseMapPos.x }}, Y: {{ mouseMapPos.y }}</span>
        <span>省份 ID: {{ hoverInfo.id }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { useMapEngine } from '../../composables/useMapEngine'
import { loadSettings, type ProvinceDefinition, type StateDefinition } from '../../api/tauri'
import { logMapEvent, measureMapAsync, measureMapSync } from '../../utils/mapPerformance'

const props = defineProps<{
  projectPath: string
}>()

// 加载全局设置
const globalPerformanceMode = ref(true)
const globalSamplingRate = ref(50)

onMounted(async () => {
  const result = await loadSettings()
  if (result.success && result.data) {
    const data = result.data as any
    globalPerformanceMode.value = data.mapPerformanceMode !== false
    globalSamplingRate.value = data.mapSamplingRate || (data.mapDownsample ? Math.round(100 / data.mapDownsample) : 50)
    isPerformanceMode.value = globalPerformanceMode.value
  }
})

// 计算下采样因子
const downsampleFactor = computed(() => {
  if (!isPerformanceMode.value) return 1
  return Math.max(1, Math.round(100 / globalSamplingRate.value))
})

// 计算路径
// const mapDirPath = computed(() => `${props.projectPath}/map`)
// const statesDirPath = computed(() => `${props.projectPath}/history/states`)
// const countryColorsPath = computed(() => `${props.projectPath}/common/countries/colors.txt`)

const modes = [
  { id: 'province', name: '地块视图' },
  { id: 'state', name: '省份视图' },
  { id: 'country', name: '国家视图' },
  { id: 'terrain', name: '地形视图' }
] as const

type MapMode = typeof modes[number]['id']

const { 
  initMap, 
  renderTile,
  getPreview,
  getProvinceId,
  getOutline,
  getStateOutline,
  isLoading, 
  error, 
  mapData,
  definitions,
  states
} = useMapEngine()

// Refs
const canvasRef = ref<HTMLCanvasElement | null>(null)
const overlayCanvasRef = ref<HTMLCanvasElement | null>(null)
const minimapCanvasRef = ref<HTMLCanvasElement | null>(null)
const containerRef = ref<HTMLDivElement | null>(null)
const currentMode = ref<MapMode>('province')
const isPerformanceMode = ref(true) // 默认开启性能模式
const isHighlightEnabled = ref(true) // 默认开启高亮
const highlightMode = ref<'tile' | 'province'>('tile') // 高亮模式：地块或省份
const hoverProvinceId = ref<number | null>(null)
const hoverOutline = ref<Uint32Array | null>(null)
const outlineCache = new Map<string, Uint32Array>()
let hoverLookupTimer: number | null = null
let hoverLookupSequence = 0
let lastHoverLookupKey = ''

// 渲染缓存与分块 (LOD & LRU)
const TILE_SIZE = 512
const MAX_CACHE_SIZE = 50 // 限制最大缓存切片数，控制内存
const tileCache = new Map<string, ImageBitmap>()
const minimapCache = new Map<string, ImageBitmap>()
const tileUsage = new Map<string, number>() // LRU 追踪
let isRendering = false
let isUnmounted = false
let renderRafId: number | null = null
let resizeObserver: ResizeObserver | null = null

// 加载状态
const loadingStatus = ref('初始化中...')
const loadingDetail = ref('准备数据')
const loadingProgress = ref(0)
const loadingTimer = ref<number | null>(null)

// 视图变换状态
const scale = ref(1)
const translateX = ref(0)
const translateY = ref(0)
const isDragging = ref(false)
const dragStart = ref({ x: 0, y: 0, tx: 0, ty: 0 })
const mousePos = ref({ x: 0, y: 0 })
const mouseMapPos = ref({ x: 0, y: 0 })

// 导航器状态
const MINIMAP_SIZE = 200
const minimapScale = computed(() => {
  if (!mapData.value) return 1
  // 缩略图固定宽度为 MINIMAP_SIZE，所以缩放比例应基于宽度计算
  return MINIMAP_SIZE / mapData.value.width
})

const viewportRect = computed(() => {
  if (!mapData.value || !containerRef.value) return { x: 0, y: 0, w: 0, h: 0 }
  
  const rect = containerRef.value.getBoundingClientRect()
  const factor = minimapScale.value
  
  // LOD 动态下采样策略
  // 缩放 < 0.3 时使用 2x 下采样，< 0.1 时使用 4x 下采样 (如果不是强制 100% 模式)
  let effectiveDownsample = downsampleFactor.value
  if (isPerformanceMode.value) {
     if (scale.value < 0.1) effectiveDownsample = Math.max(effectiveDownsample, 4)
     else if (scale.value < 0.3) effectiveDownsample = Math.max(effectiveDownsample, 2)
  }

  // 计算当前可视区域在原始地图坐标系中的范围
  const x1 = -translateX.value / scale.value
  const y1 = -translateY.value / scale.value
  const w = rect.width / scale.value
  const h = rect.height / scale.value
  
  return {
    x: x1 * factor,
    y: y1 * factor,
    w: w * factor,
    h: h * factor
  }
})

// 修改样式：Canvas 容器现在填满视口，地图通过 Canvas 内部绘制偏移
const mapWrapperStyle = computed(() => {
  return {
    width: '100%',
    height: '100%',
    overflow: 'hidden'
  }
})

interface StateHoverInfo {
  id: number
  name: string
  owner: string
  cores: string[]
  claims: string[]
  isState: true
}

interface ProvinceHoverInfo extends ProvinceDefinition {
  stateName?: string
  owner?: string
  isState: false
}

type HoverInfo = StateHoverInfo | ProvinceHoverInfo

const definitionById = computed(() => {
  const map = new Map<number, ProvinceDefinition>()
  for (const definition of definitions.value) {
    map.set(definition.id, definition)
  }
  return map
})

const stateByProvinceId = computed(() => {
  const map = new Map<number, StateDefinition>()
  for (const state of states.value) {
    for (const provinceId of state.provinces) {
      map.set(provinceId, state)
    }
  }
  return map
})

const hoverInfo = computed<HoverInfo | null>(() => {
  if (!hoverProvinceId.value || !definitions.value) return null
  
  const def = definitionById.value.get(hoverProvinceId.value)
  if (!def) return null

  const state = stateByProvinceId.value.get(def.id)

  // 根据高亮模式（highlightMode）而非视图模式（currentMode）来决定预览框内容
  if (highlightMode.value === 'province') {
    if (!state) return null
    return {
      id: state.id,
      name: state.name,
      owner: state.owner,
      cores: state.cores || [],
      claims: state.claims || [],
      isState: true
    }
  }

  return {
    ...def,
    stateName: state?.name,
    owner: state?.owner,
    isState: false
  }
})

const tooltipStyle = computed(() => {
  // 确保预览框不会超出容器边界
  const rect = containerRef.value?.getBoundingClientRect()
  if (!rect) return { left: '0px', top: '0px' }

  let left = mousePos.value.x + 15
  let top = mousePos.value.y + 15

  // 预估宽度和高度（或在 DOM 更新后动态计算，这里先给一个安全边距）
  const estimatedWidth = 200
  const estimatedHeight = 150

  if (left + estimatedWidth > rect.width) {
    left = mousePos.value.x - estimatedWidth - 10
  }
  if (top + estimatedHeight > rect.height) {
    top = mousePos.value.y - estimatedHeight - 10
  }

  return {
    left: `${left}px`,
    top: `${top}px`
  }
})

// 初始化地图
onMounted(async () => {
  window.addEventListener('resize', handleResize)
  
  // 使用 ResizeObserver 监听容器大小变化，确保在 v-show 切换时能正确重绘
  if (containerRef.value) {
    resizeObserver = new ResizeObserver(() => {
      handleResize()
    })
    resizeObserver.observe(containerRef.value)
  }

  await refreshMap()
})

onUnmounted(() => {
  isUnmounted = true
  window.removeEventListener('resize', handleResize)
  
  if (resizeObserver) {
    resizeObserver.disconnect()
    resizeObserver = null
  }
  
  if (renderRafId !== null) {
    cancelAnimationFrame(renderRafId)
  }
  
  if (loadingTimer.value) {
    clearInterval(loadingTimer.value)
  }

  // 释放 ImageBitmap 资源
  if (hoverLookupTimer !== null) {
    clearTimeout(hoverLookupTimer)
    hoverLookupTimer = null
  }

  clearTileCache()
  clearMinimapCache()
  clearOutlineCache()
})

watch(hoverProvinceId, async (newId) => {
  if (newId && isHighlightEnabled.value) {
    try {
      let cacheKey: string | null = null
      let points: Uint32Array | undefined
      if (highlightMode.value === 'tile') {
        cacheKey = `tile:${newId}`
        points = outlineCache.get(cacheKey)
        if (!points) {
          points = await getOutline(newId)
        }
      } else {
        // 查找所属的州
        const state = stateByProvinceId.value.get(newId)
        if (state) {
          cacheKey = `state:${state.id}`
          points = outlineCache.get(cacheKey)
          if (!points) {
            points = await getStateOutline(state.id)
          }
        }
      }
      
      // 防止竞态条件
      if (cacheKey && points) {
        outlineCache.set(cacheKey, points)
      }

      if (hoverProvinceId.value === newId) {
        hoverOutline.value = points ?? null
        requestRender()
      }
    } catch (e) {
      console.error('Failed to get outline:', e)
    }
  } else {
    hoverOutline.value = null
    requestRender()
  }
})

watch(highlightMode, () => {
  // 模式切换时清除当前高亮并重新触发
  hoverOutline.value = null
  if (hoverProvinceId.value) {
    const id = hoverProvinceId.value
    hoverProvinceId.value = null
    nextTick(() => {
      hoverProvinceId.value = id
    })
  }
})

watch(isHighlightEnabled, (enabled) => {
  if (!enabled) {
    hoverOutline.value = null
  }
  requestRender()
})

function handleResize() {
  updateCanvasSize()
  requestRender()
}

function updateCanvasSize() {
  if (!canvasRef.value || !containerRef.value) return
  const rect = containerRef.value.getBoundingClientRect()
  canvasRef.value.width = rect.width
  canvasRef.value.height = rect.height
  
  if (overlayCanvasRef.value) {
    overlayCanvasRef.value.width = rect.width
    overlayCanvasRef.value.height = rect.height
  }
}

// 模拟进度更新
function updateProgress(status: string, detail: string, target: number) {
  loadingStatus.value = status
  loadingDetail.value = detail
  
  // 平滑进度条动画
  const step = () => {
    if (loadingProgress.value < target) {
      loadingProgress.value = Math.min(target, loadingProgress.value + 5)
      if (loadingProgress.value < target) {
         requestAnimationFrame(step)
      }
    }
  }
  requestAnimationFrame(step)
}

async function refreshMap() {
  await measureMapAsync('viewer.refreshMap', async () => {
    logMapEvent('viewer.refreshMap:start', { projectPath: props.projectPath })
  loadingProgress.value = 0
  updateProgress('加载地图资源', '读取 provinces.bmp...', 30)
  
  // 模拟耗时操作的进度
  const progressTimer = setInterval(() => {
    if (loadingProgress.value < 90) loadingProgress.value += 1
  }, 100)
  loadingTimer.value = progressTimer as any

  try {
    await measureMapAsync('viewer.refreshMap.initMap', async () => {
      await initMap(props.projectPath)
    })
    updateProgress('准备渲染', '初始化切片缓存...', 60)
    await measureMapAsync('viewer.refreshMap.resetMapCache', async () => {
      await resetMapCache()
    })
    updateProgress('构建导航器', '生成缩略图...', 90)
    void measureMapAsync(`viewer.refreshMap.drawMinimap(${currentMode.value})`, async () => {
      await drawMinimap()
    }).catch((error) => {
      console.error('Failed to draw minimap:', error)
    })
    updateProgress('就绪', '完成', 100)
    logMapEvent('viewer.refreshMap:done', {
      width: mapData.value?.width,
      height: mapData.value?.height,
      mode: currentMode.value
    })
  } catch (e) {
    loadingStatus.value = '加载失败'
    loadingDetail.value = String(e)
  } finally {
    clearInterval(progressTimer)
    loadingTimer.value = null
    // 延迟隐藏 loading
    setTimeout(() => {
      if (loadingProgress.value >= 100) isLoading.value = false
    }, 500)
  }
  })
}

// 切换模式
async function setMode(mode: MapMode) {
  await measureMapAsync(`viewer.setMode(${mode})`, async () => {
  isLoading.value = true
  loadingProgress.value = 0
  currentMode.value = mode
  updateProgress('切换视图', '清理缓存...', 50)
  requestRender()
  void drawMinimap()
  updateProgress('就绪', '完成', 100)
  setTimeout(() => { isLoading.value = false }, 300)
  logMapEvent('viewer.setMode:done', { mode })
  })
}

async function runHoverProvinceLookup() {
  if (!mapData.value || isDragging.value) return

  const { width, height } = mapData.value!
  const x = Math.floor((mousePos.value.x - translateX.value) / scale.value)
  const y = Math.floor((mousePos.value.y - translateY.value) / scale.value)

  mouseMapPos.value = { x, y }

  if (x < 0 || x >= width || y < 0 || y >= height) {
    lastHoverLookupKey = ''
    hoverLookupSequence += 1
    if (hoverProvinceId.value !== null) {
      hoverProvinceId.value = null
    }
    return
  }

  const lookupKey = `${x}:${y}`
  if (lookupKey === lastHoverLookupKey) return
  lastHoverLookupKey = lookupKey

  const requestId = ++hoverLookupSequence

  try {
    const id = await measureMapAsync('viewer.updateHoverProvince.getProvinceId', async () => (
      await getProvinceId(x, y)
    ))

    if (requestId !== hoverLookupSequence || lookupKey !== lastHoverLookupKey) {
      return
    }

    if (hoverProvinceId.value !== id) {
      hoverProvinceId.value = id
    }
  } catch (e) {
    if (requestId === hoverLookupSequence) {
      lastHoverLookupKey = ''
    }
    console.error(e)
  }
}

/**
 * 重置地图缓存
 */
async function resetMapCache() {
  // 清理旧缓存
  clearTileCache()
  clearMinimapCache()
  clearOutlineCache()
  lastHoverLookupKey = ''
  hoverLookupSequence += 1
  hoverProvinceId.value = null
  hoverOutline.value = null
  updateCanvasSize()
  requestRender()
}

/**
 * 请求渲染一帧
 */
function clearTileCache() {
  for (const bitmap of tileCache.values()) {
    bitmap.close()
  }
  tileCache.clear()
  tileUsage.clear()
}

function clearMinimapCache() {
  for (const bitmap of minimapCache.values()) {
    bitmap.close()
  }
  minimapCache.clear()
}

function clearOutlineCache() {
  outlineCache.clear()
}

function requestRender() {
  if (isRendering || isUnmounted) return
  isRendering = true
  renderRafId = requestAnimationFrame(() => {
    if (isUnmounted) return
    measureMapSync('viewer.requestRender.frame', () => {
      drawMap()
      drawOverlay()
    })
    isRendering = false
    renderRafId = null
  })
}

/**
 * 绘制 UI 覆盖层 (高亮等)
 */
function drawOverlay() {
  if (!overlayCanvasRef.value) return
  const canvas = overlayCanvasRef.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  ctx.clearRect(0, 0, canvas.width, canvas.height)

  if (!isHighlightEnabled.value) return

  if (hoverOutline.value && hoverOutline.value.length > 0) {
    const s = scale.value
    const tx = translateX.value
    const ty = translateY.value
    const pixelSize = Math.max(1, Math.ceil(s))
    
    ctx.fillStyle = 'rgba(255, 255, 0, 0.6)'
    
    // 使用 Path2D 批量绘制以提升性能，避免成千上万次的 fillRect 调用
    const path = new Path2D()
    const points = hoverOutline.value
    
    for (let i = 0; i < points.length; i++) {
      const val = points[i]
      const x = val & 0xFFFF
      const y = val >> 16
      
      const sx = x * s + tx
      const sy = y * s + ty
      
      // 视口剔除
      if (sx < -pixelSize || sy < -pixelSize || sx > canvas.width || sy > canvas.height) continue
      
      path.rect(sx, sy, pixelSize, pixelSize)
    }
    ctx.fill(path)
  }
}

// 绘制地图到 Canvas (分块按需渲染 - 解耦加载与绘制)
async function drawMap() {
  if (isUnmounted || !canvasRef.value || !mapData.value) return
  
  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d', { alpha: false })
  if (!ctx) return

  const factor = downsampleFactor.value
  const mapWidth = Math.floor(mapData.value.width / factor)
  const mapHeight = Math.floor(mapData.value.height / factor)
  const currentScale = scale.value * factor

  // 清除画布
  ctx.fillStyle = '#1a1a1a'
  ctx.fillRect(0, 0, canvas.width, canvas.height)

  // 计算视口可见的切片范围
  const viewLeft = -translateX.value / currentScale
  const viewTop = -translateY.value / currentScale
  const viewRight = (canvas.width - translateX.value) / currentScale
  const viewBottom = (canvas.height - translateY.value) / currentScale

  const startTileX = Math.floor(Math.max(0, viewLeft / TILE_SIZE))
  const startTileY = Math.floor(Math.max(0, viewTop / TILE_SIZE))
  const endTileX = Math.floor(Math.min(mapWidth / TILE_SIZE, viewRight / TILE_SIZE))
  const endTileY = Math.floor(Math.min(mapHeight / TILE_SIZE, viewBottom / TILE_SIZE))

  ctx.imageSmoothingEnabled = false // 保持像素锐利

  // 同步绘制已加载的切片，异步触发未加载的切片
  let missingTiles = []

  for (let ty = startTileY; ty <= endTileY; ty++) {
    for (let tx = startTileX; tx <= endTileX; tx++) {
      const key = getTileKey(tx, ty, factor)
      const tile = tileCache.get(key)
      
      const dx = translateX.value + tx * TILE_SIZE * currentScale
      const dy = translateY.value + ty * TILE_SIZE * currentScale
      // TILE_SIZE 本身是 512
      const dw = TILE_SIZE * currentScale
      const dh = TILE_SIZE * currentScale

      // 视锥体剔除
      if (dx + dw <= 0 || dy + dh <= 0 || dx >= canvas.width || dy >= canvas.height) continue;

      if (tile) {
        // 更新 LRU
        tileUsage.set(key, performance.now())
        ctx.drawImage(tile, dx, dy, dw, dh)
      } else {
        // 记录缺失的切片
        missingTiles.push({ tx, ty, factor })
        
        // 层级回退 (Fallback) 渲染
        // 尝试寻找上一级 (更低分辨率/更大范围) 的切片
        let drawnFallback = false
        const parentFactor = factor * 2
        
        // 只有当父级 factor 合理时才尝试
        // 假设最大下采样大概是 16 或 32
        if (parentFactor <= 32) {
          const pTx = Math.floor(tx / 2)
          const pTy = Math.floor(ty / 2)
          const pKey = getTileKey(pTx, pTy, parentFactor)
          
          const pTile = tileCache.get(pKey)
          if (pTile) {
             // 计算父级切片中的源区域
             // 父级切片 512x512 对应当前层级 2x2 个切片区域 (1024x1024 像素)
             // 当前切片相对于父级切片左上角的偏移 (0 或 1)
             const relX = tx % 2
             const relY = ty % 2
             
             // 源区域坐标 (256x256)
             const sx = relX * 256
             const sy = relY * 256
             const sw = 256
             const sh = 256
             
             ctx.drawImage(pTile, sx, sy, sw, sh, dx, dy, dw, dh)
             tileUsage.set(pKey, performance.now()) // 标记父级切片也在使用中
             drawnFallback = true
          }
        }
        
        if (!drawnFallback) {
          // 绘制网格占位符
          ctx.strokeStyle = '#333'
          ctx.strokeRect(dx, dy, dw, dh)
          // 显示加载中文字 (可选)
          // ctx.fillStyle = '#444'
          // ctx.font = '10px sans-serif'
          // ctx.fillText('Loading...', dx + 5, dy + 15)
        }
      }
    }
  }

  // 触发异步加载 (去重)
  if (missingTiles.length > 0) {
    // 按照距离视口中心的距离排序，优先加载中心区域
    const centerX = (startTileX + endTileX) / 2
    const centerY = (startTileY + endTileY) / 2
    missingTiles.sort((a, b) => {
      const distA = (a.tx - centerX) ** 2 + (a.ty - centerY) ** 2
      const distB = (b.tx - centerX) ** 2 + (b.ty - centerY) ** 2
      return distA - distB
    })
    
    loadTiles(missingTiles)
  }

  // 预加载周边切片 (Idle 时执行)
  if ('requestIdleCallback' in window) {
    (window as any).requestIdleCallback(() => {
       preloadTiles(startTileX, startTileY, endTileX, endTileY, factor)
    })
  }
}

// 正在加载的请求集合，防止重复请求
const pendingRequests = new Set<string>()

function getTileKey(tx: number, ty: number, zoom: number) {
  return `${tx}_${ty}_${zoom}_${currentMode.value}`
}

/**
 * 预加载周边切片
 */
function preloadTiles(startX: number, startY: number, endX: number, endY: number, factor: number) {
  if (!mapData.value) return
  
  const margin = 1
  const mapW = Math.ceil(mapData.value.width / factor)
  const mapH = Math.ceil(mapData.value.height / factor)
  const maxTx = Math.floor(mapW / TILE_SIZE)
  const maxTy = Math.floor(mapH / TILE_SIZE)
  
  const pStartX = Math.max(0, startX - margin)
  const pStartY = Math.max(0, startY - margin)
  const pEndX = Math.min(maxTx, endX + margin)
  const pEndY = Math.min(maxTy, endY + margin)
  
  const tilesToLoad: Array<{tx: number, ty: number, factor: number}> = []
  
  for (let y = pStartY; y <= pEndY; y++) {
    for (let x = pStartX; x <= pEndX; x++) {
       // 跳过视口内的切片 (已经在 drawMap 中处理)
       if (x >= startX && x <= endX && y >= startY && y <= endY) continue
       
       const key = getTileKey(x, y, factor)
       if (!tileCache.has(key) && !pendingRequests.has(key)) {
         tilesToLoad.push({ tx: x, ty: y, factor })
       }
    }
  }
  
  if (tilesToLoad.length > 0) {
    // 预加载优先级较低，可以不做排序
    loadTiles(tilesToLoad)
  }
}

// 批量异步加载切片
async function loadTiles(tiles: Array<{tx: number, ty: number, factor: number}>) {
  for (const { tx, ty, factor } of tiles) {
    const key = getTileKey(tx, ty, factor)
    if (pendingRequests.has(key)) continue
    
    pendingRequests.add(key)
    
    // 异步加载，不阻塞渲染循环
    fetchTile(tx, ty, factor).then((bitmap) => {
      pendingRequests.delete(key)
      if (bitmap) {
        // 只有当组件未卸载时才更新缓存
        if (!isUnmounted) {
          addTileToCache(key, bitmap)
          // 加载完成后触发一次重绘
          requestRender()
        } else {
          bitmap.close()
        }
      }
    }).catch(() => {
      pendingRequests.delete(key)
    })
  }
}

async function fetchTile(tx: number, ty: number, zoom: number): Promise<ImageBitmap | null> {
   try {
    const rgba = await measureMapAsync(`viewer.fetchTile(${currentMode.value})`, async () => (
      await renderTile(tx, ty, zoom, currentMode.value)
    ))
    if (!rgba || rgba.length === 0) return null
    const imageData = new ImageData(new Uint8ClampedArray(rgba), 512, 512)
    return await createImageBitmap(imageData)
  } catch (e) {
    console.error('Failed to load tile:', e)
    return null
  }
}

function addTileToCache(key: string, bitmap: ImageBitmap) {
  // LRU 淘汰逻辑
  if (tileCache.size >= MAX_CACHE_SIZE) {
    let oldestKey = ''
    let oldestTime = Infinity
    for (const [k, time] of tileUsage.entries()) {
       if (time < oldestTime) {
         oldestTime = time
         oldestKey = k
       }
    }
    // 确保不删除最近使用的 (防止当前视口的切片被删除)
    // 只有当最老的时间比当前时间早很多时才删除？
    // 或者简单地删除最老的，只要它不是刚刚被访问的
    if (oldestKey && tileCache.has(oldestKey)) {
      const oldBitmap = tileCache.get(oldestKey)
      if (oldBitmap) oldBitmap.close()
      tileCache.delete(oldestKey)
      tileUsage.delete(oldestKey)
    }
  }
  
  tileCache.set(key, bitmap)
  tileUsage.set(key, performance.now())
}

/**
 * 绘制导航缩略图
 */
async function drawMinimap() {
  if (!minimapCanvasRef.value || !mapData.value) return
  
  const canvas = minimapCanvasRef.value
  const displayWidth = MINIMAP_SIZE
  const displayHeight = Math.floor(mapData.value.height * (MINIMAP_SIZE / mapData.value.width))
  const cacheKey = `${props.projectPath}:${currentMode.value}:${displayWidth}:${displayHeight}`
  
  canvas.width = displayWidth
  canvas.height = displayHeight
  
  const ctx = canvas.getContext('2d', { alpha: false })
  if (!ctx) return

  const cachedBitmap = minimapCache.get(cacheKey)
  if (cachedBitmap) {
    ctx.drawImage(cachedBitmap, 0, 0)
    return
  }

  try {
    // 从后端获取预览图
    const rgba = await measureMapAsync(`viewer.drawMinimap(${currentMode.value})`, async () => (
      await getPreview(displayWidth, displayHeight, currentMode.value)
    ))
    if (!rgba) return
    
    const imageData = new ImageData(new Uint8ClampedArray(rgba), displayWidth, displayHeight)
    ctx.putImageData(imageData, 0, 0)
    const bitmap = await createImageBitmap(imageData)
    minimapCache.set(cacheKey, bitmap)
  } catch (e) {
    console.error('Failed to draw minimap:', e)
  }
}

// 交互逻辑
watch(downsampleFactor, () => {
  resetMapCache()
})

function handleWheel(e: WheelEvent) {
  e.preventDefault()
  const delta = e.deltaY > 0 ? 0.9 : 1.1
  const newScale = Math.max(0.1, Math.min(20, scale.value * delta))
  
  const rect = containerRef.value?.getBoundingClientRect()
  if (!rect) return

  const mouseX = e.clientX - rect.left
  const mouseY = e.clientY - rect.top
  
  // 以鼠标位置为中心缩放
  translateX.value = mouseX - (mouseX - translateX.value) * (newScale / scale.value)
  translateY.value = mouseY - (mouseY - translateY.value) * (newScale / scale.value)
  scale.value = newScale
  requestRender()
}

function handleMouseDown(e: MouseEvent) {
  if (e.button !== 0) return
  isDragging.value = true
  if (hoverLookupTimer !== null) {
    clearTimeout(hoverLookupTimer)
    hoverLookupTimer = null
  }
  dragStart.value = {
    x: e.clientX,
    y: e.clientY,
    tx: translateX.value,
    ty: translateY.value
  }
  if (containerRef.value) containerRef.value.style.cursor = 'grabbing'
}

function handleMouseMove(e: MouseEvent) {
  const rect = containerRef.value?.getBoundingClientRect()
  if (!rect) return

  mousePos.value = {
    x: e.clientX - rect.left,
    y: e.clientY - rect.top
  }

  if (isDragging.value) {
    const dx = e.clientX - dragStart.value.x
    const dy = e.clientY - dragStart.value.y
    translateX.value = dragStart.value.tx + dx
    translateY.value = dragStart.value.ty + dy
    requestRender()
  } else {
    scheduleHoverProvinceUpdate()
  }
}

function handleMouseUp() {
  isDragging.value = false
  if (containerRef.value) containerRef.value.style.cursor = 'crosshair'
}

function scheduleHoverProvinceUpdate() {
  if (hoverLookupTimer !== null) return

  hoverLookupTimer = window.setTimeout(() => {
    hoverLookupTimer = null
    void updateHoverProvince()
  }, 24)
}

async function updateHoverProvince() {
  await runHoverProvinceLookup()
  return

  if (!mapData.value) return

  const { width, height } = mapData.value!
  const x = Math.floor((mousePos.value.x - translateX.value) / scale.value)
  const y = Math.floor((mousePos.value.y - translateY.value) / scale.value)

  mouseMapPos.value = { x, y }

  if (x < 0 || x >= width || y < 0 || y >= height) {
    // 异步获取省份 ID
    try {
      const id = await measureMapAsync('viewer.updateHoverProvince.getProvinceId', async () => (
        await getProvinceId(x, y)
      ))
      hoverProvinceId.value = id
    } catch (e) {
      console.error(e)
    }
  } else {
    hoverProvinceId.value = null
  }
}

/**
 * 处理导航器点击/拖拽
 */
function handleMinimapClick(e: MouseEvent) {
  if (!mapData.value || !containerRef.value) return
  
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  const clickX = e.clientX - rect.left
  const clickY = e.clientY - rect.top
  
  const factor = minimapScale.value
  const targetMapX = clickX / factor
  const targetMapY = clickY / factor
  
  const containerRect = containerRef.value.getBoundingClientRect()
  
  // 将点击位置设为视图中心
  translateX.value = containerRect.width / 2 - targetMapX * scale.value
  translateY.value = containerRect.height / 2 - targetMapY * scale.value
  requestRender()
}

watch(() => props.projectPath, refreshMap)
</script>

<style scoped>
canvas {
  image-rendering: pixelated;
}
</style>
