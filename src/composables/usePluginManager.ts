import { computed, ref } from 'vue'
import { listInstalledPlugins, installPlugin, uninstallPlugin, type InstalledPlugin } from '../api/tauri'

export type PluginPanelRef = {
  uid: string
  pluginId: string
  pluginName: string
  side: 'left' | 'right'
  panelId: string
  title: string
  entryFilePath: string
  allowedCommands: string[]
}

export type PluginToolbarItem = {
  uid: string
  pluginId: string
  pluginName: string
  title: string
  open?: {
    side: 'left' | 'right'
    panelUid: string
  }
}

const installedPlugins = ref<InstalledPlugin[]>([])
const isLoadingPlugins = ref(false)
const lastError = ref<string | null>(null)

function normalizeCommandList(cmds: string[] | undefined | null): string[] {
  if (!cmds) return []
  return cmds
    .map(s => String(s || '').trim())
    .filter(s => !!s)
}

export function usePluginManager() {
  const leftPanels = computed<PluginPanelRef[]>(() => {
    const out: PluginPanelRef[] = []
    for (const p of installedPlugins.value) {
      const pluginId = p.about.id
      const pluginName = p.about.name
      const entryFilePath = p.entry_file_path
      const allowedCommands = normalizeCommandList(p.about.permissions?.commands)
      const panels = p.about.contributes?.left_sidebar || []
      for (const panel of panels) {
        const uid = `${pluginId}:left:${panel.id}`
        out.push({
          uid,
          pluginId,
          pluginName,
          side: 'left',
          panelId: panel.id,
          title: panel.title,
          entryFilePath,
          allowedCommands
        })
      }
    }
    return out
  })

  const rightPanels = computed<PluginPanelRef[]>(() => {
    const out: PluginPanelRef[] = []
    for (const p of installedPlugins.value) {
      const pluginId = p.about.id
      const pluginName = p.about.name
      const entryFilePath = p.entry_file_path
      const allowedCommands = normalizeCommandList(p.about.permissions?.commands)
      const panels = p.about.contributes?.right_sidebar || []
      for (const panel of panels) {
        const uid = `${pluginId}:right:${panel.id}`
        out.push({
          uid,
          pluginId,
          pluginName,
          side: 'right',
          panelId: panel.id,
          title: panel.title,
          entryFilePath,
          allowedCommands
        })
      }
    }
    return out
  })

  const toolbarItems = computed<PluginToolbarItem[]>(() => {
    const out: PluginToolbarItem[] = []
    for (const p of installedPlugins.value) {
      const pluginId = p.about.id
      const pluginName = p.about.name
      const items = p.about.contributes?.toolbar || []
      for (const it of items) {
        const uid = `${pluginId}:toolbar:${it.id}`
        const open = it.open
          ? {
              side: (it.open.side === 'left' ? 'left' : 'right') as 'left' | 'right',
              panelUid: `${pluginId}:${it.open.side === 'left' ? 'left' : 'right'}:${it.open.panel}`
            }
          : undefined
        out.push({
          uid,
          pluginId,
          pluginName,
          title: it.title,
          open
        })
      }
    }
    return out
  })

  async function refreshPlugins() {
    isLoadingPlugins.value = true
    lastError.value = null
    try {
      installedPlugins.value = await listInstalledPlugins()
    } catch (e: unknown) {
      lastError.value = e instanceof Error ? e.message : String(e)
    } finally {
      isLoadingPlugins.value = false
    }
  }

  async function installFromPath(path: string) {
    lastError.value = null
    try {
      await installPlugin(path)
      await refreshPlugins()
      return { success: true as const }
    } catch (e: unknown) {
      lastError.value = e instanceof Error ? e.message : String(e)
      return { success: false as const, message: lastError.value }
    }
  }

  async function uninstallById(pluginId: string) {
    lastError.value = null
    try {
      await uninstallPlugin(pluginId)
      await refreshPlugins()
      return { success: true as const }
    } catch (e: unknown) {
      lastError.value = e instanceof Error ? e.message : String(e)
      return { success: false as const, message: lastError.value }
    }
  }

  return {
    installedPlugins,
    leftPanels,
    rightPanels,
    toolbarItems,
    isLoadingPlugins,
    lastError,
    refreshPlugins,
    installFromPath,
    uninstallById
  }
}
