import { ref, type Ref } from 'vue'
import type { Dependency } from '../types/dependency'
import type { FileNode } from './useFileManager'
import { buildDirectoryTreeFast } from '../api/tauri'
import { convertRustFileNode } from '../utils/fileUtils'
import { logger } from '../utils/logger'

export function useDependencyTreeCache(dependencies: Ref<Dependency[]>) {
  const dependencyFileTrees = ref<Map<string, FileNode[]>>(new Map())

  const hasDependencyTree = (dependencyId?: string) =>
    !!dependencyId && dependencyFileTrees.value.has(dependencyId)

  const getDependencyTree = (dependencyId?: string) =>
    dependencyId ? dependencyFileTrees.value.get(dependencyId) || [] : []

  async function loadDependencyFileTree(dependencyId: string) {
    const dependency = dependencies.value.find((dep) => dep.id === dependencyId)
    if (!dependency || dependencyFileTrees.value.has(dependencyId)) {
      return
    }

    try {
      const result = await buildDirectoryTreeFast(dependency.path, 3)
      if (result.success && result.tree) {
        const next = new Map(dependencyFileTrees.value)
        next.set(dependencyId, result.tree.map(convertRustFileNode))
        dependencyFileTrees.value = next
      }
    } catch (error) {
      logger.error('加载依赖项文件树失败:', error)
    }
  }

  function invalidateDependencyFileTree(dependencyId: string) {
    if (!dependencyFileTrees.value.has(dependencyId)) {
      return
    }

    const next = new Map(dependencyFileTrees.value)
    next.delete(dependencyId)
    dependencyFileTrees.value = next
  }

  function clearDependencyFileTrees() {
    dependencyFileTrees.value = new Map()
  }

  return {
    dependencyFileTrees,
    hasDependencyTree,
    getDependencyTree,
    loadDependencyFileTree,
    invalidateDependencyFileTree,
    clearDependencyFileTrees
  }
}
