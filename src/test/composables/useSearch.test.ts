import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useSearch, type SearchScope } from '../../../src/composables/useSearch'

vi.mock('../../../src/api/tauri', () => ({
  searchFiles: vi.fn()
}))

vi.mock('../../../src/utils/logger', () => ({
  logger: {
    error: vi.fn()
  }
}))

import { searchFiles } from '../../../src/api/tauri'
import { logger } from '../../../src/utils/logger'

describe('useSearch', () => {
  let search: ReturnType<typeof useSearch>

  beforeEach(() => {
    vi.clearAllMocks()
    search = useSearch()
  })

  it('正确初始化状态', () => {
    expect(search.searchQuery.value).toBe('')
    expect(search.searchResults.value).toEqual([])
    expect(search.isSearching.value).toBe(false)
    expect(search.searchCaseSensitive.value).toBe(false)
    expect(search.searchRegex.value).toBe(false)
    expect(search.searchScope.value).toBe('project')
    expect(search.includeAllFiles.value).toBe(false)
  })

  it('可以设置搜索选项', () => {
    search.searchQuery.value = 'test query'
    search.searchCaseSensitive.value = true
    search.searchRegex.value = true
    search.searchScope.value = 'game' as SearchScope
    search.includeAllFiles.value = true

    expect(search.searchQuery.value).toBe('test query')
    expect(search.searchCaseSensitive.value).toBe(true)
    expect(search.searchRegex.value).toBe(true)
    expect(search.searchScope.value).toBe('game')
    expect(search.includeAllFiles.value).toBe(true)

    search.searchScope.value = 'currentFile' as SearchScope
    expect(search.searchScope.value).toBe('currentFile')
  })

  it('可以执行目录搜索', async () => {
    vi.mocked(searchFiles).mockResolvedValue({
      success: true,
      message: '',
      results: [
        {
          file_name: 'test1.txt',
          file_path: '/path/to/test1.txt',
          line: 1,
          content: 'Test line 1',
          match_start: 0,
          match_end: 4
        },
        {
          file_name: 'test2.txt',
          file_path: '/path/to/test2.txt',
          line: 3,
          content: 'Another test line',
          match_start: 8,
          match_end: 12
        }
      ]
    })

    search.searchQuery.value = 'test'
    await search.performSearch('/test/path')

    expect(searchFiles).toHaveBeenCalledWith('/test/path', 'test', false, false, false)
    expect(search.isSearching.value).toBe(false)
    expect(search.searchResults.value).toHaveLength(2)
    expect(search.searchResults.value[0]).toMatchObject({
      file: {
        name: 'test1.txt',
        path: '/path/to/test1.txt',
        isDirectory: false
      },
      line: 1,
      content: 'Test line 1',
      matchStart: 0,
      matchEnd: 4
    })
  })

  it('空查询时会清空结果且不调用接口', async () => {
    search.searchQuery.value = ''
    search.searchResults.value = [
      {
        file: {
          name: 'test.txt',
          path: '/test.txt',
          isDirectory: false
        },
        line: 1,
        content: 'test',
        matchStart: 0,
        matchEnd: 4
      }
    ]

    await search.performSearch('/test/path')

    expect(searchFiles).not.toHaveBeenCalled()
    expect(search.searchResults.value).toEqual([])
  })

  it('空路径时记录错误', async () => {
    search.searchQuery.value = 'test'

    await search.performSearch('')

    expect(searchFiles).not.toHaveBeenCalled()
    expect(logger.error).toHaveBeenCalledTimes(1)
  })

  it('接口失败时记录错误', async () => {
    vi.mocked(searchFiles).mockResolvedValue({
      success: false,
      message: '搜索失败',
      results: []
    })

    search.searchQuery.value = 'test'
    await search.performSearch('/test/path')

    expect(searchFiles).toHaveBeenCalledTimes(1)
    expect(logger.error).toHaveBeenCalledTimes(1)
    expect(search.isSearching.value).toBe(false)
  })

  it('支持追加搜索结果', async () => {
    vi.mocked(searchFiles)
      .mockResolvedValueOnce({
        success: true,
        message: '',
        results: [
          {
            file_name: 'test1.txt',
            file_path: '/path/to/test1.txt',
            line: 1,
            content: 'Test line 1',
            match_start: 0,
            match_end: 4
          }
        ]
      })
      .mockResolvedValueOnce({
        success: true,
        message: '',
        results: [
          {
            file_name: 'test2.txt',
            file_path: '/path/to/test2.txt',
            line: 3,
            content: 'Another test line',
            match_start: 8,
            match_end: 12
          }
        ]
      })

    search.searchQuery.value = 'test'
    await search.performSearch('/test/path1')
    await search.performSearch('/test/path2', true)

    expect(search.searchResults.value).toHaveLength(2)
    expect(search.searchResults.value[0].file.name).toBe('test1.txt')
    expect(search.searchResults.value[1].file.name).toBe('test2.txt')
  })

  it('支持在已打开文件中搜索', async () => {
    search.searchQuery.value = 'test'

    await search.performSearchInFiles([
      {
        name: 'open.txt',
        path: '/open.txt',
        content: 'first test line\nsecond TEST line'
      }
    ])

    expect(search.searchResults.value).toHaveLength(2)
    expect(search.searchResults.value[0]).toMatchObject({
      line: 1,
      content: 'first test line',
      matchStart: 6,
      matchEnd: 10
    })
    expect(search.searchResults.value[1]).toMatchObject({
      line: 2,
      content: 'second TEST line',
      matchStart: 7,
      matchEnd: 11
    })
  })

  it('支持在已打开文件中使用正则搜索', async () => {
    search.searchQuery.value = 'te.t'
    search.searchRegex.value = true

    await search.performSearchInFiles([
      {
        name: 'regex.txt',
        path: '/regex.txt',
        content: 'test text\ntoast'
      }
    ])

    expect(search.searchResults.value).toHaveLength(2)
    expect(search.searchResults.value[0].content).toBe('test text')
    expect(search.searchResults.value[1].content).toBe('test text')
    expect(search.searchResults.value[0].matchStart).toBe(0)
    expect(search.searchResults.value[1].matchStart).toBe(5)
  })

  it('可以清空搜索结果', () => {
    search.searchQuery.value = 'test'
    search.searchResults.value = [
      {
        file: {
          name: 'test1.txt',
          path: '/path/to/test1.txt',
          isDirectory: false
        },
        line: 1,
        content: 'Test line',
        matchStart: 0,
        matchEnd: 4
      }
    ]

    search.clearResults()

    expect(search.searchResults.value).toEqual([])
    expect(search.searchQuery.value).toBe('')
  })

  it('可以跳转到搜索结果', () => {
    const mockDispatch = vi.fn()
    const mockFocus = vi.fn()
    const mockEditorView = {
      dispatch: mockDispatch,
      focus: mockFocus,
      state: {
        doc: {
          lines: 10,
          line: (lineNumber: number) => ({
            from: (lineNumber - 1) * 20,
            length: 20
          })
        }
      }
    }

    search.jumpToResult(
      {
        file: {
          name: 'test.txt',
          path: '/path/to/test.txt',
          isDirectory: false
        },
        line: 5,
        content: 'Test line content',
        matchStart: 0,
        matchEnd: 4
      },
      mockEditorView as any
    )

    expect(mockDispatch).toHaveBeenCalledWith({
      selection: { anchor: 80, head: 84 },
      scrollIntoView: true
    })
    expect(mockFocus).toHaveBeenCalledTimes(1)
  })
})
