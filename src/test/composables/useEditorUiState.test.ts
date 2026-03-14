import { describe, expect, it } from 'vitest'
import { useEditorUiState } from '../../../src/composables/useEditorUiState'

describe('useEditorUiState', () => {
  it('应该切换终端面板显示状态', () => {
    const editorUiState = useEditorUiState()

    expect(editorUiState.terminalVisible.value).toBe(false)

    editorUiState.toggleTerminalPanel()
    expect(editorUiState.terminalVisible.value).toBe(true)

    editorUiState.toggleTerminalPanel()
    expect(editorUiState.terminalVisible.value).toBe(false)
  })
})
