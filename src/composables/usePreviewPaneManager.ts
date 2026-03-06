import type { Ref } from 'vue'
import type { OpenFile } from './useFileManager'

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

export function usePreviewPaneManager(editorGroupRef: Ref<EditorGroupLike | null>) {
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
    syncPreviewContent
  }
}
