<script setup lang="ts">
import { ref } from 'vue'
import EditorPane from './EditorPane.vue'
import ConfirmDialog from './ConfirmDialog.vue'
import { useEditorGroups } from '../../composables/useEditorGroups'
import type { FileNode } from '../../composables/useFileManager'

const props = defineProps<{
  projectPath: string
  gameDirectory: string
  dependencyRoots?: string[]
  autoSave: boolean
  disableErrorHandling?: boolean
}>()

const emit = defineEmits<{
  openFile: [node: FileNode, paneId?: string]
  contextMenu: [event: MouseEvent, paneId: string, fileIndex: number]
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
  contentChange: [paneId: string, content: string]
}>()

const {
  panes,
  activePaneId,
  activePane,
  splitPane,
  closePane,
  setActivePane
} = useEditorGroups()

const resizingPaneIndex = ref<number | null>(null)
const resizeStartX = ref(0)
const resizeStartWidths = ref<number[]>([])

// 确认对话框状态
const confirmDialogVisible = ref(false)
const confirmDialogTitle = ref('')
const confirmDialogMessage = ref('')
const confirmDialogType = ref<'warning' | 'danger' | 'info'>('warning')
let confirmDialogResolve: ((value: boolean) => void) | null = null

/**
 * 显示确认对话框
 */
function showConfirmDialog(message: string, title = '⚠️ 确认操作', type: 'warning' | 'danger' | 'info' = 'warning'): Promise<boolean> {
  return new Promise((resolve) => {
    confirmDialogMessage.value = message
    confirmDialogTitle.value = title
    confirmDialogType.value = type
    confirmDialogVisible.value = true
    confirmDialogResolve = resolve
  })
}

/**
 * 处理确认对话框确认
 */
function handleConfirmDialogConfirm() {
  confirmDialogVisible.value = false
  if (confirmDialogResolve) {
    confirmDialogResolve(true)
    confirmDialogResolve = null
  }
}

/**
 * 处理确认对话框取消
 */
function handleConfirmDialogCancel() {
  confirmDialogVisible.value = false
  if (confirmDialogResolve) {
    confirmDialogResolve(false)
    confirmDialogResolve = null
  }
}

// EditorPane 引用 Map
const paneRefs = ref<Map<string, InstanceType<typeof EditorPane>>>(new Map())

// 自动保存防抖计时器
const autoSaveTimers = ref<Map<string, number>>(new Map())
const AUTO_SAVE_DELAY = 100 // 0.1秒防抖

// 处理文件切换
function handleSwitchFile(paneId: string, index: number) {
  const pane = panes.value.find(p => p.id === paneId)
  if (!pane) return
  
  pane.activeFileIndex = index
  setActivePane(paneId)
}

// 处理文件关闭
async function handleCloseFile(paneId: string, index: number) {
  const pane = panes.value.find(p => p.id === paneId)
  if (!pane) return
  
  const file = pane.openFiles[index]
  
  // 检查未保存更改
  if (file.hasUnsavedChanges) {
    const confirmed = await showConfirmDialog(
      `文件 "${file.node.name}" 有未保存的更改，是否放弃更改？`,
      '⚠️ 未保存的更改',
      'warning'
    )
    if (!confirmed) {
      return
    }
  }
  
  // 从列表中移除
  pane.openFiles.splice(index, 1)
  
  // 调整活动文件索引
  if (pane.openFiles.length === 0) {
    pane.activeFileIndex = -1
    
    // 如果窗格为空且有多个窗格，自动删除该窗格（至少保留一个窗格）
    if (panes.value.length > 1) {
      // 延迟一下执行，确保 UI 更新
      setTimeout(() => {
        closePane(paneId)
      }, 100)
    }
  } else if (index === pane.activeFileIndex) {
    pane.activeFileIndex = Math.min(index, pane.openFiles.length - 1)
  } else if (index < pane.activeFileIndex) {
    pane.activeFileIndex--
  }
}

// 处理右键菜单
function handleContextMenu(event: MouseEvent, paneId: string, index: number) {
  emit('contextMenu', event, paneId, index)
}

// 处理内容变化
function handleContentChange(paneId: string, content: string) {
  const pane = panes.value.find(p => p.id === paneId)
  if (!pane || pane.activeFileIndex === -1) return
  
  const file = pane.openFiles[pane.activeFileIndex]
  if (file) {
    // 只有内容真的改变时才标记为已修改
    if (file.content !== content) {
      file.content = content
      file.hasUnsavedChanges = true

      const path = file.node?.path
      if (path) {
        for (const p of panes.value) {
          for (const openFile of p.openFiles) {
            if (!openFile?.node?.path) continue
            if (openFile.node.path !== path) continue
            if (openFile === file) continue
            if (openFile.hasUnsavedChanges) continue
            if (openFile.isImage) continue
            openFile.content = content
          }
        }
      }
      
      // 发射内容变化事件，让父组件可以同步预览文件
      emit('contentChange', paneId, content)
      
      // 如果启用了自动保存，设置防抖计时器
      if (props.autoSave) {
        // 清除旧的计时器
        const existingTimer = autoSaveTimers.value.get(paneId)
        if (existingTimer) {
          clearTimeout(existingTimer)
        }
        
        // 设置新的计时器
        const timer = window.setTimeout(() => {
          handleSaveFile(paneId)
          autoSaveTimers.value.delete(paneId)
        }, AUTO_SAVE_DELAY)
        
        autoSaveTimers.value.set(paneId, timer)
      }
    }
  }
}

// 处理光标变化
function handleCursorChange(paneId: string, line: number, column: number) {
  const pane = panes.value.find(p => p.id === paneId)
  if (!pane || pane.activeFileIndex === -1) return
  
  const file = pane.openFiles[pane.activeFileIndex]
  if (file) {
    file.cursorLine = line
    file.cursorColumn = column
  }
}

// 处理保存文件
async function handleSaveFile(paneId: string) {
  const pane = panes.value.find(p => p.id === paneId)
  if (!pane || pane.activeFileIndex === -1) return
  
  const file = pane.openFiles[pane.activeFileIndex]
  if (!file || !file.hasUnsavedChanges) return
  
  try {
    const { writeFileContent, readFileContent } = await import('../../api/tauri')
    const result = await writeFileContent(file.node.path, file.content)
    if (result.success) {
      file.hasUnsavedChanges = false

      try {
        const readResult = await readFileContent(file.node.path)
        if (readResult.success && typeof readResult.content === 'string') {
          file.content = readResult.content

          const path = file.node?.path
          if (path) {
            for (const p of panes.value) {
              for (const openFile of p.openFiles) {
                if (!openFile?.node?.path) continue
                if (openFile.node.path !== path) continue
                if (openFile === file) continue
                if (openFile.hasUnsavedChanges) continue
                if (openFile.isImage) continue
                openFile.content = readResult.content
              }
            }
          }
        }
      } catch (error) {
        console.warn('保存后重新读取文件失败:', error)
      }
    } else {
      alert(`保存失败: ${result.message}`)
    }
  } catch (error) {
    console.error('保存文件失败:', error)
    alert(`保存失败: ${error}`)
  }
}

// 处理窗格激活
function handleActivate(paneId: string) {
  setActivePane(paneId)
}

// 处理分割窗格
function handleSplitPane(paneId: string, fileIndex?: number) {
  splitPane(paneId, fileIndex)
}

// 处理错误变化
function handleErrorsChange(paneId: string, errors: Array<{line: number, msg: string, type: string}>) {
  emit('errorsChange', paneId, errors)
}

// 处理编辑器右键菜单操作
function handleEditorContextMenuAction(action: string, paneId: string) {
  emit('editorContextMenuAction', action, paneId)
}

// 处理预览事件
function handlePreviewEvent(paneId: string) {
  emit('previewEvent', paneId)
}

// 处理预览国策树
function handlePreviewFocus(paneId: string) {
  emit('previewFocus', paneId)
}

// 处理预览地图
function handlePreviewMap(paneId: string) {
  emit('previewMap', paneId)
}

// 处理预览 GUI
function handlePreviewGui(paneId: string) {
  emit('previewGui', paneId)
}

// 处理预览 MIO
function handlePreviewMio(paneId: string) {
  emit('previewMio', paneId)
}

function handlePreviewGfx(paneId: string) {
  emit('previewGfx', paneId)
}

// 处理跳转到焦点
function handleJumpToFocusFromPreview(sourcePaneId: string, sourceFilePath: string, focusId: string, line: number) {
  emit('jumpToFocusFromPreview', sourcePaneId, sourceFilePath, focusId, line)
}

function handleJumpToMioFromPreview(sourcePaneId: string, sourceFilePath: string, traitId: string, line: number) {
  emit('jumpToMioFromPreview', sourcePaneId, sourceFilePath, traitId, line)
}

function handleJumpToGfxFromPreview(sourcePaneId: string, sourceFilePath: string, line: number) {
  emit('jumpToGfxFromPreview', sourcePaneId, sourceFilePath, line)
}

// 处理关闭窗格
function handleClosePane(paneId: string) {
  closePane(paneId)
}

// 处理外部文件更新
function handleExternalFileUpdate(_paneId: string, filePath: string, content: string) {
  const normalizePath = (p?: string) => (p || '').replace(/\\/g, '/').replace(/\/+/g, '/').trim()
  const target = normalizePath(filePath)
  if (!target) return

  for (const p of panes.value) {
    for (const openFile of p.openFiles) {
      if (!openFile?.node?.path) continue
      if (normalizePath(openFile.node.path) !== target) continue
      if (openFile.hasUnsavedChanges) continue
      if (openFile.isImage) continue
      openFile.content = content
    }
  }
}

// 开始调整窗格大小
function startResize(index: number, event: MouseEvent) {
  if (index >= panes.value.length - 1) return
  
  resizingPaneIndex.value = index
  resizeStartX.value = event.clientX
  resizeStartWidths.value = panes.value.map(p => p.width)
  
  document.addEventListener('mousemove', handleResize)
  document.addEventListener('mouseup', stopResize)
  event.preventDefault()
}

// 调整窗格大小
function handleResize(event: MouseEvent) {
  if (resizingPaneIndex.value === null) return
  
  const containerWidth = document.querySelector('.editor-group-container')?.clientWidth || 1000
  const deltaX = event.clientX - resizeStartX.value
  const deltaPercent = (deltaX / containerWidth) * 100
  
  const index = resizingPaneIndex.value
  const newWidth1 = Math.max(10, Math.min(90, resizeStartWidths.value[index] + deltaPercent))
  const newWidth2 = Math.max(10, Math.min(90, resizeStartWidths.value[index + 1] - deltaPercent))
  
  // 确保总宽度不变
  if (newWidth1 >= 10 && newWidth2 >= 10) {
    panes.value[index].width = newWidth1
    panes.value[index + 1].width = newWidth2
  }
}

// 停止调整大小
function stopResize() {
  resizingPaneIndex.value = null
  document.removeEventListener('mousemove', handleResize)
  document.removeEventListener('mouseup', stopResize)
}

// 打开文件到指定窗格
function openFileInPane(node: FileNode, paneId?: string) {
  const targetPaneId = paneId || activePaneId.value
  const pane = panes.value.find(p => p.id === targetPaneId)
  if (!pane) return
  
  emit('openFile', node, targetPaneId)
}

/**
 * 设置 EditorPane 引用
 */
function setPaneRef(paneId: string, el: any) {
  if (el) {
    paneRefs.value.set(paneId, el)
  } else {
    paneRefs.value.delete(paneId)
  }
}

/**
 * 跳转到错误行（在当前活动的 Pane 中）
 */
function jumpToErrorLine(line: number) {
  console.log('[EditorGroup] jumpToErrorLine called with line:', line)
  console.log('[EditorGroup] activePaneId:', activePaneId.value)
  console.log('[EditorGroup] paneRefs size:', paneRefs.value.size)
  console.log('[EditorGroup] paneRefs keys:', Array.from(paneRefs.value.keys()))
  
  if (!activePaneId.value) {
    console.warn('[EditorGroup] No active pane')
    return
  }
  
  const paneRef = paneRefs.value.get(activePaneId.value)
  if (!paneRef) {
    console.warn('[EditorGroup] Active pane ref not found for id:', activePaneId.value)
    return
  }
  
  console.log('[EditorGroup] Calling jumpToLine on pane')
  paneRef.jumpToLine(line)
}

/**
 * 跳转到搜索结果（支持精确匹配位置）
 */
function jumpToSearchResult(result: any) {
  console.log('[EditorGroup] jumpToSearchResult called with:', {
    line: result.line,
    matchStart: result.matchStart,
    matchEnd: result.matchEnd,
    file: result.file?.name
  })
  
  if (!activePaneId.value) {
    console.warn('[EditorGroup] No active pane')
    return
  }
  
  const paneRef = paneRefs.value.get(activePaneId.value)
  if (!paneRef) {
    console.warn('[EditorGroup] Active pane ref not found for id:', activePaneId.value)
    return
  }
  
  console.log('[EditorGroup] Calling jumpToSearchResult on pane')
  paneRef.jumpToSearchResult(result)
}

/**
 * 保存当前活动窗格的文件
 */
async function saveCurrentFile(): Promise<boolean> {
  if (!activePaneId.value) return false
  
  const pane = panes.value.find(p => p.id === activePaneId.value)
  if (!pane || pane.activeFileIndex === -1) return false
  
  const file = pane.openFiles[pane.activeFileIndex]
  if (!file || !file.hasUnsavedChanges) return false
  
  await handleSaveFile(activePaneId.value)
  return true
}

// 暴露方法供父组件使用
defineExpose({
  panes,
  activePaneId,
  activePane,
  openFileInPane,
  splitPane,
  closePane,
  setActivePane,
  jumpToErrorLine,
  jumpToSearchResult,
  saveCurrentFile,
  paneRefs
})
</script>

<template>
  <div class="editor-group-container flex-1 flex overflow-hidden">
    <!-- 确认对话框 -->
    <ConfirmDialog
      :visible="confirmDialogVisible"
      :title="confirmDialogTitle"
      :message="confirmDialogMessage"
      :type="confirmDialogType"
      @confirm="handleConfirmDialogConfirm"
      @cancel="handleConfirmDialogCancel"
    />
    <template v-for="(pane, index) in panes" :key="pane.id">
      <!-- 编辑器窗格 -->
      <div 
        class="editor-pane-wrapper relative px-1 py-1"
        :style="{ width: pane.width + '%' }"
      >
        <!-- 关闭按钮（当有多个窗格时显示） -->
        <button
          v-if="panes.length > 1"
          @click="handleClosePane(pane.id)"
          class="absolute top-2 right-2 z-10 p-1 bg-hoi4-gray hover:bg-red-600 rounded text-hoi4-text transition-colors"
          title="关闭窗格"
        >
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
          </svg>
        </button>
        
        <EditorPane
          :ref="(el) => setPaneRef(pane.id, el)"
          :pane="pane"
          :is-active="pane.id === activePaneId"
          :project-path="projectPath"
          :game-directory="gameDirectory"
          :dependency-roots="props.dependencyRoots"
          :disable-error-handling="props.disableErrorHandling"
          @switch-file="handleSwitchFile"
          @close-file="handleCloseFile"
          @context-menu="handleContextMenu"
          @content-change="handleContentChange"
          @external-file-update="handleExternalFileUpdate"
          @cursor-change="handleCursorChange"
          @save-file="handleSaveFile"
          @activate="handleActivate"
          @split-pane="handleSplitPane"
          @errors-change="handleErrorsChange"
          @editor-context-menu-action="handleEditorContextMenuAction"
          @preview-event="handlePreviewEvent"
          @preview-focus="handlePreviewFocus"
          @preview-map="handlePreviewMap"
          @preview-gui="handlePreviewGui"
          @preview-mio="handlePreviewMio"
          @preview-gfx="handlePreviewGfx"
          @jump-to-focus-from-preview="handleJumpToFocusFromPreview"
          @jump-to-mio-from-preview="handleJumpToMioFromPreview"
          @jump-to-gfx-from-preview="handleJumpToGfxFromPreview"
        />
      </div>
      
      <!-- 拖动分隔条 -->
      <div
        v-if="index < panes.length - 1"
        class="w-1 bg-hoi4-border/60 hover:bg-hoi4-accent/80 cursor-col-resize flex-shrink-0 transition-colors"
        @mousedown="startResize(index, $event)"
      ></div>
    </template>
  </div>
</template>

<style scoped>
.cursor-col-resize {
  cursor: col-resize;
}

.editor-pane-wrapper {
  min-width: 10%;
  max-width: 100%;
}
</style>
