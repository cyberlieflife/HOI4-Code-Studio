import type { Ref } from 'vue'
import type { FileNode, OpenFile } from './useFileManager'

export type PreviewKind =
  | 'event'
  | 'gfx'
  | 'mio'
  | 'focus'
  | 'map'
  | 'gui'

export interface PaneLike {
  id: string
  openFiles: OpenFile[]
  activeFileIndex: number
}

export interface EditorGroupLike {
  panes: PaneLike[]
  activePaneId?: string
  setActivePane: (paneId: string) => void
  splitPane: (paneId: string, fileIndex?: number) => boolean
}

interface PreviewDefinition {
  titleSuffix: string
  contentFactory: (currentFile: OpenFile) => string
  applyFlags: (file: OpenFile) => void
}

const previewDefinitions: Record<PreviewKind, PreviewDefinition> = {
  event: {
    titleSuffix: ' - Event Preview',
    contentFactory: (currentFile) => currentFile.content,
    applyFlags: (file) => {
      file.isEventGraph = true
    }
  },
  gfx: {
    titleSuffix: ' - GFX Preview',
    contentFactory: (currentFile) => currentFile.content,
    applyFlags: (file) => {
      file.isGfxPreview = true
    }
  },
  mio: {
    titleSuffix: ' - MIO Preview',
    contentFactory: (currentFile) => currentFile.content,
    applyFlags: (file) => {
      file.isMioPreview = true
    }
  },
  focus: {
    titleSuffix: ' - Focus Preview',
    contentFactory: (currentFile) => currentFile.content,
    applyFlags: (file) => {
      file.isFocusTree = true
    }
  },
  map: {
    titleSuffix: ' - World Map',
    contentFactory: () => '',
    applyFlags: (file) => {
      file.isWorldMap = true
    }
  },
  gui: {
    titleSuffix: ' - GUI Preview',
    contentFactory: (currentFile) => currentFile.content,
    applyFlags: (file) => {
      file.isGuiPreview = true
    }
  }
}

function isPreviewHost(file: OpenFile | undefined) {
  return !!(
    file?.isEventGraph ||
    file?.isFocusTree ||
    file?.isWorldMap ||
    file?.isGuiPreview ||
    file?.isMioPreview ||
    file?.isGfxPreview
  )
}

function createPreviewFile(currentFile: OpenFile, kind: PreviewKind): OpenFile {
  const definition = previewDefinitions[kind]
  const previewFile: OpenFile = {
    node: {
      ...currentFile.node,
      name: `${currentFile.node.name}${definition.titleSuffix}`
    },
    content: definition.contentFactory(currentFile),
    hasUnsavedChanges: false,
    cursorLine: 1,
    cursorColumn: 1,
    isPreview: true,
    sourceFilePath: currentFile.node.path
  }

  definition.applyFlags(previewFile)
  return previewFile
}

function createStandalonePreviewFile(
  node: FileNode,
  kind: PreviewKind,
  sourceFilePath = node.path
): OpenFile {
  const definition = previewDefinitions[kind]
  const previewFile: OpenFile = {
    node: {
      ...node,
      isDirectory: false,
      name: `${node.name}${definition.titleSuffix}`
    },
    content: '',
    hasUnsavedChanges: false,
    cursorLine: 1,
    cursorColumn: 1,
    isPreview: true,
    sourceFilePath
  }

  definition.applyFlags(previewFile)
  return previewFile
}

export function usePreviewPaneManager(editorGroupRef: Ref<EditorGroupLike | null>) {
  function findExistingPreview(sourceFilePath: string, kind: PreviewKind) {
    const editorGroup = editorGroupRef.value
    if (!editorGroup) return null

    for (const pane of editorGroup.panes) {
      const fileIndex = pane.openFiles.findIndex((file) => {
        if (!file.isPreview || file.sourceFilePath !== sourceFilePath) {
          return false
        }

        if (kind === 'map') {
          return file.isWorldMap === true
        }

        return false
      })

      if (fileIndex !== -1) {
        return { pane, fileIndex }
      }
    }

    return null
  }

  function resolveBasePane(preferredPaneId?: string): PaneLike | null {
    const editorGroup = editorGroupRef.value
    if (!editorGroup) return null

    if (preferredPaneId) {
      const preferredPane = editorGroup.panes.find((pane) => pane.id === preferredPaneId)
      if (preferredPane) return preferredPane
    }

    if (editorGroup.activePaneId) {
      const activePane = editorGroup.panes.find((pane) => pane.id === editorGroup.activePaneId)
      if (activePane) return activePane
    }

    return editorGroup.panes[0] || null
  }

  function resolveTargetPane(basePane: PaneLike): PaneLike | null {
    const editorGroup = editorGroupRef.value
    if (!editorGroup) return null

    if (editorGroup.panes.length >= 2) {
      const previewPane = editorGroup.panes.find((pane) =>
        pane.openFiles.some((file) => isPreviewHost(file))
      )
      if (previewPane) {
        return previewPane
      }
    }

    if (basePane.openFiles.length === 0) {
      return basePane
    }

    const splitSuccess = editorGroup.splitPane(basePane.id)
    if (!splitSuccess) {
      return basePane
    }

    return editorGroup.panes[editorGroup.panes.length - 1] || null
  }

  async function openPreview(paneId: string, kind: PreviewKind) {
    const editorGroup = editorGroupRef.value
    if (!editorGroup) return

    const sourcePane = editorGroup.panes.find((pane) => pane.id === paneId)
    if (!sourcePane || sourcePane.activeFileIndex < 0) return

    const currentFile = sourcePane.openFiles[sourcePane.activeFileIndex]
    if (!currentFile) return

    let targetPane: PaneLike | null = null
    if (editorGroup.panes.length >= 2) {
      targetPane = editorGroup.panes.find((pane) =>
        pane.openFiles.some((file) => isPreviewHost(file))
      ) || null
    }

    if (!targetPane) {
      const splitSuccess = editorGroup.splitPane(paneId)
      if (!splitSuccess) return
      targetPane = editorGroup.panes[editorGroup.panes.length - 1] || null
    }

    if (!targetPane) return

    targetPane.openFiles.push(createPreviewFile(currentFile, kind))
    targetPane.activeFileIndex = targetPane.openFiles.length - 1
    editorGroup.setActivePane(targetPane.id)
  }

  async function openProjectMapPreview(projectPath: string) {
    const editorGroup = editorGroupRef.value
    if (!editorGroup) return

    const normalizedProjectPath = projectPath.replace(/[\\/]+$/, '')
    const sourceFilePath = `${normalizedProjectPath}/map`
    const existingPreview = findExistingPreview(sourceFilePath, 'map')
    if (existingPreview) {
      existingPreview.pane.activeFileIndex = existingPreview.fileIndex
      editorGroup.setActivePane(existingPreview.pane.id)
      return
    }

    const basePane = resolveBasePane()
    if (!basePane) return

    const targetPane = resolveTargetPane(basePane)
    if (!targetPane) return

    const previewNode: FileNode = {
      name: 'map',
      path: sourceFilePath,
      isDirectory: false
    }

    targetPane.openFiles.push(createStandalonePreviewFile(previewNode, 'map', sourceFilePath))
    targetPane.activeFileIndex = targetPane.openFiles.length - 1
    editorGroup.setActivePane(targetPane.id)
  }

  function syncPreviewContent(paneId: string, content: string) {
    const editorGroup = editorGroupRef.value
    if (!editorGroup) return

    const pane = editorGroup.panes.find((item) => item.id === paneId)
    if (!pane || pane.activeFileIndex < 0) return

    const currentFile = pane.openFiles[pane.activeFileIndex]
    if (!currentFile) return

    const currentFilePath = currentFile.node.path
    editorGroup.panes.forEach((targetPane) => {
      targetPane.openFiles.forEach((file) => {
        if (file.isPreview && file.sourceFilePath === currentFilePath) {
          file.content = content
        }
      })
    })
  }

  return {
    openPreview,
    openProjectMapPreview,
    syncPreviewContent
  }
}
