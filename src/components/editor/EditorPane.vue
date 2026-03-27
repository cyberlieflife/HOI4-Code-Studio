<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import EditorTabs from './EditorTabs.vue'
import CodeMirrorEditor from './CodeMirrorEditor.vue'
import ContextMenu from './ContextMenu.vue'
import EventGraphViewer from './EventGraphViewer.vue'
import FocusTreeViewer from './FocusTreeViewer.vue'
import ImagePreviewer from './ImagePreviewer.vue'
import WorldMapViewer from './WorldMapViewer.vue'
import GuiViewer from './GuiViewer.vue'
import MioViewer from './MioViewer.vue'
import GfxViewer from './GfxViewer.vue'
import type { EditorPane } from '../../composables/useEditorGroups'
import { useSyntaxHighlight } from '../../composables/useSyntaxHighlight'
import { collectErrors } from '../../utils/ErrorTip'
import { EditorView } from '@codemirror/view'

const props = defineProps<{
  pane: EditorPane
  isActive: boolean
  projectPath: string
  gameDirectory: string
  dependencyRoots?: string[]
  disableErrorHandling?: boolean
}>()

const emit = defineEmits<{
  switchFile: [paneId: string, index: number]
  closeFile: [paneId: string, index: number]
  contextMenu: [event: MouseEvent, paneId: string, index: number]
  contentChange: [paneId: string, content: string]
  externalFileUpdate: [paneId: string, filePath: string, content: string]
  cursorChange: [paneId: string, line: number, column: number]
  saveFile: [paneId: string]
  activate: [paneId: string]
  splitPane: [paneId: string, fileIndex?: number]
  errorsChange: [paneId: string, errors: Array<{line: number, msg: string, type: string}>]
  editorContextMenuAction: [action: string, paneId: string]
  previewEvent: [paneId: string]
  previewFocus: [paneId: string]
  previewMap: [paneId: string]
  previewGui: [paneId: string]
  previewMio: [paneId: string]
  previewGfx: [paneId: string]
  jumpToFocusFromPreview: [sourcePaneId: string, sourceFilePath: string, focusId: string, line: number]
  jumpToMioFromPreview: [sourcePaneId: string, sourceFilePath: string, traitId: string, line: number]
  jumpToGfxFromPreview: [sourcePaneId: string, sourceFilePath: string, line: number]
}>()

function handleExternalFileUpdate(filePath: string, content: string) {
  emit('externalFileUpdate', props.pane.id, filePath, content)
}

function handleJumpToGfx(line: number) {
  if (isCurrentFileGfx.value) {
    const sourcePath = currentFile.value?.sourceFilePath || currentFile.value?.node.path
    if (!sourcePath) return
    emit('jumpToGfxFromPreview', props.pane.id, sourcePath, line)
    return
  }

  jumpToLine(line)
}

function handleJumpToMio(traitId: string, line: number) {
  console.log('Jump to mio trait:', traitId, 'at line:', line)

  if (isCurrentFileMio.value) {
    const sourcePath = currentFile.value?.sourceFilePath || currentFile.value?.node.path
    if (!sourcePath) return
    emit('jumpToMioFromPreview', props.pane.id, sourcePath, traitId, line)
    return
  }

  jumpToLine(line)
}

const editorRef = ref<InstanceType<typeof CodeMirrorEditor> | null>(null)
const fileContent = ref('')
const currentLine = ref(1)
const currentColumn = ref(1)
const hasUnsavedChanges = ref(false)
const txtErrors = ref<{line: number, msg: string, type: string}[]>([])
const isLoadingFile = ref(false)  // 标记是否正在加载文件

// 编辑器右键菜单状态
const editorContextMenuVisible = ref(false)
const editorContextMenuX = ref(0)
const editorContextMenuY = ref(0)

const { highlightCode, getLanguage } = useSyntaxHighlight()

// 当前活动文件
const currentFile = computed(() => {
  if (props.pane.activeFileIndex >= 0 && props.pane.openFiles[props.pane.activeFileIndex]) {
    return props.pane.openFiles[props.pane.activeFileIndex]
  }
  return null
})

// 当前文件是否为图片
const isCurrentFileImage = computed(() => {
  return currentFile.value?.isImage === true
})

// 当前文件是否为事件关系图
const isCurrentFileEventGraph = computed(() => {
  return currentFile.value?.isEventGraph === true
})

// 当前文件是否为事件文件（路径包含 /events/）
const isEventFile = computed(() => {
  if (!currentFile.value?.node.path) return false
  const normalizedPath = currentFile.value.node.path.replace(/\\/g, '/')
  return normalizedPath.includes('/events/') && !isCurrentFileImage.value && !isCurrentFileEventGraph.value
})

// 当前文件是否为国策树预览
const isCurrentFileFocusTree = computed(() => {
  return currentFile.value?.isFocusTree === true
})

// 当前文件是否为国策文件（路径包含 /common/national_focus/）
const isFocusFile = computed(() => {
  if (!currentFile.value?.node.path) return false
  const normalizedPath = currentFile.value.node.path.replace(/\\/g, '/')
  return (normalizedPath.includes('/common/national_focus/') || 
          normalizedPath.includes('\\common\\national_focus\\')) &&
         !isCurrentFileImage.value && 
         !isCurrentFileEventGraph.value &&
         !isCurrentFileFocusTree.value
})

 // 当前文件是否为地图预览
 const isCurrentFileWorldMap = computed(() => {
   return currentFile.value?.isWorldMap === true
 })

 // 当前地图预览的覆盖模式（fallback=右键预览, project-only=编辑器内置）
 const currentMergeMode = computed(() => {
   // 如果 OpenFile 中显式设置了 mergeMode 则使用，否则默认 fallback
   return currentFile.value?.mergeMode || 'fallback'
 })

// 追踪地图是否曾被加载过，用于延迟加载并保持状态
const hasMapLoaded = ref(false)
watch(isCurrentFileWorldMap, (val) => {
  if (val) hasMapLoaded.value = true
}, { immediate: true })

// 当前文件是否为地图定义文件 (map/default.map)
const isMapFile = computed(() => {
  if (!currentFile.value?.node.path) return false
  const normalizedPath = currentFile.value.node.path.replace(/\\/g, '/')
  return normalizedPath.endsWith('/map/default.map') && 
         !isCurrentFileImage.value && 
         !isCurrentFileEventGraph.value &&
         !isCurrentFileFocusTree.value &&
         !isCurrentFileWorldMap.value
})

// 当前文件是否为 GUI 预览
const isCurrentFileGui = computed(() => {
  return currentFile.value?.isGuiPreview === true
})

// 当前文件是否为 MIO 预览
const isCurrentFileMio = computed(() => {
  return currentFile.value?.isMioPreview === true
})

// 当前文件是否为 GFX 预览
const isCurrentFileGfx = computed(() => {
  return currentFile.value?.isGfxPreview === true
})

// 当前文件是否为 MIO 文件
const isMioFile = computed(() => {
  if (!currentFile.value?.node.path) return false
  const normalizedPath = currentFile.value.node.path.replace(/\\/g, '/')
  return normalizedPath.includes('/common/military_industrial_organization/organizations/') &&
         normalizedPath.toLowerCase().endsWith('.txt') &&
         !isCurrentFileImage.value &&
         !isCurrentFileEventGraph.value &&
         !isCurrentFileFocusTree.value &&
         !isCurrentFileWorldMap.value &&
         !isCurrentFileGui.value &&
         !isCurrentFileMio.value
})

// 当前文件是否为 GUI 文件
const isGuiFile = computed(() => {
  if (!currentFile.value?.node.path) return false
  const name = currentFile.value.node.name.toLowerCase()
  // 支持 .gui 文件，排除预览窗口本身
  return (name.endsWith('.gui') || currentFile.value.node.path.toLowerCase().endsWith('.gui')) && 
         !isCurrentFileImage.value && 
         !isCurrentFileGui.value
})

// 当前文件是否为 GFX 文件
const isGfxFile = computed(() => {
  if (!currentFile.value?.node.path) return false
  const name = currentFile.value.node.name.toLowerCase()
  return (name.endsWith('.gfx') || currentFile.value.node.path.toLowerCase().endsWith('.gfx')) &&
         !isCurrentFileImage.value &&
         !isCurrentFileEventGraph.value &&
         !isCurrentFileFocusTree.value &&
         !isCurrentFileWorldMap.value &&
         !isCurrentFileGui.value &&
         !isCurrentFileMio.value &&
         !isCurrentFileGfx.value
})

// 图片 URL (使用 base64 数据)
const imageUrl = computed(() => {
  if (isCurrentFileImage.value && currentFile.value && currentFile.value.content) {
    // content 存储的是 base64 字符串，需要添加 data URI 前缀
    const ext = currentFile.value.node.name.split('.').pop()?.toLowerCase() || ''
    let mimeType = 'image/png'
    switch (ext) {
      case 'jpg':
      case 'jpeg':
        mimeType = 'image/jpeg'
        break
      case 'gif':
        mimeType = 'image/gif'
        break
      case 'bmp':
        mimeType = 'image/bmp'
        break
      case 'webp':
        mimeType = 'image/webp'
        break
      case 'tga':
        mimeType = 'image/x-tga'
        break
    }
    return `data:${mimeType};base64,${currentFile.value.content}`
  }
  return ''
})

// 监听活动文件变化
watch(currentFile, (file, oldFile) => {
  if (file) {
    // 标记正在加载文件，避免触发 hasUnsavedChanges
    isLoadingFile.value = true
    
    // 只有当文件路径变化或内容真正不同时才更新内容，避免不必要的光标重置
    if (!oldFile || oldFile.node.path !== file.node.path || oldFile.content !== file.content) {
      fileContent.value = file.content
    }
    
    hasUnsavedChanges.value = file.hasUnsavedChanges
    currentLine.value = file.cursorLine
    currentColumn.value = file.cursorColumn
    
    nextTick(() => {
      if (file.node) {
          const language = getLanguage(file.node.name)
          if (language === 'hoi4') {
            txtErrors.value = collectErrors(fileContent.value, { 
              filePath: file.node.path, 
              projectRoot: props.projectPath, 
              gameDirectory: props.gameDirectory,
              disableErrorHandling: props.disableErrorHandling
            })
          } else {
            txtErrors.value = []
          }
          highlightCode(fileContent.value, file.node.name, txtErrors.value)
          // 触发错误变化事件
          emit('errorsChange', props.pane.id, txtErrors.value)
        }
      // 文件加载完成，可以开始追踪修改
      isLoadingFile.value = false
    })
  } else {
    isLoadingFile.value = true
    fileContent.value = ''
    hasUnsavedChanges.value = false
    txtErrors.value = []
    // 触发错误变化事件（清空错误）
    emit('errorsChange', props.pane.id, [])
    nextTick(() => {
      isLoadingFile.value = false
    })
  }
}, { immediate: true })

// 监听当前文件内容变化（用于预览文件同步更新）
watch(() => currentFile.value?.content, (newContent, oldContent) => {
  if (currentFile.value && newContent !== oldContent && newContent !== fileContent.value) {
    console.log(`[EditorPane] 检测到文件内容变化，更新内容: ${currentFile.value.node.name}`)
    if (newContent !== undefined) {
      fileContent.value = newContent
    }
    hasUnsavedChanges.value = currentFile.value.hasUnsavedChanges
    
    // 如果是预览文件，重新高亮和解析错误
    if (currentFile.value.node) {
      const language = getLanguage(currentFile.value.node.name)
      if (language === 'hoi4') {
        txtErrors.value = collectErrors(fileContent.value, { 
          filePath: currentFile.value.node.path, 
          projectRoot: props.projectPath, 
          gameDirectory: props.gameDirectory,
          disableErrorHandling: props.disableErrorHandling
        })
      }
      highlightCode(fileContent.value, currentFile.value.node.name, txtErrors.value)
      // 触发错误变化事件
      emit('errorsChange', props.pane.id, txtErrors.value)
    }
  }
})

watch(() => currentFile.value?.hasUnsavedChanges, (nextHasUnsavedChanges) => {
  hasUnsavedChanges.value = nextHasUnsavedChanges ?? false
})

function handleSwitchFile(index: number) {
  emit('switchFile', props.pane.id, index)
}

function handleCloseFile(index: number) {
  emit('closeFile', props.pane.id, index)
}

function handleContextMenu(event: MouseEvent, index: number) {
  emit('contextMenu', event, props.pane.id, index)
}

function handleContentChange(content: string) {
  fileContent.value = content
  // 只有在不是加载文件时才标记为已修改和触发事件
  if (!isLoadingFile.value) {
    hasUnsavedChanges.value = true
    emit('contentChange', props.pane.id, content)
  }
  
  if (currentFile.value?.node) {
    const language = getLanguage(currentFile.value.node.name)
    if (language === 'hoi4') {
      txtErrors.value = collectErrors(content, { 
        filePath: currentFile.value.node.path, 
        projectRoot: props.projectPath, 
        gameDirectory: props.gameDirectory,
        disableErrorHandling: props.disableErrorHandling
      })
      // 触发错误变化事件
      emit('errorsChange', props.pane.id, txtErrors.value)
    }
    highlightCode(content, currentFile.value.node.name, txtErrors.value)
  }
}

function handleCursorChange(line: number, column: number) {
  currentLine.value = line
  currentColumn.value = column
  emit('cursorChange', props.pane.id, line, column)
}

function handleSaveFile() {
  emit('saveFile', props.pane.id)
}

function handleActivate() {
  emit('activate', props.pane.id)
}

function handleSplitPane() {
  emit('splitPane', props.pane.id, props.pane.activeFileIndex)
}

function handlePreviewEvent() {
  emit('previewEvent', props.pane.id)
}

function handlePreviewFocus() {
  emit('previewFocus', props.pane.id)
}

function handlePreviewMap() {
  emit('previewMap', props.pane.id)
}

function handlePreviewGui() {
  emit('previewGui', props.pane.id)
}

function handlePreviewMio() {
  emit('previewMio', props.pane.id)
}

function handlePreviewGfx() {
  emit('previewGfx', props.pane.id)
}

function handleJumpToEvent(eventId: string, line: number) {
  console.log('Jump to event:', eventId, 'at line:', line)
  // 这里可以跳转到源文件的对应位置
  jumpToLine(line)
}

function handleJumpToFocus(focusId: string, line: number) {
  console.log('Jump to focus:', focusId, 'at line:', line)

  // 如果当前是国策树预览页，则需要跳回源文件
  if (isCurrentFileFocusTree.value) {
    const sourcePath = currentFile.value?.sourceFilePath || currentFile.value?.node.path
    if (!sourcePath) return
    emit('jumpToFocusFromPreview', props.pane.id, sourcePath, focusId, line)
    return
  }

  jumpToLine(line)
}

/**
 * 跳转到指定行并高亮显示
 */
function jumpToLine(line: number) {
  console.log('[EditorPane] jumpToLine called with line:', line)
  
  // 检查行号有效性
  if (!line || line < 1) {
    console.warn('[EditorPane] Invalid line number:', line)
    return
  }
  
  // 通过 getEditorView() 方法获取 editorView
  const view = (editorRef.value as any)?.getEditorView?.()
  if (!view) {
    console.warn('[EditorPane] Editor view not available, retrying in 100ms...')
    // 重试一次，以防编辑器还在加载
    setTimeout(() => jumpToLine(line), 100)
    return
  }
  
  try {
    const doc = view.state.doc
    const totalLines = doc.lines
    console.log('[EditorPane] Got editor view, doc lines:', totalLines)
    
    // 确保行号在有效范围内
    const targetLine = Math.max(1, Math.min(line, totalLines))
    console.log('[EditorPane] Target line (clamped):', targetLine, 'original:', line)
    
    const lineInfo = doc.line(targetLine)
    console.log('[EditorPane] Line info:', { from: lineInfo.from, to: lineInfo.to, text: lineInfo.text.slice(0, 50) })
    
    // 滚动到该行并设置光标位置到行首
    view.dispatch({
      selection: { anchor: lineInfo.from, head: lineInfo.from },
      effects: EditorView.scrollIntoView(lineInfo.from, { y: 'start' })
    })
    
    console.log('[EditorPane] Dispatched selection and scroll')
    
    // 确保编辑器获得焦点
    view.focus()
    console.log('[EditorPane] Focused editor successfully')
  } catch (error) {
    console.error('[EditorPane] Error jumping to line:', error)
  }
}

/**
 * 跳转到搜索结果（支持精确匹配位置）
 */
function jumpToSearchResult(result: any) {
  console.log('[EditorPane] jumpToSearchResult called with:', {
    line: result.line,
    matchStart: result.matchStart,
    matchEnd: result.matchEnd,
    file: result.file?.name
  })
  
  // 通过 getEditorView() 方法获取 editorView
  const view = (editorRef.value as any)?.getEditorView?.()
  if (!view) {
    console.warn('[EditorPane] Editor view not available for search result, retrying in 100ms...')
    setTimeout(() => jumpToSearchResult(result), 100)
    return
  }
  
  try {
    const doc = view.state.doc
    const totalLines = doc.lines
    
    // 确保行号在有效范围内
    const targetLine = Math.max(1, Math.min(result.line, totalLines))
    const line = doc.line(targetLine)
    
    // 计算字符位置（如果没有精确位置信息，则跳转到行首）
    let pos = line.from
    let endPos = line.from
    
    if (result.matchStart !== undefined && result.matchEnd !== undefined) {
      // 使用精确的匹配位置
      pos = Math.max(line.from, Math.min(line.from + result.matchStart, line.to))
      endPos = Math.max(pos, Math.min(line.from + result.matchEnd, line.to))
      console.log('[EditorPane] Using precise match positions:', { pos, endPos })
    } else {
      console.log('[EditorPane] Using line start position')
    }
    
    // 跳转并选中匹配的文本
    view.dispatch({
      selection: { anchor: pos, head: endPos },
      effects: EditorView.scrollIntoView(pos, { y: 'start' })
    })
    
    console.log('[EditorPane] Dispatched search result selection and scroll')
    
    // 确保编辑器获得焦点
    view.focus()
    console.log('[EditorPane] Focused editor after search result jump')
  } catch (error) {
    console.error('[EditorPane] Error jumping to search result:', error)
  }
}

// 处理编辑器右键菜单
function handleEditorContextMenu(event: MouseEvent) {
  const menuWidth = 200  // 菜单宽度
  const menuHeight = 250 // 菜单高度（估计值）
  
  let x = event.clientX
  let y = event.clientY
  
  // 检查右侧边界
  if (x + menuWidth > window.innerWidth) {
    x = window.innerWidth - menuWidth - 10
  }
  
  // 检查底部边界
  if (y + menuHeight > window.innerHeight) {
    y = window.innerHeight - menuHeight - 10
  }
  
  // 确保不会超出左侧和顶部
  x = Math.max(10, x)
  y = Math.max(10, y)
  
  editorContextMenuX.value = x
  editorContextMenuY.value = y
  editorContextMenuVisible.value = true
}

// 关闭编辑器右键菜单
function closeEditorContextMenu() {
  editorContextMenuVisible.value = false
}

// 处理编辑器右键菜单操作
function handleEditorContextMenuAction(action: string) {
  closeEditorContextMenu()
  emit('editorContextMenuAction', action, props.pane.id)
}

// 获取编辑器实例的方法
function getEditorMethods() {
  return editorRef.value
}

// 暴露方法供父组件调用
defineExpose({
  jumpToLine,
  jumpToSearchResult,
  getEditorMethods
})
</script>

<template>
  <div 
    class="h-full flex flex-col ui-island rounded-xl overflow-hidden transition-colors duration-200"
    :class="{ 'ring-1 ring-hoi4-selected/50 shadow-sm': isActive }"
    @click="handleActivate"
  >
    <!-- 文件标签栏 -->
    <div v-if="pane.openFiles.length > 0" class="ui-island-header ui-separator-bottom">
      <EditorTabs
        :open-files="pane.openFiles"
        :active-file-index="pane.activeFileIndex"
        @switch-file="handleSwitchFile"
        @close-file="handleCloseFile"
        @context-menu="handleContextMenu"
      />
      
      <!-- 编辑器工具栏 -->
      <div class="px-4 py-2 flex items-center justify-between bg-hoi4-accent/60 ui-separator-bottom">
        <div class="flex items-center space-x-2">
          <button
            @click="handleSaveFile"
            class="px-3 py-1 bg-hoi4-gray hover:bg-hoi4-border rounded text-hoi4-text text-xs transition-colors flex items-center space-x-1"
            :disabled="!hasUnsavedChanges"
            :class="{ 'opacity-50 cursor-not-allowed': !hasUnsavedChanges }"
            title="保存 (Ctrl+S)"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4"></path>
            </svg>
            <span>保存</span>
          </button>
          
          <button
            v-if="pane.openFiles.length > 0"
            @click="handleSplitPane"
            class="px-3 py-1 bg-hoi4-gray hover:bg-hoi4-border rounded text-hoi4-text text-xs transition-colors flex items-center space-x-1"
            title="向右分割"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 4v16m6-16v16"></path>
            </svg>
            <span>分割</span>
          </button>
          
          <button
            v-if="isEventFile"
            @click="handlePreviewEvent"
            class="px-3 py-1 bg-hoi4-gray hover:bg-hoi4-border rounded text-hoi4-text text-xs transition-colors flex items-center space-x-1"
            title="预览事件关系图"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path>
            </svg>
            <span>预览</span>
          </button>
          
          <button
            v-if="isFocusFile"
            @click="handlePreviewFocus"
            class="px-3 py-1 bg-hoi4-gray hover:bg-hoi4-border rounded text-hoi4-text text-xs transition-colors flex items-center space-x-1"
            title="预览国策树"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7"></path>
            </svg>
            <span>预览</span>
          </button>

          <button
            v-if="isMapFile"
            @click="handlePreviewMap"
            class="px-3 py-1 bg-hoi4-gray hover:bg-hoi4-border rounded text-hoi4-text text-xs transition-colors flex items-center space-x-1"
            title="预览世界地图"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7"></path>
            </svg>
            <span>预览地图</span>
          </button>

          <button
            v-if="isGuiFile"
            @click="handlePreviewGui"
            class="px-3 py-1 bg-hoi4-gray hover:bg-hoi4-border rounded text-hoi4-text text-xs transition-colors flex items-center space-x-1"
            title="预览 GUI 界面"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v14a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13h16M10 4v9"></path>
            </svg>
            <span>预览 GUI</span>
          </button>

          <button
            v-if="isMioFile"
            @click="handlePreviewMio"
            class="px-3 py-1 bg-hoi4-gray hover:bg-hoi4-border rounded text-hoi4-text text-xs transition-colors flex items-center space-x-1"
            title="预览 MIO"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L7 19.75 4.25 17M7 19.75V4.25m10.75 2L17 4.25 14.25 7M17 4.25v15.5"></path>
            </svg>
            <span>预览 MIO</span>
          </button>

          <button
            v-if="isGfxFile"
            @click="handlePreviewGfx"
            class="px-3 py-1 bg-hoi4-gray hover:bg-hoi4-border rounded text-hoi4-text text-xs transition-colors flex items-center space-x-1"
            title="预览 GFX"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7h16M4 12h16M4 17h16" />
            </svg>
            <span>预览 GFX</span>
          </button>
        </div>
        
        <div class="flex items-center space-x-4 text-xs text-hoi4-text-dim">
          <span>行: {{ currentLine }}</span>
          <span>列: {{ currentColumn }}</span>
          <span>字符: {{ fileContent.length }}</span>
        </div>
      </div>
    </div>

    <!-- 编辑器 / 图片预览 / 事件关系图预览 -->
    <div v-if="currentFile" class="flex-1 overflow-hidden relative bg-hoi4-dark">
     <!-- 世界地图预览 (使用 v-if 进行首次初始化，v-show 保持状态并避免重新加载) -->
     <WorldMapViewer
       v-if="hasMapLoaded"
       v-show="isCurrentFileWorldMap"
       :project-path="projectPath"
       :game-directory="gameDirectory"
       :dependency-roots="dependencyRoots"
       :preview-source-path="currentFile?.sourceFilePath || currentFile?.node.path"
       :merge-mode="currentMergeMode"
     />

      <!-- 其他预览器 (使用 v-if 以节省资源) -->
      <template v-if="!isCurrentFileWorldMap">
        <!-- 图片预览 -->
        <ImagePreviewer
          v-if="isCurrentFileImage"
          :image-url="imageUrl"
          :file-name="currentFile.node.name"
          :file-path="currentFile.node.path"
        />
        
        <!-- 事件关系图预览 -->
        <EventGraphViewer
          v-else-if="isCurrentFileEventGraph"
          :content="currentFile.content"
          :file-path="currentFile.node.path"
          @jump-to-event="handleJumpToEvent"
        />
        
        <!-- 国策树预览 -->
        <FocusTreeViewer
          v-else-if="isCurrentFileFocusTree"
          :content="fileContent"
          :file-path="currentFile.node.path"
          :project-path="projectPath"
          :game-directory="gameDirectory"
          :dependency-roots="dependencyRoots"
          @jump-to-focus="handleJumpToFocus"
          @update-content="handleContentChange"
          @external-file-update="handleExternalFileUpdate"
        />
        
        <!-- GUI 预览 -->
        <GuiViewer
          v-else-if="isCurrentFileGui"
          :content="fileContent"
          :file-path="currentFile.node.path"
          :project-path="projectPath"
          :game-directory="gameDirectory"
        />

        <!-- MIO 预览 -->
        <MioViewer
          v-else-if="isCurrentFileMio"
          :content="fileContent"
          :file-path="currentFile.node.path"
          :project-path="projectPath"
          :game-directory="gameDirectory"
          :dependency-roots="dependencyRoots"
          @jump-to-trait="handleJumpToMio"
        />

        <!-- GFX 预览 -->
        <GfxViewer
          v-else-if="isCurrentFileGfx"
          :content="fileContent"
          :file-path="currentFile.node.path"
          :project-path="projectPath"
          :game-directory="gameDirectory"
          :dependency-roots="dependencyRoots"
          @jump-to-line="handleJumpToGfx"
        />
        
        <!-- 代码编辑器 -->
        <CodeMirrorEditor
          v-else
          ref="editorRef"
          :content="fileContent"
          :file-name="currentFile.node.name"
          :file-path="currentFile.node.path"
          :project-root="projectPath"
          :game-directory="gameDirectory"
          :disable-error-handling="disableErrorHandling"
          @update:content="handleContentChange"
          @cursor-change="handleCursorChange"
          @contextmenu="handleEditorContextMenu"
        />
      </template>
    </div>

    <!-- 空状态 -->
    <div v-else class="flex-1 flex items-center justify-center">
      <div class="text-center">
        <h2 class="text-hoi4-text text-xl font-bold mb-2">编辑器窗格</h2>
        <p class="text-hoi4-text-dim text-sm">点击左侧文件树中的文件进行编辑</p>
      </div>
    </div>

    <!-- 编辑器右键菜单 -->
    <ContextMenu
      :visible="editorContextMenuVisible"
      :x="editorContextMenuX"
      :y="editorContextMenuY"
      menu-type="editor"
      :current-file-path="currentFile?.node.path"
      @action="handleEditorContextMenuAction"
      @close="closeEditorContextMenu"
    />
  </div>
  
  <!-- 全局点击关闭菜单 -->
  <Teleport to="body">
    <div
      v-if="editorContextMenuVisible"
      class="fixed inset-0 z-40"
      @click="closeEditorContextMenu"
      @contextmenu.prevent="closeEditorContextMenu"
    ></div>
  </Teleport>
</template>

<style scoped>
</style>
