import { ref, type Ref } from 'vue'
import { buildDirectoryTreeFast } from '../api/tauri'
import type { FileNode } from './useFileManager'
import {
  collectExpandedPaths,
  mergeExpandedChildren,
  restoreExpandedState
} from '../utils/fileTreeState'
import { convertRustFileNode } from '../utils/fileUtils'
import { logger } from '../utils/logger'

interface RootSyncOptions {
  projectPath: string
  gameDirectory?: string
  dependencyPaths: string[]
}

interface ProjectFileTreeLoaderOptions {
  projectPath: Ref<string>
  gameDirectory: Ref<string>
  getEnabledDependencyPaths: () => string[]
  syncRoots: (options: RootSyncOptions) => void
}

export function useProjectFileTreeLoader(options: ProjectFileTreeLoaderOptions) {
  const loading = ref(true)
  const fileTree = ref<FileNode[]>([])

  async function loadFileTree() {
    if (!options.projectPath.value) return

    const oldTree = fileTree.value
    const expandedPaths = collectExpandedPaths(oldTree)

    try {
      const result = await buildDirectoryTreeFast(options.projectPath.value, 3)
      if (result.success && result.tree) {
        const newTree = result.tree.map(convertRustFileNode)
        mergeExpandedChildren(oldTree, newTree, expandedPaths)
        restoreExpandedState(newTree, expandedPaths)
        fileTree.value = newTree
      }

      options.syncRoots({
        projectPath: options.projectPath.value,
        gameDirectory: options.gameDirectory.value,
        dependencyPaths: options.getEnabledDependencyPaths()
      })
    } catch (error) {
      logger.error('加载文件树失败:', error)
    } finally {
      loading.value = false
    }
  }

  return {
    loading,
    fileTree,
    loadFileTree
  }
}
