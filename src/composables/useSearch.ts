import { ref } from 'vue'
import { searchFiles, type SearchResult as ApiSearchResult } from '../api/tauri'
import { escapeRegExp } from '../utils/fileUtils'
import { logger } from '../utils/logger'
import type { EditorView } from '@codemirror/view'

/**
 * 搜索结果接口
 */
export interface SearchResult {
  file: {
    name: string
    path: string
    isDirectory: boolean
  }
  line: number
  content: string
  matchStart: number
  matchEnd: number
}

export interface SearchSourceFile {
  name: string
  path: string
  content: string
}

export type SearchScope = 'project' | 'game' | 'dependencies' | 'currentFile' | 'openFiles'

/**
 * 搜索功能 Composable
 * 管理项目和游戏目录的搜索功能
 */
export function useSearch() {
  const searchQuery = ref('')
  const searchResults = ref<SearchResult[]>([])
  const isSearching = ref(false)
  const searchCaseSensitive = ref(false)
  const searchRegex = ref(false)
  const searchScope = ref<SearchScope>('project')
  const includeAllFiles = ref(false)
  
  /**
   * 将 API 搜索结果转换为本地格式
   */
  function convertApiSearchResult(apiResult: ApiSearchResult): SearchResult {
    return {
      file: {
        name: apiResult.file_name,
        path: apiResult.file_path,
        isDirectory: false
      },
      line: apiResult.line,
      content: apiResult.content,
      matchStart: apiResult.match_start,
      matchEnd: apiResult.match_end
    }
  }

  function createSearchPattern(): RegExp {
    const flags = searchCaseSensitive.value ? 'g' : 'gi'
    return searchRegex.value
      ? new RegExp(searchQuery.value, flags)
      : new RegExp(escapeRegExp(searchQuery.value), flags)
  }

  function createFileSearchResult(
    file: SearchSourceFile,
    line: number,
    content: string,
    matchStart: number,
    matchEnd: number
  ): SearchResult {
    return {
      file: {
        name: file.name,
        path: file.path,
        isDirectory: false
      },
      line,
      content,
      matchStart,
      matchEnd
    }
  }
  
  /**
   * 执行搜索
   * @param searchPath 搜索路径
   * @param append 是否追加搜索结果（默认：false）
   */
  async function performSearch(searchPath: string, append: boolean = false) {
    if (!searchQuery.value.trim()) {
      searchResults.value = []
      return
    }
    
    if (!searchPath) {
      logger.error('搜索路径未设置')
      return
    }
    
    if (!append) {
      isSearching.value = true
      searchResults.value = []
    }
    
    try {
      const result = await searchFiles(
        searchPath,
        searchQuery.value,
        searchCaseSensitive.value,
        searchRegex.value,
        includeAllFiles.value
      )
      
      if (result.success) {
        const newResults = result.results.map(convertApiSearchResult)
        if (append) {
          // 追加搜索结果
          searchResults.value = [...searchResults.value, ...newResults]
        } else {
          // 替换搜索结果
          searchResults.value = newResults
        }
      } else {
        logger.error(`搜索失败: ${result.message}`)
      }
    } catch (error) {
      logger.error('搜索失败:', error)
    } finally {
      if (!append) {
        isSearching.value = false
      }
    }
  }

  async function performSearchInFiles(files: SearchSourceFile[]) {
    if (!searchQuery.value.trim()) {
      searchResults.value = []
      return
    }

    isSearching.value = true
    searchResults.value = []

    try {
      const pattern = createSearchPattern()
      const results: SearchResult[] = []

      for (const file of files) {
        const lines = file.content.split(/\r?\n/)

        for (let index = 0; index < lines.length; index++) {
          const lineContent = lines[index]
          const linePattern = new RegExp(pattern.source, pattern.flags)
          let match: RegExpExecArray | null

          while ((match = linePattern.exec(lineContent)) !== null) {
            const matchedText = match[0] ?? ''
            const matchStart = match.index
            const matchEnd = matchStart + matchedText.length

            results.push(createFileSearchResult(file, index + 1, lineContent, matchStart, matchEnd))

            if (matchedText.length === 0) {
              linePattern.lastIndex += 1
            }
          }
        }
      }

      searchResults.value = results
    } catch (error) {
      logger.error('鎼滅储澶辫触:', error)
      searchResults.value = []
    } finally {
      isSearching.value = false
    }
  }
  
  /**
   * 跳转到搜索结果（CodeMirror 6 版本）
   */
  function jumpToResult(result: SearchResult, editorView: EditorView) {
    if (!editorView) return
    
    try {
      // 计算目标行
      const targetLine = Math.max(1, Math.min(result.line, editorView.state.doc.lines))
      const line = editorView.state.doc.line(targetLine)
      
      // 计算字符位置
      const matchStart = Math.max(0, Math.min(result.matchStart, line.length))
      const matchEnd = Math.max(matchStart, Math.min(result.matchEnd, line.length))
      const pos = line.from + matchStart
      const endPos = line.from + matchEnd
      
      // 跳转并选中
      editorView.dispatch({
        selection: { anchor: pos, head: endPos },
        scrollIntoView: true
      })
      editorView.focus()
    } catch (error) {
      logger.error('跳转到搜索结果失败:', error)
    }
  }
  
  /**
   * 清空搜索结果
   */
  function clearResults() {
    searchResults.value = []
    searchQuery.value = ''
  }
  
  return {
    searchQuery,
    searchResults,
    isSearching,
    searchCaseSensitive,
    searchRegex,
    searchScope,
    includeAllFiles,
    performSearch,
    performSearchInFiles,
    jumpToResult,
    clearResults
  }
}
