import type { Ref } from 'vue'

export interface EditorError {
  line: number
  msg: string
  type: string
}

interface OpenFileLike {
  cursorLine: number
}

interface PaneLike {
  id: string
  activeFileIndex: number
  openFiles: OpenFileLike[]
}

interface EditorGroupLike {
  activePaneId: string
  panes: PaneLike[]
  jumpToErrorLine: (line: number) => void
}

export function useEditorErrorNavigation(
  editorGroupRef: Ref<EditorGroupLike | null>,
  errors: Ref<EditorError[]>
) {
  function jumpToError(error: EditorError) {
    if (!editorGroupRef.value) return
    editorGroupRef.value.jumpToErrorLine(error.line)
  }

  function getActiveCursorLine() {
    const editorGroup = editorGroupRef.value
    if (!editorGroup) return null

    const activePane = editorGroup.panes.find((pane) => pane.id === editorGroup.activePaneId)
    if (!activePane || activePane.activeFileIndex === -1) return null

    const activeFile = activePane.openFiles[activePane.activeFileIndex]
    return activeFile?.cursorLine ?? null
  }

  function jumpToNextError() {
    const currentLine = getActiveCursorLine()
    if (currentLine === null) return

    const sortedErrors = [...errors.value].sort((a, b) => a.line - b.line)
    if (sortedErrors.length === 0) return

    const nextError = sortedErrors.find((error) => error.line > currentLine)
    jumpToError(nextError || sortedErrors[0])
  }

  function jumpToPreviousError() {
    const currentLine = getActiveCursorLine()
    if (currentLine === null) return

    const sortedErrors = [...errors.value].sort((a, b) => a.line - b.line)
    if (sortedErrors.length === 0) return

    const previousError = [...sortedErrors].reverse().find((error) => error.line < currentLine)
    jumpToError(previousError || sortedErrors[sortedErrors.length - 1])
  }

  return {
    jumpToError,
    jumpToNextError,
    jumpToPreviousError
  }
}
