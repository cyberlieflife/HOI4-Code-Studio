<template>
  <div 
    class="gui-node absolute"
    :style="nodeStyle"
    :title="node.properties.name"
  >
    <!-- 背景/Sprite 渲染 (底层) -->
    <div 
      v-if="spriteUrl" 
      :style="spriteStyle" 
      class="absolute inset-0 z-0 pointer-events-none"
    ></div>

    <!-- 文本渲染 (仅限 TextBox 或 Button) (中层) -->
    <div 
      v-if="node.node_type === 'instant_text_box' || (node.node_type === 'button' && node.properties.text)"
      class="absolute pointer-events-none z-10 flex"
      :style="textStyle"
      v-html="formattedText"
    >
    </div>

    <!-- 子节点容器 (高层) -->
    <!-- 这里的 overflow 由 nodeStyle 中的 clipping 属性决定 -->
    <div 
      v-if="node.children && node.children.length > 0" 
      class="absolute inset-0 z-20 pointer-events-none"
      :style="containerStyle"
    >
      <!-- 如果是 GridBox，我们需要特殊布局其子项 -->
      <template v-if="node.node_type === 'grid_box'">
        <div 
          v-for="(child, idx) in node.children" 
          :key="`${child.properties?.name || child.node_type}-${idx}`"
          class="absolute pointer-events-none"
          :style="getGridItemStyle(idx)"
        >
          <GuiNodeRenderer 
            :node="child"
            :parent-size="node.properties.slotsize || { width: 100, height: 100 }"
            :sprites="sprites"
            :project-path="projectPath"
            :game-directory="gameDirectory"
            :dependency-roots="dependencyRoots"
          />
        </div>
      </template>
      
      <template v-else>
        <GuiNodeRenderer 
          v-for="(child, idx) in node.children" 
          :key="`${child.properties?.name || child.node_type}-${idx}`" 
          :node="child"
          :parent-size="effectiveSize"
          :sprites="sprites"
          :project-path="projectPath"
          :game-directory="gameDirectory"
          :dependency-roots="dependencyRoots"
        />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, watch, type CSSProperties } from 'vue'
import { type GuiNode, resolveGuiResource, readImageAsBase64 } from '../../api/tauri'

const props = defineProps<{
  node: GuiNode
  parentSize?: { width: number; height: number }
  sprites: Record<string, any>
  projectPath?: string
  gameDirectory?: string
  dependencyRoots?: string[]
}>()

const spriteUrl = ref<string | null>(null)
const spriteMeta = ref<{ 
  noOfFrames: number,
  borderSize?: { x: number, y: number }
} | null>(null)
const imageDimensions = ref<{ width: number, height: number } | null>(null)

const spriteName = computed(() => (
  props.node.properties.sprite_type ||
  props.node.properties.quad_texture_sprite ||
  props.node.properties.background ||
  ''
))

/**
 * 格式化 HOI4 文本，处理 §Y 等颜色代码和 \n
 */
const formattedText = computed(() => {
  const text = props.node.properties.text || ''
  if (!text) return ''

  // HTML 转义，防止 XSS 注入
  const escapeHtml = (s: string) =>
    s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')

  // 先整体转义，再处理 HOI4 格式代码（§ 和 \n 不受转义影响）
  let result = escapeHtml(text)

  // 处理换行符
  result = result.replace(/\\n/g, '<br/>')

  // HOI4 颜色代码映射
  const colorMap: Record<string, string> = {
    'Y': '#ffff00', 'R': '#ff0000', 'G': '#00ff00', 'B': '#0000ff',
    'W': '#ffffff', 'H': '#fe8a08', 'L': '#333333', 'P': '#ffc0cb',
    'C': '#00ffff', 'M': '#ff00ff', 'T': '#ffffff', 'g': '#a9a9a9',
  }

  // 解析 §X...§! 颜色代码
  const colorRegex = /§([YRGWBHLPCMTg])([^§]+)(?:§!)?/g
  result = result.replace(colorRegex, (_, color, content) => {
    const hex = colorMap[color] || '#ffffff'
    return `<span style="color: ${hex}">${content}</span>`
  })

  // 清除剩余的颜色代码
  result = result.replace(/§[YRGWBHLPCMTg!]/g, '')

  return result
})

const currentNoOfFrames = computed(() => {
  const spriteDef = spriteName.value ? props.sprites[spriteName.value] : null
  return spriteMeta.value?.noOfFrames || spriteDef?.noOfFrames || 1
})

const layoutReady = computed(() => {
  const p = props.node.properties
  if (!props.node.children || props.node.children.length === 0) return true
  if (p.size?.width && p.size?.height) return true
  if (p.max_width && p.max_height) return true
  if (!spriteName.value) return true
  return imageDimensions.value !== null
})

// 监听图片加载以获取真实尺寸（同时监听帧数，避免竞态导致尺寸计算错误）
const imageLoadToken = ref(0)
watch([spriteUrl, currentNoOfFrames], ([url, frames]) => {
  imageLoadToken.value += 1
  const token = imageLoadToken.value

  if (url) {
    const img = new Image()
    img.onload = () => {
      if (token !== imageLoadToken.value) return
      const f = typeof frames === 'number' && frames > 0 ? frames : 1
      imageDimensions.value = {
        width: img.width / f,
        height: img.height,
      }
    }
    img.src = url
  } else {
    imageDimensions.value = null
  }
})

// 辅助函数：解析 Orientation/Origo
function getAnchorPoint(type: string | undefined): { x: string, y: string } {
  if (!type) return { x: '0%', y: '0%' }
  // 统一处理：转大写，去除空格，兼容下划线
  const t = type.toUpperCase().replace(/\s+/g, '_')
  switch (t) {
    case 'UPPER_LEFT':
    case 'UP_LEFT':
    case 'TOP_LEFT': return { x: '0%', y: '0%' }
    
    case 'UPPER_RIGHT':
    case 'UP_RIGHT':
    case 'TOP_RIGHT': return { x: '100%', y: '0%' }
    
    case 'LOWER_LEFT':
    case 'BOTTOM_LEFT': return { x: '0%', y: '100%' }
    
    case 'LOWER_RIGHT':
    case 'BOTTOM_RIGHT': return { x: '100%', y: '100%' }
    
    case 'CENTER':
    case 'CENTRE': return { x: '50%', y: '50%' }
    
    case 'CENTER_UP':
    case 'CENTER_TOP':
    case 'CENTRE_UP':
    case 'CENTRE_TOP': return { x: '50%', y: '0%' }
    
    case 'CENTER_DOWN':
    case 'CENTER_BOTTOM':
    case 'CENTRE_DOWN':
    case 'CENTRE_BOTTOM': return { x: '50%', y: '100%' }
    
    case 'CENTER_LEFT':
    case 'CENTRE_LEFT': return { x: '0%', y: '50%' }
    
    case 'CENTER_RIGHT':
    case 'CENTRE_RIGHT': return { x: '100%', y: '50%' }
    
    default: return { x: '0%', y: '0%' }
  }
}

function anchorToRatio(type: string | undefined): { x: number; y: number } {
  const a = getAnchorPoint(type)
  const x = parseFloat(a.x) / 100
  const y = parseFloat(a.y) / 100
  return {
    x: Number.isFinite(x) ? x : 0,
    y: Number.isFinite(y) ? y : 0,
  }
}

/**
 * 计算用于子节点定位的有效参考尺寸。
 */
const effectiveSize = computed((): { width: number; height: number } => {
  const p = props.node.properties
  const parentW = props.parentSize?.width ?? 0
  const parentH = props.parentSize?.height ?? 0

  // 1. 优先使用显式定义的 size
  if (p.size && (p.size.width > 0 || p.size.height > 0)) {
    return { width: p.size.width, height: p.size.height }
  }

  // 2. 其次使用图片尺寸 (仅限有图片的类型)
  if (imageDimensions.value && (props.node.node_type === 'icon' || props.node.node_type === 'button')) {
    return imageDimensions.value
  }

  // 3. 再次使用 max_width/max_height
  if (p.max_width || p.max_height) {
    return {
      width: p.max_width || 0,
      height: p.max_height || 0
    }
  }

  // 4. 文本框特殊处理：优先使用 size -> max_width -> 默认值
  if (props.node.node_type === 'instant_text_box') {
    const w = p.size?.width || p.max_width || 200 // 文本框如果没有 size 也没有 max_width，通常有一个默认合理宽度
    const h = p.size?.height || p.max_height || 24
    return { width: w, height: h }
  }

  // 5. 容器兜底：如果是容器且没有定义尺寸，则它通常在逻辑上继承父容器的尺寸
  const isContainer = 
    props.node.node_type === 'container_window' || 
    props.node.node_type === 'window_type' || 
    props.node.node_type === 'window'

  if (isContainer) {
    return { width: parentW, height: parentH }
  }

  return { width: 0, height: 0 }
})

// 基础样式计算
const nodeStyle = computed(() => {
  const p = props.node.properties
  const orientation = anchorToRatio(p.orientation)
  const origo = anchorToRatio(p.origo)
  
  const parentW = props.parentSize?.width ?? 0
  const parentH = props.parentSize?.height ?? 0
  const size = effectiveSize.value

  let displayW = size.width
  let displayH = size.height

  const isContainer = 
    props.node.node_type === 'container_window' || 
    props.node.node_type === 'window_type' || 
    props.node.node_type === 'window'

  const style: any = {
    position: 'absolute',
    width: displayW > 0 ? `${displayW}px` : 'auto',
    height: displayH > 0 ? `${displayH}px` : 'auto',
    transformOrigin: '0 0',
    pointerEvents: 'auto',
    zIndex: props.node.node_type === 'window' ? 100 : 1,
  }

  // HOI4 默认不剪切，只有明确设置 clipping = yes 时才剪切
  // 某些容器可能默认剪切（如带有滚动条的容器），但目前我们优先遵循显式属性
  if (p.clipping === true) {
    style.overflow = 'hidden'
  } else {
    style.overflow = 'visible'
  }

  if (!layoutReady.value) {
    style.visibility = 'hidden'
  }

  const posX = p.position?.x ?? 0
  const posY = p.position?.y ?? 0

  // 核心定位公式：ParentAnchor + Offset - SelfOrigo
  const left = parentW * orientation.x + posX
  const top = parentH * orientation.y + posY

  style.left = `${left}px`
  style.top = `${top}px`

  // Origo 偏移
  const actualW = size.width
  const actualH = size.height
  const origoPxX = actualW * origo.x
  const origoPxY = actualH * origo.y

  const scale = p.scale && p.scale !== 1 ? p.scale : 1
  style.transform = `translate(${-origoPxX}px, ${-origoPxY}px) scale(${scale})`

  // 调试辅助
  if (!spriteUrl.value && isContainer) {
    style.backgroundColor = 'rgba(120, 120, 120, 0.03)'
    style.outline = '1px dashed rgba(255, 255, 255, 0.05)'
  }

  return style
})

// 子容器样式 (处理内部剪切)
const containerStyle = computed(() => {
  const p = props.node.properties
  const style: any = {}
  
  // 容器内部的子节点容器也需要遵循 clipping
  // 默认不剪切，只有明确设置 clipping = yes 时才剪切
  if (p.clipping === true) {
    style.overflow = 'hidden'
  } else {
    style.overflow = 'visible'
  }
  
  return style
})

/**
 * 为 GridBox 的模拟项计算位置
 */
function getGridItemStyle(index: number): CSSProperties {
  const p = props.node.properties
  const slotW = p.slotsize?.width || 100
  const slotH = p.slotsize?.height || 100
  const addHorizontal = p.add_horizontal !== false // 默认为 true
  const maxSlotsH = p.max_slots_horizontal || 999
  
  let col = 0
  let row = 0
  
  if (addHorizontal) {
    col = index % maxSlotsH
    row = Math.floor(index / maxSlotsH)
  } else {
    // 垂直优先布局
    const maxSlotsV = p.max_slots_vertical || 999
    row = index % maxSlotsV
    col = Math.floor(index / maxSlotsV)
  }
  
  return {
    left: `${col * slotW}px`,
    top: `${row * slotH}px`,
    width: `${slotW}px`,
    height: `${slotH}px`
  }
}

// Sprite 样式计算
const spriteStyle = computed((): CSSProperties => {
  const p = props.node.properties
  const style: CSSProperties = {
    backgroundImage: spriteUrl.value ? `url(${spriteUrl.value})` : 'none',
    backgroundRepeat: 'no-repeat',
  }

  const frame = p.frame || 1
  const frameCount = currentNoOfFrames.value

  // 处理 9 宫格渲染 (corneredTileSpriteType)
  if (spriteMeta.value?.borderSize) {
    const border = spriteMeta.value.borderSize
    style.borderImageSource = spriteUrl.value ? `url(${spriteUrl.value})` : 'none'
    style.borderImageSlice = `${border.y} ${border.x}`
    style.borderImageWidth = `${border.y}px ${border.x}px`
    style.borderImageRepeat = 'stretch'
    style.backgroundSize = '0 0' // 隐藏原始背景图
  } else if (frameCount > 1) {
    // 处理多帧序列图
    style.backgroundSize = `${frameCount * 100}% 100%`
    // 这里的百分比定位公式：(frame - 1) / (frameCount - 1) * 100%
    const positionX = frameCount > 1 ? ((frame - 1) / (frameCount - 1)) * 100 : 0
    style.backgroundPosition = `${positionX}% 0%`
  } else {
    style.backgroundSize = '100% 100%'
    style.backgroundPosition = '0 0'
  }

  return style
})

// 文本样式
const textStyle = computed((): CSSProperties => {
  const p = props.node.properties
  const format = p.format || 'left'
  const vAlign = p.vertical_alignment || 'top'

  const style: CSSProperties = {
    position: 'absolute',
    left: '0',
    top: '0',
    width: '100%',
    height: '100%',
    display: 'flex',
    flexDirection: 'column',
    padding: '0px',
    fontSize: '14px',
    color: '#ffffff',
    textShadow: '1px 1px 1px rgba(0,0,0,0.8)',
    lineHeight: '1.2',
    wordBreak: 'break-word',
    boxSizing: 'border-box',
    overflow: 'visible',
    pointerEvents: 'none',
  }

  // 3. 字体处理
  if (p.font) {
    if (p.font.includes('24')) (style as any).fontSize = '24px'
    else if (p.font.includes('18')) (style as any).fontSize = '18px'
    else if (p.font.includes('36')) (style as any).fontSize = '36px'
    else (style as any).fontFamily = p.font
  }

  // 4. 文本对齐控制
  // 注意：在 Flex Column 模式下，justifyContent 控制垂直，alignItems 控制水平
  if (format === 'center') {
    style.alignItems = 'center'
    style.textAlign = 'center'
  } else if (format === 'right') {
    style.alignItems = 'flex-end'
    style.textAlign = 'right'
  } else {
    style.alignItems = 'flex-start'
    style.textAlign = 'left'
  }

  if (vAlign === 'center') {
    style.justifyContent = 'center'
  } else if (vAlign === 'bottom' || vAlign === 'down') {
    style.justifyContent = 'flex-end'
  } else {
    style.justifyContent = 'flex-start'
  }

  // 5. Max Width 处理
  // 在 HOI4 中，maxWidth 限制文本换行宽度。
  // 如果 format 是 center，文本块本身应该在容器内居中。
  if (p.max_width) {
    style.maxWidth = `${p.max_width}px`
    // 如果设置了 maxWidth，我们通常希望宽度是自适应的（针对 align-items 居中）
    style.width = 'auto'
    
    // 关键：绝对定位下的居中
    if (format === 'center') {
      style.left = '50%'
      style.transform = 'translateX(-50%)'
    } else if (format === 'right') {
      style.left = 'auto'
      style.right = '0'
    }
  }

  // 6. Fixed Size 处理
  if (p.fixedsize) {
    style.whiteSpace = 'nowrap'
    style.textOverflow = 'ellipsis'
    style.overflow = 'hidden'
  }

  return style
})

// 加载资源
const loadResource = async () => {
  if (!spriteName.value) {
    spriteUrl.value = null
    spriteMeta.value = null
    return
  }

  try {
    const res = await resolveGuiResource(
      spriteName.value,
      props.projectPath || '',
      props.gameDirectory || '',
      props.dependencyRoots || []
    )

    if (res && res.success && res.path) {
      // 使用 readImageAsBase64 处理 DDS/TGA 转 PNG
      const imgRes = await readImageAsBase64(res.path)
      if (imgRes.success && imgRes.base64) {
        // 先设置元信息，再设置 url，避免图片 onload 早于 spriteMeta 更新导致宽度按 1 帧计算
        spriteMeta.value = { 
          noOfFrames: res.noOfFrames ?? 1,
          borderSize: res.borderSize
        }
        spriteUrl.value = `data:${imgRes.mimeType || 'image/png'};base64,${imgRes.base64}`
      }
    }
  } catch (err) {
    console.warn(`Failed to resolve sprite: ${spriteName.value}`, err)
  }
}

function resetSpriteState() {
  spriteUrl.value = null
  spriteMeta.value = null
  imageDimensions.value = null
}

// 只监听“会影响资源解析/图片尺寸”的关键依赖，避免 deep watch 带来的频繁抖动
watch(
  [
    () => spriteName.value,
    () => props.projectPath,
    () => props.gameDirectory,
    () => (props.dependencyRoots || []).join('|'),
  ],
  () => {
    resetSpriteState()
    loadResource()
  },
  { immediate: false }
)

onMounted(loadResource)
</script>
