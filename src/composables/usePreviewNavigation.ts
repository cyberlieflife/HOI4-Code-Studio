/**
 * 预览跳转导航功能
 * 从预览面板跳转回源文件的指定行
 */

import { basename } from '../utils/fileUtils'

export interface FileNodeLike {
  name: string
  path: string
  isDirectory: boolean
}

export interface EditorGroupRefLike {
  panes: PaneInfo[]
  setActivePane: (paneId: string) => void
}

export interface PaneInfo {
  id: string
  openFiles: OpenFileInfo[]
  activeFileIndex: number
}

export interface OpenFileInfo {
  isFocusTree?: boolean
  isGfxPreview?: boolean
  isMioPreview?: boolean
}

export interface OpenFileFunc {
  (node: FileNodeLike, paneId?: string): Promise<void>
}

export interface PaneRefsLike {
  get: (paneId: string) => { jumpToLine?: (line: number) => void } | undefined
}

/**
 * 通用的预览跳转函数
 * @param editorGroupRef 编辑器组引用
 * @param sourcePaneId 源窗格ID
 * @param sourceFilePath 源文件路径
 * @param line 目标行号
 * @param openFileFunc 打开文件的函数
 * @param excludePredicate 排除特定类型预览的谓词
 */
export async function jumpFromPreview(
  editorGroupRef: EditorGroupRefLike | null,
  sourcePaneId: string,
  sourceFilePath: string,
  line: number,
  openFileFunc: OpenFileFunc,
  excludePredicate: (file: OpenFileInfo) => boolean
): Promise<void> {
  if (!editorGroupRef) return

  const panes = editorGroupRef.panes
  
  // 查找目标窗格：优先查找非预览窗格
  let targetPane = panes.find(p => {
    const active = p.openFiles[p.activeFileIndex]
    return !!active && !excludePredicate(active)
  })

  // 如果没找到，使用源窗格
  if (!targetPane) {
    targetPane = panes.find(p => p.id === sourcePaneId)
  }
  if (!targetPane) return

  editorGroupRef.setActivePane(targetPane.id)

  // 创建文件节点
  const node: FileNodeLike = {
    name: basename(sourceFilePath),
    path: sourceFilePath,
    isDirectory: false
  }

  // 打开文件
  await openFileFunc(node, targetPane.id)

  // 延迟跳转到指定行
  const paneId = targetPane.id
  setTimeout(() => {
    const paneRef = (editorGroupRef as unknown as { paneRefs?: PaneRefsLike })?.paneRefs
    const paneRefInstance = paneRef?.get?.(paneId)
    if (paneRefInstance?.jumpToLine) {
      paneRefInstance.jumpToLine(line)
    }
  }, 80)
}

/**
 * 从 FocusTree 预览跳转到源文件
 */
export async function jumpFromFocusPreview(
  editorGroupRef: EditorGroupRefLike | null,
  sourcePaneId: string,
  sourceFilePath: string,
  _focusId: string,
  line: number,
  openFileFunc: OpenFileFunc
): Promise<void> {
  return jumpFromPreview(
    editorGroupRef,
    sourcePaneId,
    sourceFilePath,
    line,
    openFileFunc,
    (file) => file.isFocusTree === true
  )
}

/**
 * 从 GFX 预览跳转到源文件
 */
export async function jumpFromGfxPreview(
  editorGroupRef: EditorGroupRefLike | null,
  sourcePaneId: string,
  sourceFilePath: string,
  line: number,
  openFileFunc: OpenFileFunc
): Promise<void> {
  return jumpFromPreview(
    editorGroupRef,
    sourcePaneId,
    sourceFilePath,
    line,
    openFileFunc,
    (file) => file.isGfxPreview === true
  )
}

/**
 * 从 MIO 预览跳转到源文件
 */
export async function jumpFromMioPreview(
  editorGroupRef: EditorGroupRefLike | null,
  sourcePaneId: string,
  sourceFilePath: string,
  _traitId: string,
  line: number,
  openFileFunc: OpenFileFunc
): Promise<void> {
  return jumpFromPreview(
    editorGroupRef,
    sourcePaneId,
    sourceFilePath,
    line,
    openFileFunc,
    (file) => file.isMioPreview === true
  )
}
