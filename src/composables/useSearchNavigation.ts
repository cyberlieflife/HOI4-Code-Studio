import type { Ref } from 'vue'
import type { FileNode } from './useFileManager'

interface EditorGroupLike {
  jumpToSearchResult: (result: any) => void
}

interface SearchResultLike {
  file?: {
    path?: string
    name?: string
  }
  line?: number
  matchStart?: number
  matchEnd?: number
  content?: string
  [key: string]: any
}

interface OpenFileLike {
  (node: FileNode, paneId?: string, jumpInfo?: any): Promise<void>
}

export function useSearchNavigation(
  editorGroupRef: Ref<EditorGroupLike | null>,
  openFile: OpenFileLike
) {
  async function jumpToSearchResult(result: SearchResultLike) {
    const targetPath = result?.file?.path
    if (!targetPath) return

    const name = result?.file?.name || (targetPath.split(/[\\\/]/).pop() || targetPath)
    const node: FileNode = { name, path: targetPath, isDirectory: false }

    await openFile(node, undefined, result)

    setTimeout(() => {
      if (editorGroupRef.value && result?.line) {
        editorGroupRef.value.jumpToSearchResult(result)
        setTimeout(() => {
          if (editorGroupRef.value && result?.line) {
            editorGroupRef.value.jumpToSearchResult(result)
          }
        }, 100)
      }
    }, 1000)
  }

  return {
    jumpToSearchResult
  }
}
