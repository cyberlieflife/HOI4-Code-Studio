import { ref } from 'vue'

type LeftPanelTab = 'project' | 'dependencies' | 'plugins'
type RightPanelTab = 'info' | 'game' | 'errors' | 'search' | 'ai' | 'plugins'
type CreateDialogType = 'file' | 'folder'
type CreateDialogMode = 'create' | 'rename'

export function useEditorUiState() {
  const rightPanelExpanded = ref(false)

  const createDialogVisible = ref(false)
  const createDialogType = ref<CreateDialogType>('file')
  const createDialogMode = ref<CreateDialogMode>('create')
  const createDialogInitialValue = ref('')

  const leftPanelActiveTab = ref<LeftPanelTab>('project')
  const activeDependencyId = ref<string | undefined>(undefined)
  const dependencyManagerVisible = ref(false)
  const activeLeftPluginPanelUid = ref('')

  const loadingMonitorVisible = ref(false)
  const packageDialogVisible = ref(false)

  const rightPanelActiveTab = ref<RightPanelTab>('info')
  const activeRightPluginPanelUid = ref('')

  function handleSwitchToProject() {
    leftPanelActiveTab.value = 'project'
    activeDependencyId.value = undefined
  }

  function handleSwitchToDependency(id: string) {
    leftPanelActiveTab.value = 'dependencies'
    activeDependencyId.value = id
  }

  function handleSwitchToPlugins(defaultPanelUid?: string) {
    leftPanelActiveTab.value = 'plugins'
    activeDependencyId.value = undefined
    if (!activeLeftPluginPanelUid.value && defaultPanelUid) {
      activeLeftPluginPanelUid.value = defaultPanelUid
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

  function handlePluginToolbarClick(_uid: string, open?: { side: 'left' | 'right'; panelUid: string }) {
    if (!open) return

    if (open.side === 'left') {
      handleSwitchToPlugins(open.panelUid)
      return
    }

    rightPanelExpanded.value = true
    rightPanelActiveTab.value = 'plugins'
    activeRightPluginPanelUid.value = open.panelUid
  }

  return {
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
    handleSwitchToDependency,
    handleSwitchToPlugins,
    handleManageDependencies,
    openDependenciesFromToolbar,
    toggleLoadingMonitor,
    openPackageDialog,
    toggleRightPanel,
    handlePluginToolbarClick
  }
}
