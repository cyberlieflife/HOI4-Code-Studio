<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { buildDirectoryTreeFast, createFile, createFolder, writeFileContent, launchGame, renamePath, deletePath, openFolder, loadSettingsSnapshot, type Settings } from '../api/tauri'
import 'highlight.js/styles/github-dark.css'
import 'highlight.js/lib/languages/json'
import 'highlight.js/lib/languages/yaml'

// 组件导入
import EditorToolbar from '../components/editor/EditorToolbar.vue'
import EditorGroup from '../components/editor/EditorGroup.vue'
import RightPanel from '../components/editor/RightPanel.vue'
import ContextMenu from '../components/editor/ContextMenu.vue'
import CreateDialog from '../components/editor/CreateDialog.vue'
import ConfirmDialog from '../components/editor/ConfirmDialog.vue'
import FileTreeNode from '../components/FileTreeNode.vue'
import LeftPanelTabs from '../components/editor/LeftPanelTabs.vue'
import DependencyManager from '../components/editor/DependencyManager.vue'
import LoadingMonitor from '../components/editor/LoadingMonitor.vue'
import PackageDialog from '../components/editor/PackageDialog.vue'
import EditorWorkspaceShell from '../components/editor/EditorWorkspaceShell.vue'

// Composables 导入
import { type FileNode } from '../composables/useFileManager'
import { useSearch } from '../composables/useSearch'
import { useKeyboardShortcuts } from '../composables/useKeyboardShortcuts'
import { usePanelResize } from '../composables/usePanelResize'

import { setTagRoots, useTagRegistry } from '../composables/useTagRegistry'
import { useTheme } from '../composables/useTheme'
import { useFileTreeIcons } from '../composables/useFileTreeIcons'
import { setIdeaRoots, useIdeaRegistry } from '../composables/useIdeaRegistry'
import { logger } from '../utils/logger'
import { readFileContent } from '../api/tauri'
import { useDependencyManager } from '../composables/useDependencyManager'
import { useDependencyTreeCache } from '../composables/useDependencyTreeCache'
import { useProjectBootstrap } from '../composables/useProjectBootstrap'
import { useEditorSettingsSync } from '../composables/useEditorSettingsSync'
import { useProjectFileTreeLoader } from '../composables/useProjectFileTreeLoader'
import { useAutoRefreshInterval } from '../composables/useAutoRefreshInterval'

// 新提取的模块
import { escapeRegExp, isImageFile, isPathUnder, convertRustFileNode, DIRECTORY_EXPAND_LOAD_DEPTH } from '../utils/fileUtils'
import { useConfirmDialog } from '../composables/useConfirmDialog'
import { useContextMenu } from '../composables/useContextMenu'
import { loadFontConfigFromSettings } from '../composables/useEditorFont'
import { usePluginManager } from '../composables/usePluginManager'
import { handleInsertTemplate, type EditorMethods } from '../composables/useEditorTemplates'
import { jumpFromFocusPreview, jumpFromGfxPreview, jumpFromMioPreview } from '../composables/usePreviewNavigation'
import { usePreviewPaneManager } from '../composables/usePreviewPaneManager'
import { useSearchNavigation } from '../composables/useSearchNavigation'
import { useEditorErrorNavigation } from '../composables/useEditorErrorNavigation'
import PluginIframeHost from '../components/plugins/PluginIframeHost.vue'
import { useEditorUiState } from '../composables/useEditorUiState'
import { markStartupStep } from '../utils/startupPerformance'

// Highlight.js 语言定义已移至 useSyntaxHighlight.ts 中

const router = useRouter()
const route = useRoute()

// 基础状态
const projectPath = ref('')
const selectedNode = ref<FileNode | null>(null)
const txtErrors = ref<{line: number, msg: string, type: string}[]>([])
const isLaunchingGame = ref(false)

const {
  rightPanelExpanded,
  createDialogVisible,
  createDialogType,
  createDialogMode,
  createDialogInitialValue,
  leftPanelActiveTab,
  activeDependencyId,
  dependencyManagerVisible,
  activeLeftPluginPanelUid,
  loadingMonitorVisible,
  packageDialogVisible,
  rightPanelActiveTab,
  activeRightPluginPanelUid,
  handleSwitchToProject,
  handleSwitchToPlugins,
  handleManageDependencies,
  openDependenciesFromToolbar,
  toggleLoadingMonitor,
  openPackageDialog,
  toggleRightPanel,
  handlePluginToolbarClick
} = useEditorUiState()

// 右键菜单状态（使用 composable）
const {
  contextMenuVisible,
  contextMenuX,
  contextMenuY,
  contextMenuType,
  contextMenuPaneId,
  contextMenuFileIndex,
  treeContextMenuNode,
  showFileTabContextMenu,
  showTreeContextMenu,
  hideContextMenu
} = useContextMenu()

// 创建对话框状态

// 确认对话框状态（使用 composable）
const {
  confirmDialogVisible,
  confirmDialogTitle,
  confirmDialogMessage,
  confirmDialogType,
  showConfirmDialog,
  handleConfirmDialogConfirm,
  handleConfirmDialogCancel
} = useConfirmDialog()

function syncRegistryRoots(options: { projectPath: string; gameDirectory?: string; dependencyPaths: string[] }) {
  setTagRoots(options.projectPath, options.gameDirectory, options.dependencyPaths)
  setIdeaRoots(options.projectPath, options.gameDirectory, options.dependencyPaths)
}

const { projectInfo, loadProjectInfo } = useProjectBootstrap(projectPath, showConfirmDialog)

// 预览跳转函数（使用 usePreviewNavigation 模块）
async function handleJumpToFocusFromPreview(sourcePaneId: string, sourceFilePath: string, focusId: string, line: number) {
  await jumpFromFocusPreview(editorGroupRef.value, sourcePaneId, sourceFilePath, focusId, line, handleOpenFile)
}

async function handleJumpToGfxFromPreview(sourcePaneId: string, sourceFilePath: string, line: number) {
  await jumpFromGfxPreview(editorGroupRef.value, sourcePaneId, sourceFilePath, line, handleOpenFile)
}

async function handlePerformReplace(replaceText: string) {
  if (!searchQuery.value.trim()) return
  if (searchResults.value.length === 0) return

  const confirmed = await showConfirmDialog(
    `确定要将搜索到的内容替换为 "${replaceText}" 吗？该操作将直接修改文件内容，且不可恢复。`,
    '✏️ 替换确认',
    'warning'
  )
  if (!confirmed) return

  const flags = searchCaseSensitive.value ? 'g' : 'gi'
  let pattern: RegExp
  try {
    pattern = searchRegex.value
      ? new RegExp(searchQuery.value, flags)
      : new RegExp(escapeRegExp(searchQuery.value), flags)
  } catch (error) {
    alert(`替换失败：无效的正则表达式: ${error}`)
    return
  }

  const filePaths = Array.from(new Set(searchResults.value.map(r => r.file.path)))
  let totalReplacements = 0
  const updatedContents = new Map<string, string>()

  function syncOpenedFilesContent() {
    if (!editorGroupRef.value) return

    for (const pane of editorGroupRef.value.panes) {
      for (const openFile of pane.openFiles) {
        if (!openFile?.node?.path) continue
        if (openFile.hasUnsavedChanges) continue
        if (openFile.isImage) continue

        const updated = updatedContents.get(openFile.node.path)
        if (updated !== undefined) {
          openFile.content = updated
        }
      }
    }
  }

  try {
    for (const filePath of filePaths) {
      const readResult = await readFileContent(filePath)
      if (!readResult.success) {
        alert(`读取文件失败: ${filePath}\n${readResult.message}`)
        continue
      }

      const original = readResult.content ?? ''
      const matches = original.match(pattern)
      const matchCount = matches ? matches.length : 0
      if (matchCount === 0) continue

      const updated = original.replace(pattern, replaceText)
      const writeResult = await writeFileContent(filePath, updated)
      if (!writeResult.success) {
        alert(`写入文件失败: ${filePath}\n${writeResult.message}`)
        continue
      }

      updatedContents.set(filePath, updated)
      totalReplacements += matchCount
    }
  } catch (error) {
    logger.error('替换失败:', error)
    alert(`替换失败: ${error}`)
    return
  }

  syncOpenedFilesContent()

  await handlePerformSearch()
  alert(`替换完成：共替换 ${totalReplacements} 处。`)
}

// 依赖项管理状态
// Refs
const editorGroupRef = ref<InstanceType<typeof EditorGroup> | null>(null)

const { openPreview, openProjectMapPreview, syncPreviewContent } = usePreviewPaneManager(editorGroupRef)
const {
  jumpToError,
  jumpToNextError,
  jumpToPreviousError
} = useEditorErrorNavigation(editorGroupRef, txtErrors)

// 计算可移动到的窗格列表（排除当前窗格）
const availablePanesForMove = computed(() => {
  if (!editorGroupRef.value || contextMenuType.value !== 'pane') return []
  
  return editorGroupRef.value.panes
    .filter(p => p.id !== contextMenuPaneId.value)
    .map((p) => ({
      id: p.id,
      name: `窗格 ${editorGroupRef.value!.panes.findIndex(pane => pane.id === p.id) + 1}`
    }))
})

const {
  leftPanelWidth,
  rightPanelWidth,
  startResizeLeft,
  startResizeRight
} = usePanelResize()

// 搜索功能
const {
  searchQuery,
  searchResults,
  isSearching,
  searchCaseSensitive,
  searchRegex,
  searchScope,
  includeAllFiles,
  performSearch
} = useSearch()


const packageDialogRef = ref<InstanceType<typeof PackageDialog> | null>(null)

// 目录树自动刷新

const { isLoading: tagLoading, refresh: refreshTags, tags: tagList } = useTagRegistry()
const { isLoading: ideaLoading, refresh: refreshIdeas, ideas: ideaList } = useIdeaRegistry()

// 主题系统
const { toggleThemePanel, loadThemeFromSettings } = useTheme()

// 图标系统
const { toggleIconPanel, loadIconSetFromSettings } = useFileTreeIcons()

const pluginManager = usePluginManager()
const {
  leftPanels: pluginLeftPanels,
  rightPanels: pluginRightPanels,
  toolbarItems: pluginToolbarItems,
  refreshPlugins
} = pluginManager
// 依赖项管理
const dependencyManager = useDependencyManager(projectPath.value)
const {
  dependencies,
  isLoading: isDependencyLoading,
  addDependency,
  removeDependency,
  toggleDependency,
  loadDependencies: loadDependenciesList
} = dependencyManager

const {
  gameDirectory,
  gameFileTree,
  isLoadingGameTree,
  autoSave,
  disableErrorHandling,
  loadInitialSettings,
  loadGameDirectory,
  toggleAutoSave
} = useEditorSettingsSync({
  projectPath,
  dependencies,
  refreshTags,
  loadFontConfigFromSettings,
  syncRoots: syncRegistryRoots
})

const enabledDependencyRoots = computed(() =>
  (dependencies.value || []).filter(d => d.enabled).map(d => d.path)
)

const {
  loading,
  fileTree,
  loadFileTree
} = useProjectFileTreeLoader({
  projectPath,
  gameDirectory,
  getEnabledDependencyPaths: () => enabledDependencyRoots.value,
  syncRoots: syncRegistryRoots
})

const {
  hasDependencyTree,
  getDependencyTree,
  loadDependencyFileTree,
  invalidateDependencyFileTree
} = useDependencyTreeCache(dependencies)

const hasActiveDependencyTree = computed(() => hasDependencyTree(activeDependencyId.value))

const activeDependencyTree = computed(() => getDependencyTree(activeDependencyId.value))

const {
  start: startFileTreeAutoRefresh,
  stop: stopFileTreeAutoRefresh
} = useAutoRefreshInterval(() => {
  if (projectPath.value) {
    return loadFileTree()
  }
}, 2000)

async function handleRefreshTags() {
  await refreshTags()
}

async function handleRefreshIdeas() {
  await refreshIdeas()
}

async function loadEditorBackgroundData(settingsSnapshot: Settings) {
  try {
    await refreshPlugins()
    await loadDependenciesList()
    markStartupStep('startup:editor-dependencies-loaded', '编辑器依赖列表加载完成')

    await loadGameDirectory({ refreshRegistries: false, settings: settingsSnapshot })
    markStartupStep('startup:editor-game-directory-loaded', '编辑器游戏目录加载完成')

    await refreshTags()
    markStartupStep('startup:editor-tags-loaded', '编辑器标签索引加载完成')

    await refreshIdeas()
    markStartupStep('startup:editor-ideas-loaded', '编辑器创意索引加载完成')

    markStartupStep('startup:editor-background-ready', '编辑器后台初始化完成')
    markStartupStep('startup:editor-ready', '编辑器启动流程完成')
  } catch (error) {
    logger.error('编辑器后台初始化失败:', error)
  }
}

// 依赖项管理函数
function handleSwitchToDependency(id: string) {
  activeDependencyId.value = id
  leftPanelActiveTab.value = 'dependencies'
  loadDependencyFileTree(id)
}

function handleSwitchToPluginsTab() {
  handleSwitchToPlugins(pluginLeftPanels.value[0]?.uid)
}

async function handleAddDependency(path: string) {
  const result = await addDependency(path)
  if (result.success) {
    // 成功添加后刷新依赖项列表
    await loadDependenciesList()
  } else {
    alert(result.message)
  }
}

async function handleRemoveDependency(id: string) {
  const result = await removeDependency(id)
  if (result.success) {
    // 如果删除的是当前激活的依赖项，切换回项目
    if (activeDependencyId.value === id) {
      handleSwitchToProject()
    }
    invalidateDependencyFileTree(id)
  } else {
    alert(result.message)
  }
}

async function handleToggleDependency(id: string) {
  await toggleDependency(id)
}

// 切换文件夹
async function toggleFolder(node: FileNode) {
  if (!node.isDirectory) return
  selectedNode.value = node
  node.expanded = !node.expanded
  if (node.expanded && (!node.children || node.children.length === 0)) {
    try {
      const result = await buildDirectoryTreeFast(node.path, DIRECTORY_EXPAND_LOAD_DEPTH)
      if (result.success && result.tree) {
        node.children = result.tree.map(convertRustFileNode)
      }
    } catch (error) {
      logger.error('加载子目录失败:', error)
    }
  }
}

// 切换游戏文件夹
async function toggleGameFolder(node: FileNode) {
  if (!node.isDirectory) return
  node.expanded = !node.expanded
  if (node.expanded && (!node.children || node.children.length === 0)) {
    try {
      const result = await buildDirectoryTreeFast(node.path, DIRECTORY_EXPAND_LOAD_DEPTH)
      if (result.success && result.tree) {
        node.children = result.tree.map(convertRustFileNode)
      }
    } catch (error) {
      logger.error('加载游戏目录子目录失败:', error)
    }
  }
}

// 打开文件处理
async function handleOpenFile(node: FileNode, paneId?: string, jumpInfo?: any) {
  if (node.isDirectory) return
  
  selectedNode.value = node
  const targetPaneId = paneId || editorGroupRef.value?.activePaneId
  if (!targetPaneId) return
  
  const pane = editorGroupRef.value?.panes.find(p => p.id === targetPaneId)
  if (!pane) return
  
  // 检查文件是否已在该窗格中打开
  const existingIndex = pane.openFiles.findIndex(f => f.node.path === node.path && !f.isPreview)
  if (existingIndex !== -1) {
    pane.activeFileIndex = existingIndex
    
    // 如果有跳转信息且文件已存在，直接执行跳转，避免后续文件切换事件重置光标
    if (jumpInfo && editorGroupRef.value) {
      console.log('[Editor] File already open, jumping directly to avoid cursor reset')
      setTimeout(() => {
        editorGroupRef.value!.jumpToSearchResult(jumpInfo)
      }, 50) // 短暂延迟确保文件切换完成
    }
    return
  }
  
  // 检查是否为图片文件
  const isImage = isImageFile(node.path)
  
  // 如果是图片，读取为 base64
  if (isImage) {
    try {
      // 使用自定义命令读取图片文件为 base64
      const { readImageAsBase64 } = await import('../api/tauri')
      const result = await readImageAsBase64(node.path)
      if (result.success && result.base64) {
        pane.openFiles.push({
          node,
          content: result.base64, // 存储 base64 数据
          hasUnsavedChanges: false,
          cursorLine: 1,
          cursorColumn: 1,
          isImage: true
        })
        pane.activeFileIndex = pane.openFiles.length - 1
      } else {
        alert(`打开图片失败: ${result.message || '无法读取图片'}`)
      }
    } catch (error) {
      logger.error('打开图片失败:', error)
      alert(`打开图片失败: ${error}`)
    }
    return
  }
  
  // 读取文本文件内容
  try {
    const result = await readFileContent(node.path)
    if (result.success) {
      pane.openFiles.push({
        node,
        content: result.content,
        hasUnsavedChanges: false,
        cursorLine: 1,
        cursorColumn: 1,
        isImage: false
      })
      pane.activeFileIndex = pane.openFiles.length - 1
    } else {
      alert(`打开文件失败: ${result.message}`)
    }
  } catch (error) {
    logger.error('打开文件失败:', error)
    alert(`打开文件失败: ${error}`)
  }
}

const { jumpToSearchResult: handleJumpToSearchResult } = useSearchNavigation(editorGroupRef, handleOpenFile)

async function handlePreviewEvent(paneId: string) {
  await openPreview(paneId, 'event')
}

async function handlePreviewGfx(paneId: string) {
  await openPreview(paneId, 'gfx')
}

async function handlePreviewMio(paneId: string) {
  await openPreview(paneId, 'mio')
}

async function handlePreviewFocus(paneId: string) {
  await openPreview(paneId, 'focus')
}

async function handlePreviewMap(paneId: string) {
  await openPreview(paneId, 'map')
}

async function handlePreviewGui(paneId: string) {
  await openPreview(paneId, 'gui')
}

// 右键菜单包装函数（处理 selectedNode 高亮）
function handleShowTreeContextMenu(event: MouseEvent, node: FileNode | null = null) {
  showTreeContextMenu(event, node)
  // 强制高亮选中的节点
  if (node) {
    selectedNode.value = node
  }
}

function isProjectMapFolder(node: FileNode | null) {
  if (!node || !node.isDirectory) return false

  const normalizePath = (path: string) => path.replace(/\\/g, '/').replace(/\/+$/, '')
  return normalizePath(node.path) === `${normalizePath(projectPath.value)}/map`
}

async function closeOpenedFilesUnderPath(basePath: string) {
  if (!editorGroupRef.value) return

  for (const pane of editorGroupRef.value.panes) {
    const indicesToClose: number[] = []
    for (let i = 0; i < pane.openFiles.length; i++) {
      const file = pane.openFiles[i]
      if (file?.node?.path && isPathUnder(file.node.path, basePath)) {
        indicesToClose.push(i)
      }
    }

    for (let i = indicesToClose.length - 1; i >= 0; i--) {
      pane.openFiles.splice(indicesToClose[i], 1)
    }

    if (pane.openFiles.length === 0) {
      pane.activeFileIndex = -1
    } else if (pane.activeFileIndex >= pane.openFiles.length) {
      pane.activeFileIndex = pane.openFiles.length - 1
    }
  }
}

async function handleContextMenuAction(action: string, payload?: any) {
  if (contextMenuType.value === 'pane') {
    const pane = editorGroupRef.value?.panes.find(p => p.id === contextMenuPaneId.value)
    if (!pane) return
    
    if (action === 'splitRight') {
      editorGroupRef.value?.splitPane(contextMenuPaneId.value, contextMenuFileIndex.value)
    } else if (action === 'moveToPane') {
      // 移动文件到其他窗格
      const targetPaneId = payload as string
      const targetPane = editorGroupRef.value?.panes.find(p => p.id === targetPaneId)
      if (!targetPane || !pane || contextMenuFileIndex.value < 0) return
      
      const file = pane.openFiles[contextMenuFileIndex.value]
      if (!file) return
      
      // 检查目标窗格是否已有该文件
      const existingIndex = targetPane.openFiles.findIndex(f => f.node.path === file.node.path)
      if (existingIndex !== -1) {
        // 如果已存在，直接激活
        targetPane.activeFileIndex = existingIndex
        editorGroupRef.value?.setActivePane(targetPaneId)
      } else {
        // 复制文件到目标窗格
        targetPane.openFiles.push({ ...file })
        targetPane.activeFileIndex = targetPane.openFiles.length - 1
        editorGroupRef.value?.setActivePane(targetPaneId)
      }
      
      // 从源窗格删除文件
      pane.openFiles.splice(contextMenuFileIndex.value, 1)
      if (pane.openFiles.length === 0) {
        pane.activeFileIndex = -1
      } else if (contextMenuFileIndex.value === pane.activeFileIndex) {
        pane.activeFileIndex = Math.min(contextMenuFileIndex.value, pane.openFiles.length - 1)
      } else if (contextMenuFileIndex.value < pane.activeFileIndex) {
        pane.activeFileIndex--
      }
    } else if (action === 'closeAll') {
      if (pane.openFiles.some(f => f.hasUnsavedChanges)) {
        const confirmed = await showConfirmDialog(
          '有文件包含未保存的更改，是否关闭？',
          '⚠️ 未保存的更改',
          'warning'
        )
        if (!confirmed) return
      }
      pane.openFiles = []
      pane.activeFileIndex = -1
      
      // 如果窗格为空且有多个窗格，自动删除该窗格（与逐个删除文件行为保持一致）
      if (editorGroupRef.value && editorGroupRef.value.panes.length > 1) {
        // 延迟一下执行，确保 UI 更新
        setTimeout(() => {
          editorGroupRef.value?.closePane(contextMenuPaneId.value)
        }, 100)
      }
    } else if (action === 'closeOthers') {
      const keepFile = pane.openFiles[contextMenuFileIndex.value]
      if (!keepFile) return
      
      const others = pane.openFiles.filter((_, i) => i !== contextMenuFileIndex.value)
      if (others.some(f => f.hasUnsavedChanges)) {
        const confirmed = await showConfirmDialog(
          '其他文件包含未保存的更改，是否关闭？',
          '⚠️ 未保存的更改',
          'warning'
        )
        if (!confirmed) return
      }
      
      pane.openFiles = [keepFile]
      pane.activeFileIndex = 0
    }
  } else if (contextMenuType.value === 'tree') {
    if (action === 'createFile') {
      createDialogType.value = 'file'
      createDialogMode.value = 'create'
      createDialogInitialValue.value = ''
      createDialogVisible.value = true
    } else if (action === 'createFolder') {
      createDialogType.value = 'folder'
      createDialogMode.value = 'create'
      createDialogInitialValue.value = ''
      createDialogVisible.value = true
    } else if (action === 'rename') {
      if (!treeContextMenuNode.value) return
      createDialogType.value = treeContextMenuNode.value.isDirectory ? 'folder' : 'file'
      createDialogMode.value = 'rename'
      createDialogInitialValue.value = treeContextMenuNode.value.name
      createDialogVisible.value = true
    } else if (action === 'delete') {
      if (!treeContextMenuNode.value) return

      const node = treeContextMenuNode.value
      const confirmed = await showConfirmDialog(
        node.isDirectory
          ? `确定要删除文件夹 "${node.name}" 吗？该操作将递归删除其下所有内容，且不可恢复。`
          : `确定要删除文件 "${node.name}" 吗？该操作不可恢复。`,
        '🗑️ 删除确认',
        'danger'
      )
      if (!confirmed) return

      try {
        const result = await deletePath(node.path)
        if (!result.success) {
          alert(result.message || '删除失败')
          return
        }

        await closeOpenedFilesUnderPath(node.path)

        if (leftPanelActiveTab.value === 'dependencies' && activeDependencyId.value) {
          invalidateDependencyFileTree(activeDependencyId.value)
          await loadDependencyFileTree(activeDependencyId.value)
        } else {
          await loadFileTree()
        }
      } catch (error) {
        logger.error('删除失败:', error)
        alert(`删除失败: ${error}`)
      }
    } else if (action === 'copyPath') {
      if (treeContextMenuNode.value) {
        navigator.clipboard.writeText(treeContextMenuNode.value.path).catch(err => {
          console.error('无法复制路径: ', err)
        })
      } else if (projectPath.value) {
        // 如果是在根目录空白处点击，复制项目路径
        navigator.clipboard.writeText(projectPath.value).catch(err => {
          console.error('无法复制路径: ', err)
        })
      }
    } else if (action === 'showInExplorer') {
      const targetPath = treeContextMenuNode.value ? treeContextMenuNode.value.path : projectPath.value
      if (targetPath) {
        // 如果是文件，打开父目录；如果是目录，直接打开
        // 由于 openFolder 目前只负责打开，对于文件，我们尝试获取其父目录
        if (treeContextMenuNode.value && !treeContextMenuNode.value.isDirectory) {
          const lastSepIndex = Math.max(targetPath.lastIndexOf('/'), targetPath.lastIndexOf('\\'))
          if (lastSepIndex > 0) {
             openFolder(targetPath.substring(0, lastSepIndex))
          } else {
             openFolder(targetPath)
          }
        } else {
          openFolder(targetPath)
        }
      }
    } else if (action === 'previewMap') {
      if (!isProjectMapFolder(treeContextMenuNode.value)) return
      await openProjectMapPreview(projectPath.value)
    }
  }
  hideContextMenu()
}

// 创建文件/文件夹
async function handleCreateConfirm(name: string, useBom: boolean = false) {
  if (createDialogMode.value === 'rename') {
    if (!treeContextMenuNode.value) return
    
    const oldPath = treeContextMenuNode.value.path
    // 获取父目录
    const lastSepIndex = Math.max(oldPath.lastIndexOf('/'), oldPath.lastIndexOf('\\'))
    const parentPath = lastSepIndex > 0 ? oldPath.substring(0, lastSepIndex) : oldPath
    const newPath = `${parentPath}\\${name}` // 假设是 Windows 分隔符，或者应该检测系统
    
    try {
      const result = await renamePath(oldPath, newPath)
      if (result.success) {
        await loadFileTree()
        createDialogVisible.value = false
      } else {
        alert(result.message || '重命名失败')
      }
    } catch (error) {
      logger.error('重命名失败:', error)
      alert(`重命名失败: ${error}`)
    }
    return
  }

  let parentPath: string
  if (treeContextMenuNode.value) {
    parentPath = treeContextMenuNode.value.isDirectory 
      ? treeContextMenuNode.value.path 
      : treeContextMenuNode.value.path.substring(0, treeContextMenuNode.value.path.lastIndexOf('\\'))
  } else if (selectedNode.value) {
    parentPath = selectedNode.value.isDirectory 
      ? selectedNode.value.path 
      : selectedNode.value.path.substring(0, selectedNode.value.path.lastIndexOf('\\'))
  } else {
    parentPath = projectPath.value
  }
  const targetPath = `${parentPath}\\${name}`
  try {
    let result
    if (createDialogType.value === 'file') {
      result = await createFile(targetPath, '', useBom)
    } else {
      result = await createFolder(targetPath)
    }
    if (result.success) {
      await loadFileTree()
      createDialogVisible.value = false
    } else {
      alert(result.message || '创建失败')
    }
  } catch (error) {
    logger.error('创建失败:', error)
    alert(`创建失败: ${error}`)
  }
}

// 返回主界面
async function goBack() {
  const hasUnsaved = editorGroupRef.value?.panes.some(pane => 
    pane.openFiles.some((f: any) => f.hasUnsavedChanges)
  )
  if (hasUnsaved) {
    const confirmed = await showConfirmDialog(
      '有文件包含未保存的更改，是否放弃所有更改？',
      '⚠️ 未保存的更改',
      'warning'
    )
    if (!confirmed) {
      return
    }
  }
  router.push('/')
}

// 打开依赖项管理对话框（从工具栏）
/**
 * 打开 Modifier 速查表小窗口
 */
async function openModifierSheet() {
  try {
    // 检查是否已经存在该窗口
    const existingWindow = await WebviewWindow.getByLabel('modifier-sheet')
    if (existingWindow) {
      await existingWindow.setFocus()
      return
    }

    // 创建新窗口
    const webview = new WebviewWindow('modifier-sheet', {
      url: '/modifier-sheet',
      title: 'Modifier 速查表',
      width: 600,
      height: 800,
      resizable: true,
      minWidth: 400,
      minHeight: 500,
      alwaysOnTop: true, // 速查表通常需要置顶方便查看
    })

    webview.once('tauri://created', () => {
      console.log('Modifier 速查表窗口已创建')
    })

    webview.once('tauri://error', (e) => {
      console.error('创建 Modifier 速查表窗口失败:', e)
    })
  } catch (error) {
    logger.error('打开 Modifier 速查表失败:', error)
    alert('无法打开 Modifier 速查表: ' + error)
  }
}

// 预览跳转函数（使用 usePreviewNavigation 模块）

async function handleJumpToMioFromPreview(sourcePaneId: string, sourceFilePath: string, traitId: string, line: number) {
  await jumpFromMioPreview(editorGroupRef.value, sourcePaneId, sourceFilePath, traitId, line, handleOpenFile)
}


// 处理编辑器右键菜单操作
async function handleEditorContextMenuAction(action: string, paneId: string) {
  if (!editorGroupRef.value) return
  
  const pane = editorGroupRef.value.panes.find(p => p.id === paneId)
  if (!pane) return
  
  const paneRef = (editorGroupRef.value as any).paneRefs?.get?.(paneId)
  if (!paneRef) return
  
  const editorMethods = paneRef.getEditorMethods?.()
  if (!editorMethods) return
  
  switch (action) {
    case 'selectAll':
      // 全选文本
      if (editorMethods.selectAll) {
        editorMethods.selectAll()
      }
      break
      
    case 'copy':
      // 复制选中文本到剪贴板
      try {
        const selectedText = editorMethods.getSelectedText?.() || ''
        if (selectedText) {
          await navigator.clipboard.writeText(selectedText)
        }
      } catch (error) {
        console.error('复制失败:', error)
      }
      break
      
    case 'cut':
      // 剪切选中文本
      try {
        const selectedText = editorMethods.cutSelection?.() || ''
        if (selectedText) {
          await navigator.clipboard.writeText(selectedText)
        }
      } catch (error) {
        console.error('剪切失败:', error)
      }
      break
      
    case 'paste':
      // 粘贴剪贴板内容
      try {
        const clipboardText = await navigator.clipboard.readText()
        if (clipboardText) {
          editorMethods.insertText?.(clipboardText)
        }
      } catch (error) {
        console.error('粘贴失败:', error)
      }
      break

    case 'insertIdeaTemplate':
    case 'insertTagTemplate':
    case 'insertBopTemplate':
      // 使用 useEditorTemplates 模块插入模板
      handleInsertTemplate(action, pane, editorMethods as EditorMethods)
      break
  }
}

// 处理打包
async function handlePackageProject(fileName: string) {
  if (!projectPath.value || !packageDialogRef.value) return
  
  // 开始打包
  packageDialogRef.value.startPacking()
  
  try {
    // 导入 API
    const { packProject } = await import('../api/tauri')
    
    // 执行打包
    const result = await packProject({
      projectPath: projectPath.value,
      outputName: fileName,
      excludeDependencies: true
    })
    
    // 显示结果
    packageDialogRef.value.finishPacking(result)
  } catch (error) {
    logger.error('打包失败:', error)
    packageDialogRef.value.finishPacking({
      success: false,
      message: `打包失败: ${error}`
    })
  }
}

// 右侧面板活动标签页
// 启动游戏
async function handleLaunchGame() {
  if (isLaunchingGame.value) return
  
  isLaunchingGame.value = true
  
  try {
    const result = await launchGame()
    
    // 最少显示 500ms 的加载状态，让用户看到反馈
    await new Promise(resolve => setTimeout(resolve, 500))
    
    if (result.success) {
      console.log('游戏启动成功:', result.message)
    } else {
      alert(`启动游戏失败: ${result.message}`)
    }
  } catch (error) {
    logger.error('启动游戏失败:', error)
    alert(`启动游戏失败: ${error}`)
  } finally {
    isLaunchingGame.value = false
  }
}

// 处理错误变化
function handleErrorsChange(_paneId: string, errors: Array<{line: number, msg: string, type: string}>) {
  // 更新全局错误列表
  txtErrors.value = errors
}

// 处理搜索
async function handlePerformSearch() {
  if (searchScope.value === 'dependencies') {
    // 搜索所有启用的依赖项
    const enabledDependencies = dependencies.value.filter(dep => dep.enabled)
    if (enabledDependencies.length === 0) {
      searchResults.value = []
      return
    }
    
    // 清空现有结果
    searchResults.value = []
    isSearching.value = true
    
    try {
      // 遍历所有依赖项，执行搜索
      for (let i = 0; i < enabledDependencies.length; i++) {
        const dep = enabledDependencies[i]
        // 第一个依赖项不追加（清空现有结果），后续依赖项追加
        await performSearch(dep.path, i > 0)
      }
    } finally {
      isSearching.value = false
    }
  } else {
    // 搜索项目或游戏目录
    const searchPath = searchScope.value === 'project' ? projectPath.value : gameDirectory.value
    if (searchPath) {
      performSearch(searchPath)
    }
  }
}



// 跳转到上一个错误
// 切换自动保存

function handleContentChange(paneId: string, content: string) {
  syncPreviewContent(paneId, content)
}

function handleNextError() {
  jumpToNextError()
}

function handlePreviousError() {
  jumpToPreviousError()
}

// 键盘快捷键
useKeyboardShortcuts({
  save: () => {
    // 保存当前活动窗格的文件
    if (editorGroupRef.value) {
      editorGroupRef.value.saveCurrentFile()
    }
  },
  undo: () => {},
  redo: () => {},
  search: () => {
    // 打开右侧边栏并切换到搜索标签页
    rightPanelExpanded.value = true
    rightPanelActiveTab.value = 'search'
  },
  nextError: handleNextError,
  previousError: handlePreviousError,
  toggleTheme: toggleThemePanel,
  toggleIconPanel: toggleIconPanel
})


// 生命周期
onMounted(async () => {
  markStartupStep('startup:editor-mounted', '编辑器页面挂载完成')
  const settingsSnapshot = await loadSettingsSnapshot()
  await Promise.all([
    loadThemeFromSettings(settingsSnapshot),
    loadIconSetFromSettings(settingsSnapshot),
    loadInitialSettings(settingsSnapshot)
  ])
  markStartupStep('startup:editor-theme-loaded', '编辑器主题加载完成')
  markStartupStep('startup:editor-icons-loaded', '编辑器图标集加载完成')
  markStartupStep('startup:editor-settings-loaded', '编辑器基础设置加载完成')
  projectPath.value = route.query.path as string || ''
  document.addEventListener('click', hideContextMenu)
  if (projectPath.value) {
    dependencyManager.setProjectPath(projectPath.value)
    await Promise.all([
      loadProjectInfo(),
      loadFileTree()
    ])
    markStartupStep('startup:editor-project-info-loaded', '编辑器项目信息加载完成')
    markStartupStep('startup:editor-file-tree-loaded', '编辑器文件树加载完成')
    markStartupStep('startup:editor-shell-ready', '编辑器首屏骨架就绪')
    // 启动目录树自动刷新
    startFileTreeAutoRefresh()
    void loadEditorBackgroundData(settingsSnapshot)
  } else {
    loading.value = false
  }
})

// 组件卸载时清理
onUnmounted(() => {
  stopFileTreeAutoRefresh()
  document.removeEventListener('click', hideContextMenu)
})
</script>

<template>
  <EditorWorkspaceShell>
    <!-- 顶部工具栏 -->
    <EditorToolbar
      :project-name="projectInfo?.name"
      :right-panel-expanded="rightPanelExpanded"
      :is-launching-game="isLaunchingGame"
      :tag-count="tagList.length"
      :idea-count="ideaList.length"
      :auto-save="autoSave"
      :plugin-toolbar-items="pluginToolbarItems"
      @go-back="goBack"
      @toggle-right-panel="toggleRightPanel"
      @launch-game="handleLaunchGame"
      @manage-dependencies="openDependenciesFromToolbar"
      @toggle-loading-monitor="toggleLoadingMonitor"
      @package-project="openPackageDialog"
      @toggle-auto-save="toggleAutoSave"
      @open-modifier-sheet="openModifierSheet"
      @plugin-toolbar-click="handlePluginToolbarClick"
    />

    <!-- 主内容区域 -->
    <div class="flex-1 flex overflow-hidden">
      <!-- 左侧文件树面板 -->
      <div
        class="ui-island flex-shrink-0 rounded-xl my-2 ml-2 flex flex-col overflow-hidden"
        :style="{ width: leftPanelWidth + 'px' }"
      >
        <!-- 左侧面板标签栏 -->
        <LeftPanelTabs
          :active-tab="leftPanelActiveTab"
          :active-dependency-id="activeDependencyId"
          :dependencies="dependencies"
          @switch-to-project="handleSwitchToProject"
          @switch-to-dependency="handleSwitchToDependency"
          @switch-to-plugins="handleSwitchToPluginsTab"
          @manage-dependencies="handleManageDependencies"
        />
        
        <!-- 文件树内容 -->
        <div class="flex-1 overflow-y-auto p-2" @contextmenu.prevent="handleShowTreeContextMenu($event, null)">
          <h3 class="text-hoi4-text font-bold mb-2 text-sm">
            {{ leftPanelActiveTab === 'project' ? '项目文件' : leftPanelActiveTab === 'dependencies' ? '依赖项文件' : '插件' }}
          </h3>
          <!-- 文件树切换过渡效果 -->
          <Transition name="sidebar-fade-slide" mode="out-in">
            <!-- 项目文件树 -->
            <div v-if="leftPanelActiveTab === 'project'" :key="'project'">
              <div v-if="loading" class="text-hoi4-text-dim text-sm p-2">加载中...</div>
              <div v-else-if="fileTree.length === 0" class="text-hoi4-text-dim text-sm p-2">无文件</div>
              <div v-else>
                <FileTreeNode
                  v-for="node in fileTree"
                  :key="node.path"
                  :node="node"
                  :level="0"
                  :selected-path="selectedNode?.path"
                  @toggle="toggleFolder"
                  @open-file="handleOpenFile"
                  @contextmenu="(e, n) => handleShowTreeContextMenu(e, n)"
                />
              </div>
            </div>

            <!-- 依赖项文件树 -->
            <div v-else-if="leftPanelActiveTab === 'dependencies' && activeDependencyId" :key="activeDependencyId">
              <div v-if="!hasActiveDependencyTree" class="text-hoi4-text-dim text-sm p-2">
                加载中...
              </div>
              <div v-else-if="activeDependencyTree.length === 0" class="text-hoi4-text-dim text-sm p-2">
                无文件
              </div>
              <div v-else>
                <FileTreeNode
                  v-for="node in activeDependencyTree"
                  :key="node.path"
                  :node="node"
                  :level="0"
                  :selected-path="selectedNode?.path"
                  @toggle="toggleFolder"
                  @open-file="handleOpenFile"
                  @contextmenu="(e, n) => handleShowTreeContextMenu(e, n)"
                />
              </div>
            </div>

            <div v-else-if="leftPanelActiveTab === 'plugins'" :key="'plugins'" class="h-full overflow-hidden flex flex-col">
              <div class="p-2 ui-separator-bottom flex items-center gap-2 overflow-x-auto">
                <button
                  v-for="p in pluginLeftPanels"
                  :key="p.uid"
                  class="px-2 py-1 rounded text-xs flex-shrink-0"
                  :class="p.uid === activeLeftPluginPanelUid ? 'bg-hoi4-accent text-hoi4-text' : 'bg-hoi4-border/40 text-hoi4-text-dim hover:text-hoi4-text hover:bg-hoi4-border/60'"
                  @click="activeLeftPluginPanelUid = p.uid"
                  :title="p.title"
                >
                  {{ p.title }}
                </button>
              </div>
              <div class="flex-1 overflow-hidden">
                <div v-if="pluginLeftPanels.length === 0" class="p-3 text-hoi4-text-dim text-sm">暂无插件面板</div>
                <PluginIframeHost
                  v-else
                  :entry-file-path="(pluginLeftPanels.find(x => x.uid === activeLeftPluginPanelUid) || pluginLeftPanels[0]).entryFilePath"
                  :plugin-id="(pluginLeftPanels.find(x => x.uid === activeLeftPluginPanelUid) || pluginLeftPanels[0]).pluginId"
                  :plugin-name="(pluginLeftPanels.find(x => x.uid === activeLeftPluginPanelUid) || pluginLeftPanels[0]).pluginName"
                  :side="(pluginLeftPanels.find(x => x.uid === activeLeftPluginPanelUid) || pluginLeftPanels[0]).side"
                  :panel-id="(pluginLeftPanels.find(x => x.uid === activeLeftPluginPanelUid) || pluginLeftPanels[0]).panelId"
                  :panel-title="(pluginLeftPanels.find(x => x.uid === activeLeftPluginPanelUid) || pluginLeftPanels[0]).title"
                  :allowed-commands="(pluginLeftPanels.find(x => x.uid === activeLeftPluginPanelUid) || pluginLeftPanels[0]).allowedCommands"
                />
              </div>
            </div>
          </Transition>
        </div>
      </div>

      <!-- 左侧拖动条 -->
      <div
        class="w-1 bg-hoi4-border hover:bg-hoi4-accent cursor-col-resize flex-shrink-0"
        @mousedown="startResizeLeft"
      ></div>

      <!-- 中间编辑区域 - EditorGroup -->
      <EditorGroup
        ref="editorGroupRef"
        :project-path="projectPath"
        :game-directory="gameDirectory"
        :dependency-roots="enabledDependencyRoots"
        :auto-save="autoSave"
        :disable-error-handling="disableErrorHandling"
        @open-file="handleOpenFile"
        @context-menu="showFileTabContextMenu"
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
      @content-change="handleContentChange"
      />

      <!-- 右侧拖动条 -->
      <div
        v-if="rightPanelExpanded"
        class="w-1 bg-hoi4-border hover:bg-hoi4-accent cursor-col-resize flex-shrink-0"
        @mousedown="startResizeRight"
      ></div>

      <!-- 右侧面板 -->
      <RightPanel
        v-if="rightPanelExpanded"
        :project-info="projectInfo"
        :game-directory="gameDirectory"
        :game-file-tree="gameFileTree"
        :is-loading-game-tree="isLoadingGameTree"
        :txt-errors="txtErrors"
        :width="rightPanelWidth"
        :search-query="searchQuery"
        :search-results="searchResults"
        :is-searching="isSearching"
        :search-case-sensitive="searchCaseSensitive"
        :search-regex="searchRegex"
        :search-scope="searchScope"
        :include-all-files="includeAllFiles"
        :project-path="projectPath"
        :plugin-panels="pluginRightPanels"
        v-model:activePluginPanelUid="activeRightPluginPanelUid"
        v-model:active-tab="rightPanelActiveTab"
        @close="toggleRightPanel"
        @jumpToError="jumpToError"
        @toggleGameFolder="toggleGameFolder"
        @openFile="handleOpenFile"
        @update:search-query="searchQuery = $event"
        @update:search-case-sensitive="searchCaseSensitive = $event"
        @update:search-regex="searchRegex = $event"
        @update:search-scope="searchScope = $event as 'project' | 'game' | 'dependencies'"
        @update:include-all-files="includeAllFiles = $event"
        @perform-search="handlePerformSearch"
        @perform-replace="handlePerformReplace"
        @jumpToSearchResult="handleJumpToSearchResult"
      />
    </div>

    <!-- 右键菜单 -->
    <ContextMenu
      :visible="contextMenuVisible"
      :x="contextMenuX"
      :y="contextMenuY"
      :menu-type="contextMenuType"
      :can-split="(editorGroupRef?.panes.length || 0) < 3"
      :tree-node-path="treeContextMenuNode?.path"
      :tree-node-is-directory="treeContextMenuNode?.isDirectory"
      :project-root="projectPath"
      :available-panes="availablePanesForMove"
      @action="handleContextMenuAction"
      @close="hideContextMenu"
    />

    <!-- 创建对话框 -->
    <CreateDialog
      :visible="createDialogVisible"
      :type="createDialogType"
      :mode="createDialogMode"
      :initial-value="createDialogInitialValue"
      @confirm="handleCreateConfirm"
      @cancel="createDialogVisible = false"
    />

    <!-- 确认对话框 -->
    <ConfirmDialog
      :visible="confirmDialogVisible"
      :title="confirmDialogTitle"
      :message="confirmDialogMessage"
      :type="confirmDialogType"
      @confirm="handleConfirmDialogConfirm"
      @cancel="handleConfirmDialogCancel"
    />


    <!-- 依赖项管理对话框 -->
    <DependencyManager
      :visible="dependencyManagerVisible"
      :dependencies="dependencies"
      :is-loading="isDependencyLoading"
      @close="dependencyManagerVisible = false"
      @add="handleAddDependency"
      @remove="handleRemoveDependency"
      @toggle="handleToggleDependency"
    />

    <!-- 加载监控面板 -->
    <LoadingMonitor
      :visible="loadingMonitorVisible"
      :tags="tagList"
      :ideas="ideaList"
      :is-loading-tags="tagLoading"
      :is-loading-ideas="ideaLoading"
      @close="loadingMonitorVisible = false"
      @refresh-tags="handleRefreshTags"
      @refresh-ideas="handleRefreshIdeas"
    />

    <!-- 打包对话框 -->
    <PackageDialog
      ref="packageDialogRef"
      :visible="packageDialogVisible"
      :project-name="projectInfo?.name"
      @close="packageDialogVisible = false"
      @confirm="handlePackageProject"
    />

    <!-- 主题切换面板 -->
    
    <!-- 图标选择面板 -->
  </EditorWorkspaceShell>
</template>

<style scoped>
.cursor-col-resize {
  cursor: col-resize;
}
</style>
