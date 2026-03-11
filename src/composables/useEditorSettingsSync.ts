import { ref, watch, type Ref } from 'vue'
import { buildDirectoryTreeFast, loadSettings, saveSettings, type Settings } from '../api/tauri'
import type { Dependency } from '../types/dependency'
import type { FileNode } from './useFileManager'
import { ensureIdeaRegistry } from './useIdeaRegistry'
import { convertRustFileNode } from '../utils/fileUtils'
import { logger } from '../utils/logger'

interface RootSyncOptions {
  projectPath: string
  gameDirectory?: string
  dependencyPaths: string[]
}

interface EditorSettingsSyncOptions {
  projectPath: Ref<string>
  dependencies: Ref<Dependency[]>
  refreshTags: () => Promise<unknown>
  loadFontConfigFromSettings: (settings: Record<string, unknown>) => void
  syncRoots: (options: RootSyncOptions) => void
}

interface LoadGameDirectoryOptions {
  refreshRegistries?: boolean
  settings?: Settings
}

export function useEditorSettingsSync(options: EditorSettingsSyncOptions) {
  const gameDirectory = ref('')
  const gameFileTree = ref<FileNode[]>([])
  const isLoadingGameTree = ref(false)
  const autoSave = ref(true)
  const disableErrorHandling = ref(false)

  function getEnabledDependencyPaths() {
    return options.dependencies.value.filter((dep) => dep.enabled).map((dep) => dep.path)
  }

  async function loadGameFileTree() {
    if (!gameDirectory.value) return

    isLoadingGameTree.value = true
    try {
      const result = await buildDirectoryTreeFast(gameDirectory.value, 3)
      if (result.success && result.tree) {
        gameFileTree.value = result.tree.map(convertRustFileNode)
      }
    } catch (error) {
      logger.error('加载游戏目录文件树失败:', error)
    } finally {
      isLoadingGameTree.value = false
    }
  }

  async function loadInitialSettings(settings?: Settings) {
    const data = settings ?? await (async () => {
      const settingsResult = await loadSettings()
      if (!settingsResult.success || !settingsResult.data) return null
      return settingsResult.data as Settings
    })()

    if (!data) return

    autoSave.value = data.autoSave !== false
    disableErrorHandling.value = data.disableErrorHandling === true
    options.loadFontConfigFromSettings(data)
  }

  async function loadGameDirectory(loadOptions: LoadGameDirectoryOptions = {}) {
    const { refreshRegistries = true, settings } = loadOptions

    try {
      const dependencyPaths = getEnabledDependencyPaths()
      const data = settings ?? await (async () => {
        const result = await loadSettings()
        if (!result.success || !result.data || typeof result.data !== 'object') return null
        return result.data as Settings
      })()

      if (data && 'gameDirectory' in data) {
        gameDirectory.value = String(data.gameDirectory || '')
        autoSave.value = data.autoSave === false ? false : true
        disableErrorHandling.value = data.disableErrorHandling === true

        options.syncRoots({
          projectPath: options.projectPath.value,
          gameDirectory: gameDirectory.value,
          dependencyPaths
        })
        await loadGameFileTree()
        if (refreshRegistries) {
          await options.refreshTags()
          await ensureIdeaRegistry()
        }
        return
      }

      options.syncRoots({
        projectPath: options.projectPath.value,
        dependencyPaths
      })
      if (refreshRegistries) {
        await options.refreshTags()
        await ensureIdeaRegistry()
      }
    } catch (error) {
      logger.error('加载游戏目录设置失败:', error)
    }
  }

  async function toggleAutoSave() {
    autoSave.value = !autoSave.value
  }

  watch(autoSave, async (newValue) => {
    try {
      const result = await loadSettings()
      if (result.success && result.data) {
        await saveSettings({
          ...result.data,
          autoSave: newValue
        })
      }
    } catch (error) {
      logger.error('保存自动保存设置失败:', error)
    }
  })

  return {
    gameDirectory,
    gameFileTree,
    isLoadingGameTree,
    autoSave,
    disableErrorHandling,
    loadInitialSettings,
    loadGameDirectory,
    loadGameFileTree,
    toggleAutoSave
  }
}
