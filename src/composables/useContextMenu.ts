/**
 * 右键菜单 composable
 * 负责维护全局右键菜单的显示状态和上下文信息。
 */

import { ref } from 'vue'
import type { FileNode } from './useFileManager'
import type { SidebarSide } from './useEditorUiState'

export type ContextMenuType = 'file' | 'tree' | 'pane' | 'sidebar'

// 全局状态，使用单例模式保证同一时间只有一个右键菜单实例。
const contextMenuVisible = ref(false)
const contextMenuX = ref(0)
const contextMenuY = ref(0)
const contextMenuType = ref<ContextMenuType>('file')
const contextMenuPaneId = ref('')
const contextMenuFileIndex = ref(-1)
const treeContextMenuNode = ref<FileNode | null>(null)
const treeContextMenuSide = ref<SidebarSide>('left')
const sidebarContextItemKey = ref('')
const sidebarContextSide = ref<SidebarSide>('left')
const lastContextMenuTime = ref(0)

/**
 * 右键菜单 composable
 * 通过单例共享状态，避免多个菜单同时出现。
 */
export function useContextMenu() {
  /**
   * 显示文件标签页右键菜单。
   */
  function showFileTabContextMenu(event: MouseEvent, paneId: string, fileIndex: number) {
    contextMenuPaneId.value = paneId
    contextMenuFileIndex.value = fileIndex
    contextMenuX.value = event.clientX
    contextMenuY.value = event.clientY
    contextMenuType.value = 'pane'
    contextMenuVisible.value = true
  }

  /**
   * 显示文件树右键菜单。
   * 当 `node` 为空时表示点击了文件树空白区域。
   */
  function showTreeContextMenu(event: MouseEvent, node: FileNode | null = null) {
    // 避免节点点击冒泡后立刻触发空白区域菜单。
    const now = Date.now()
    if (node === null && now - lastContextMenuTime.value < 100) {
      return
    }

    if (node) {
      lastContextMenuTime.value = now
      treeContextMenuNode.value = node
    } else {
      treeContextMenuNode.value = null
    }

    contextMenuX.value = event.clientX
    contextMenuY.value = event.clientY
    contextMenuType.value = 'tree'
    contextMenuVisible.value = true
  }

  function showTreeContextMenuForSide(event: MouseEvent, side: SidebarSide, node: FileNode | null = null) {
    treeContextMenuSide.value = side
    showTreeContextMenu(event, node)
  }

  function showSidebarTabContextMenu(event: MouseEvent, key: string, side: SidebarSide) {
    sidebarContextItemKey.value = key
    sidebarContextSide.value = side
    contextMenuX.value = event.clientX
    contextMenuY.value = event.clientY
    contextMenuType.value = 'sidebar'
    contextMenuVisible.value = true
  }

  /**
   * 隐藏右键菜单。
   */
  function hideContextMenu() {
    contextMenuVisible.value = false
  }

  return {
    contextMenuVisible,
    contextMenuX,
    contextMenuY,
    contextMenuType,
    contextMenuPaneId,
    contextMenuFileIndex,
    treeContextMenuNode,
    treeContextMenuSide,
    sidebarContextItemKey,
    sidebarContextSide,
    lastContextMenuTime,
    showFileTabContextMenu,
    showTreeContextMenu,
    showTreeContextMenuForSide,
    showSidebarTabContextMenu,
    hideContextMenu
  }
}
