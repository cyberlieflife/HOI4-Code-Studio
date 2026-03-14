import { ref } from 'vue'
import type { Settings } from '../api/tauri'
import type { Dependency } from '../types/dependency'

export type SidebarSide = 'left' | 'right'
export type BuiltinSidebarTab = 'project' | 'search' | 'info' | 'game' | 'errors' | 'ai'
type CreateDialogType = 'file' | 'folder'
type CreateDialogMode = 'create' | 'rename'

export interface SidebarMovableItem {
  key: string
  kind: 'builtin' | 'dependency'
  builtinId?: BuiltinSidebarTab
  dependencyId?: string
}

export type SidebarView =
  | { type: 'builtin'; builtinId: BuiltinSidebarTab }
  | { type: 'dependency'; dependencyId: string }
  | { type: 'plugins' }

interface SidebarLayoutRecord {
  kind: 'builtin' | 'dependency'
  id: string
}

interface SidebarLayoutSnapshot {
  left: SidebarLayoutRecord[]
  right: SidebarLayoutRecord[]
}

const LEFT_DEFAULT_BUILTINS: BuiltinSidebarTab[] = ['project', 'search']
const RIGHT_DEFAULT_BUILTINS: BuiltinSidebarTab[] = ['info', 'game', 'errors', 'ai']

function createBuiltinItem(builtinId: BuiltinSidebarTab): SidebarMovableItem {
  return {
    key: `builtin:${builtinId}`,
    kind: 'builtin',
    builtinId
  }
}

function createDependencyItem(dependencyId: string): SidebarMovableItem {
  return {
    key: `dependency:${dependencyId}`,
    kind: 'dependency',
    dependencyId
  }
}

function buildDefaultSidebarItems(side: SidebarSide): SidebarMovableItem[] {
  const builtins = side === 'left' ? LEFT_DEFAULT_BUILTINS : RIGHT_DEFAULT_BUILTINS
  return builtins.map(createBuiltinItem)
}

function isSidebarLayoutSnapshot(value: unknown): value is SidebarLayoutSnapshot {
  if (!value || typeof value !== 'object') return false
  const layout = value as Record<string, unknown>
  return Array.isArray(layout.left) && Array.isArray(layout.right)
}

function normalizeStoredRecords(records: SidebarLayoutRecord[] | unknown, dependencies: Dependency[]) {
  if (!Array.isArray(records)) return []

  const availableBuiltinIds = new Set<BuiltinSidebarTab>([
    ...LEFT_DEFAULT_BUILTINS,
    ...RIGHT_DEFAULT_BUILTINS
  ])
  const availableDependencyIds = new Set(dependencies.map(dep => dep.id))
  const seenKeys = new Set<string>()
  const normalized: SidebarMovableItem[] = []

  for (const record of records) {
    if (!record || typeof record !== 'object') continue
    const item = record as Partial<SidebarLayoutRecord>
    if (item.kind === 'builtin' && typeof item.id === 'string' && availableBuiltinIds.has(item.id as BuiltinSidebarTab)) {
      const builtinId = item.id as BuiltinSidebarTab
      const key = `builtin:${builtinId}`
      if (seenKeys.has(key)) continue
      seenKeys.add(key)
      normalized.push(createBuiltinItem(builtinId))
      continue
    }

    if (item.kind === 'dependency' && typeof item.id === 'string' && availableDependencyIds.has(item.id)) {
      const key = `dependency:${item.id}`
      if (seenKeys.has(key)) continue
      seenKeys.add(key)
      normalized.push(createDependencyItem(item.id))
    }
  }

  return normalized
}

function normalizeSidebarLayout(rawLayout: unknown, dependencies: Dependency[]): { left: SidebarMovableItem[]; right: SidebarMovableItem[] } {
  const defaultLeft = buildDefaultSidebarItems('left')
  const defaultRight = buildDefaultSidebarItems('right')

  if (!isSidebarLayoutSnapshot(rawLayout)) {
    return {
      left: [...defaultLeft, ...dependencies.map(dep => createDependencyItem(dep.id))],
      right: defaultRight
    }
  }

  const left = normalizeStoredRecords(rawLayout.left, dependencies)
  const right = normalizeStoredRecords(rawLayout.right, dependencies)
  const seenKeys = new Set([...left, ...right].map(item => item.key))

  for (const builtinId of LEFT_DEFAULT_BUILTINS) {
    const key = `builtin:${builtinId}`
    if (!seenKeys.has(key)) {
      left.push(createBuiltinItem(builtinId))
      seenKeys.add(key)
    }
  }

  for (const builtinId of RIGHT_DEFAULT_BUILTINS) {
    const key = `builtin:${builtinId}`
    if (!seenKeys.has(key)) {
      right.push(createBuiltinItem(builtinId))
      seenKeys.add(key)
    }
  }

  for (const dependency of dependencies) {
    const key = `dependency:${dependency.id}`
    if (!seenKeys.has(key)) {
      left.push(createDependencyItem(dependency.id))
      seenKeys.add(key)
    }
  }

  return { left, right }
}

function cloneView(view: SidebarView): SidebarView {
  if (view.type === 'builtin') {
    return { type: 'builtin', builtinId: view.builtinId }
  }
  if (view.type === 'dependency') {
    return { type: 'dependency', dependencyId: view.dependencyId }
  }
  return { type: 'plugins' }
}

function getItemKeyFromView(view: SidebarView): string | null {
  if (view.type === 'builtin') {
    return `builtin:${view.builtinId}`
  }
  if (view.type === 'dependency') {
    return `dependency:${view.dependencyId}`
  }
  return null
}

function itemToView(item: SidebarMovableItem): SidebarView {
  if (item.kind === 'builtin' && item.builtinId) {
    return { type: 'builtin', builtinId: item.builtinId }
  }
  return { type: 'dependency', dependencyId: item.dependencyId || '' }
}

function serializeItems(items: SidebarMovableItem[]): SidebarLayoutRecord[] {
  return items.reduce<SidebarLayoutRecord[]>((records, item) => {
    if (item.kind === 'builtin' && item.builtinId) {
      records.push({ kind: 'builtin', id: item.builtinId })
      return records
    }
    if (item.kind === 'dependency' && item.dependencyId) {
      records.push({ kind: 'dependency', id: item.dependencyId })
    }
    return records
  }, [])
}

export function useEditorUiState() {
  const rightPanelExpanded = ref(false)
  const terminalVisible = ref(false)

  const createDialogVisible = ref(false)
  const createDialogType = ref<CreateDialogType>('file')
  const createDialogMode = ref<CreateDialogMode>('create')
  const createDialogInitialValue = ref('')

  const leftSidebarItems = ref<SidebarMovableItem[]>(buildDefaultSidebarItems('left'))
  const rightSidebarItems = ref<SidebarMovableItem[]>(buildDefaultSidebarItems('right'))
  const leftSidebarView = ref<SidebarView>({ type: 'builtin', builtinId: 'project' })
  const rightSidebarView = ref<SidebarView>({ type: 'builtin', builtinId: 'info' })

  const activeDependencyId = ref<string | undefined>(undefined)
  const dependencyManagerVisible = ref(false)
  const activeLeftPluginPanelUid = ref('')

  const loadingMonitorVisible = ref(false)
  const packageDialogVisible = ref(false)

  const activeRightPluginPanelUid = ref('')

  function getSidebarItemsRef(side: SidebarSide) {
    return side === 'left' ? leftSidebarItems : rightSidebarItems
  }

  function getSidebarViewRef(side: SidebarSide) {
    return side === 'left' ? leftSidebarView : rightSidebarView
  }

  function ensureSidebarView(side: SidebarSide) {
    const items = getSidebarItemsRef(side).value
    const viewRef = getSidebarViewRef(side)
    const activeKey = getItemKeyFromView(viewRef.value)

    if (activeKey && items.some(item => item.key === activeKey)) {
      return
    }

    if (items.length > 0) {
      viewRef.value = itemToView(items[0])
      if (items[0].kind === 'dependency' && items[0].dependencyId) {
        activeDependencyId.value = items[0].dependencyId
      }
      return
    }

    viewRef.value = { type: 'plugins' }
  }

  function setActiveView(side: SidebarSide, view: SidebarView) {
    const viewRef = getSidebarViewRef(side)
    viewRef.value = cloneView(view)

    if (view.type === 'dependency') {
      activeDependencyId.value = view.dependencyId
    } else if (activeDependencyId.value) {
      const activeDependencyExists = [...leftSidebarItems.value, ...rightSidebarItems.value].some(
        item => item.kind === 'dependency' && item.dependencyId === activeDependencyId.value
      )
      if (!activeDependencyExists) {
        activeDependencyId.value = undefined
      }
    }

    if (side === 'right' && view.type !== 'plugins') {
      rightPanelExpanded.value = true
    }
  }

  function getItemSide(key: string): SidebarSide | null {
    if (leftSidebarItems.value.some(item => item.key === key)) return 'left'
    if (rightSidebarItems.value.some(item => item.key === key)) return 'right'
    return null
  }

  function getBuiltinSide(builtinId: BuiltinSidebarTab) {
    return getItemSide(`builtin:${builtinId}`)
  }

  function getDependencySide(dependencyId: string) {
    return getItemSide(`dependency:${dependencyId}`)
  }

  function activateItemByKey(side: SidebarSide, key: string) {
    const item = getSidebarItemsRef(side).value.find(entry => entry.key === key)
    if (!item) return
    setActiveView(side, itemToView(item))
  }

  function handleSwitchToProject() {
    const side = getBuiltinSide('project')
    if (!side) return
    setActiveView(side, { type: 'builtin', builtinId: 'project' })
  }

  function handleSwitchToDependency(id: string) {
    const side = getDependencySide(id)
    if (!side) return
    setActiveView(side, { type: 'dependency', dependencyId: id })
  }

  function handleSwitchToSearch() {
    const side = getBuiltinSide('search')
    if (!side) return
    setActiveView(side, { type: 'builtin', builtinId: 'search' })
  }

  function handleSwitchToPlugins(defaultPanelUid?: string) {
    leftSidebarView.value = { type: 'plugins' }
    activeDependencyId.value = undefined
    if (!activeLeftPluginPanelUid.value && defaultPanelUid) {
      activeLeftPluginPanelUid.value = defaultPanelUid
    }
  }

  function handleSwitchToRightPlugins(defaultPanelUid?: string) {
    rightPanelExpanded.value = true
    rightSidebarView.value = { type: 'plugins' }
    if (!activeRightPluginPanelUid.value && defaultPanelUid) {
      activeRightPluginPanelUid.value = defaultPanelUid
    }
  }

  function handleManageDependencies() {
    dependencyManagerVisible.value = true
  }

  function openDependenciesFromToolbar() {
    dependencyManagerVisible.value = true
  }

  function toggleLoadingMonitor() {
    loadingMonitorVisible.value = !loadingMonitorVisible.value
  }

  function openPackageDialog() {
    packageDialogVisible.value = true
  }

  function toggleRightPanel() {
    rightPanelExpanded.value = !rightPanelExpanded.value
  }

  function toggleTerminalPanel() {
    terminalVisible.value = !terminalVisible.value
  }

  function handlePluginToolbarClick(_uid: string, open?: { side: 'left' | 'right'; panelUid: string }) {
    if (!open) return

    if (open.side === 'left') {
      handleSwitchToPlugins(open.panelUid)
      return
    }

    handleSwitchToRightPlugins(open.panelUid)
  }

  function syncSidebarLayout(dependencies: Dependency[], settings?: Settings) {
    const layout = normalizeSidebarLayout(settings?.sidebarLayout, dependencies)
    leftSidebarItems.value = layout.left
    rightSidebarItems.value = layout.right
    ensureSidebarView('left')
    ensureSidebarView('right')
  }

  function moveSidebarItem(key: string, targetSide: SidebarSide) {
    const sourceSide = getItemSide(key)
    if (!sourceSide || sourceSide === targetSide) return

    const sourceItems = [...getSidebarItemsRef(sourceSide).value]
    const targetItems = [...getSidebarItemsRef(targetSide).value]
    const sourceIndex = sourceItems.findIndex(item => item.key === key)
    if (sourceIndex === -1) return

    const [item] = sourceItems.splice(sourceIndex, 1)
    targetItems.push(item)

    getSidebarItemsRef(sourceSide).value = sourceItems
    getSidebarItemsRef(targetSide).value = targetItems

    if (getItemKeyFromView(getSidebarViewRef(sourceSide).value) === key) {
      ensureSidebarView(sourceSide)
    }

    setActiveView(targetSide, itemToView(item))
    ensureSidebarView(sourceSide)
  }

  function reorderSidebarItems(
    side: SidebarSide,
    draggedKey: string,
    targetKey?: string,
    position: 'before' | 'after' | 'end' = 'before'
  ) {
    const items = [...getSidebarItemsRef(side).value]
    const draggedIndex = items.findIndex(item => item.key === draggedKey)
    if (draggedIndex === -1) return

    const [draggedItem] = items.splice(draggedIndex, 1)

    if (!targetKey || position === 'end') {
      items.push(draggedItem)
      getSidebarItemsRef(side).value = items
      return
    }

    const targetIndex = items.findIndex(item => item.key === targetKey)
    if (targetIndex === -1) {
      items.push(draggedItem)
      getSidebarItemsRef(side).value = items
      return
    }

    const insertIndex = position === 'after' ? targetIndex + 1 : targetIndex
    items.splice(insertIndex, 0, draggedItem)
    getSidebarItemsRef(side).value = items
  }

  function serializeSidebarLayout(): SidebarLayoutSnapshot {
    return {
      left: serializeItems(leftSidebarItems.value),
      right: serializeItems(rightSidebarItems.value)
    }
  }

  return {
    rightPanelExpanded,
    terminalVisible,
    createDialogVisible,
    createDialogType,
    createDialogMode,
    createDialogInitialValue,
    leftSidebarItems,
    rightSidebarItems,
    leftSidebarView,
    rightSidebarView,
    activeDependencyId,
    dependencyManagerVisible,
    activeLeftPluginPanelUid,
    loadingMonitorVisible,
    packageDialogVisible,
    activeRightPluginPanelUid,
    activateItemByKey,
    handleSwitchToProject,
    handleSwitchToDependency,
    handleSwitchToSearch,
    handleSwitchToPlugins,
    handleSwitchToRightPlugins,
    handleManageDependencies,
    openDependenciesFromToolbar,
    toggleLoadingMonitor,
    openPackageDialog,
    toggleRightPanel,
    toggleTerminalPanel,
    handlePluginToolbarClick,
    syncSidebarLayout,
    moveSidebarItem,
    reorderSidebarItems,
    serializeSidebarLayout
  }
}
