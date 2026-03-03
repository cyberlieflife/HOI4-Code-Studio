/**
 * 右键菜单 Composable
 * 管理右键菜单的显示状态和位置
 */

import { ref } from 'vue'
import type { FileNode } from './useFileManager'

export type ContextMenuType = 'file' | 'tree' | 'pane'

// 全局状态（单例模式）
const contextMenuVisible = ref(false)
const contextMenuX = ref(0)
const contextMenuY = ref(0)
const contextMenuType = ref<ContextMenuType>('file')
const contextMenuPaneId = ref('')
const contextMenuFileIndex = ref(-1)
const treeContextMenuNode = ref<FileNode | null>(null)
const lastContextMenuTime = ref(0)

/**
 * 右键菜单 Composable
 * 使用单例模式，确保全局只有一个右键菜单实例
 */
export function useContextMenu() {
  /**
   * 显示文件标签页右键菜单
   * @param event 鼠标事件
   * @param paneId 窗格 ID
   * @param fileIndex 文件索引
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
   * 显示文件树右键菜单
   * @param event 鼠标事件
   * @param node 文件节点（可选，为空时表示点击背景）
   */
  function showTreeContextMenu(event: MouseEvent, node: FileNode | null = null) {
    // 如果是背景点击（node=null），且距离上次有效点击时间很近，则忽略（视为冒泡）
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

  /**
   * 隐藏右键菜单
   */
  function hideContextMenu() {
    contextMenuVisible.value = false
  }

  return {
    // 状态
    contextMenuVisible,
    contextMenuX,
    contextMenuY,
    contextMenuType,
    contextMenuPaneId,
    contextMenuFileIndex,
    treeContextMenuNode,
    lastContextMenuTime,
    // 方法
    showFileTabContextMenu,
    showTreeContextMenu,
    hideContextMenu
  }
}
