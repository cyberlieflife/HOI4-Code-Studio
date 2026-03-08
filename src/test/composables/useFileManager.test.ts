/**
 * 文件管理 Composable 测试
 */

import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { useFileManager } from '../../../src/composables/useFileManager'
import type { FileNode } from '../../../src/composables/useFileManager'

// 模拟 Tauri API
vi.mock('../../../src/api/tauri', () => ({
  readFileContent: vi.fn(),
  writeFileContent: vi.fn(),
  readImageAsBase64: vi.fn()
}))

import { readFileContent, writeFileContent, readImageAsBase64 } from '../../../src/api/tauri'

// 模拟 logger
vi.mock('../../../src/utils/logger', () => ({
  logger: {
    info: vi.fn(),
    error: vi.fn()
  }
}))

import { logger } from '../../../src/utils/logger'

describe('useFileManager', () => {
  let fileManager: ReturnType<typeof useFileManager>

  beforeEach(() => {
    vi.clearAllMocks()
    fileManager = useFileManager()
  })

  afterEach(() => {
    // 清理全局模拟
    vi.restoreAllMocks()
  })

  it('应该正确初始化状态', () => {
    expect(fileManager.openFiles.value).toEqual([])
    expect(fileManager.activeFileIndex.value).toBe(-1)
    expect(fileManager.currentFile.value).toBeNull()
    expect(fileManager.isLoadingFile.value).toBe(false)
  })

  it('应该能够检测图片文件', () => {
    // 由于 isImageFile 是内部函数，我们通过打开文件来间接测试
    // 这里我们测试文件管理器能够正确识别图片文件
    
    // 测试图片文件打开逻辑
    const mockImageNode: FileNode = {
      name: 'image.png',
      path: '/project/image.png',
      isDirectory: false
    }

    const mockTextNode: FileNode = {
      name: 'document.txt',
      path: '/project/document.txt',
      isDirectory: false
    }

    // 模拟图片文件读取
    vi.mocked(readImageAsBase64).mockResolvedValue({
      success: true,
      base64: 'test',
      mimeType: 'image/png'
    })

    // 模拟文本文件读取
    vi.mocked(readFileContent).mockResolvedValue({
      success: true,
      message: '',
      content: 'text content',
      encoding: 'UTF-8',
      is_binary: false,
      is_image: false
    })

    // 图片文件应该调用 readImageAsBase64
    fileManager.openFile(mockImageNode)
    expect(readImageAsBase64).toHaveBeenCalledWith('/project/image.png')

    // 文本文件应该调用 readFileContent
    fileManager.openFile(mockTextNode)
    expect(readFileContent).toHaveBeenCalledWith('/project/document.txt')
  })

  it('应该能够打开文本文件', async () => {
    const mockFileNode: FileNode = {
      name: 'test.txt',
      path: '/project/test.txt',
      isDirectory: false
    }

    const mockContent = 'Hello, World!'
    vi.mocked(readFileContent).mockResolvedValue({
      success: true,
      message: '',
      content: mockContent,
      encoding: 'UTF-8',
      is_binary: false,
      is_image: false
    })

    const onContentLoaded = vi.fn()
    const result = await fileManager.openFile(mockFileNode, onContentLoaded)

    expect(result).toBe(true)
    expect(fileManager.openFiles.value.length).toBe(1)
    expect(fileManager.activeFileIndex.value).toBe(0)
    expect(fileManager.currentFile.value).toStrictEqual(mockFileNode)
    
    const openedFile = fileManager.openFiles.value[0]
    expect(openedFile.node).toStrictEqual(mockFileNode)
    expect(openedFile.content).toBe(mockContent)
    expect(openedFile.hasUnsavedChanges).toBe(false)
    expect(openedFile.isImage).toBeUndefined()
    
    expect(onContentLoaded).toHaveBeenCalledWith(mockContent)
    // 只有当编码不是 UTF-8 时才会记录
    // 这里编码是 UTF-8，所以不会调用 logger.info
    expect(logger.info).not.toHaveBeenCalled()
  })

  it('应该能够打开图片文件', async () => {
    const mockFileNode: FileNode = {
      name: 'image.png',
      path: '/project/image.png',
      isDirectory: false
    }

    const mockBase64 = 'base64encodeddata'
    const mockMimeType = 'image/png'
    vi.mocked(readImageAsBase64).mockResolvedValue({
      success: true,
      base64: mockBase64,
      mimeType: mockMimeType
    })

    const onContentLoaded = vi.fn()
    const result = await fileManager.openFile(mockFileNode, onContentLoaded)

    expect(result).toBe(true)
    expect(fileManager.openFiles.value.length).toBe(1)
    
    const openedFile = fileManager.openFiles.value[0]
    expect(openedFile.isImage).toBe(true)
    expect(openedFile.mimeType).toBe(mockMimeType)
    expect(openedFile.content).toBe(`data:${mockMimeType};base64,${mockBase64}`)
    
    expect(onContentLoaded).toHaveBeenCalledWith(`data:${mockMimeType};base64,${mockBase64}`)
  })

  it('应该处理重复打开同一个文件', async () => {
    const mockFileNode: FileNode = {
      name: 'test.txt',
      path: '/project/test.txt',
      isDirectory: false
    }

    vi.mocked(readFileContent).mockResolvedValue({
      success: true,
      message: '',
      content: 'content1',
      encoding: 'UTF-8',
      is_binary: false,
      is_image: false
    })

    // 第一次打开
    await fileManager.openFile(mockFileNode)
    expect(fileManager.openFiles.value.length).toBe(1)

    // 第二次打开同一个文件
    const result = await fileManager.openFile(mockFileNode)
    expect(result).toBe(true)
    expect(fileManager.openFiles.value.length).toBe(1) // 不应该重复添加
    expect(fileManager.activeFileIndex.value).toBe(0) // 应该切换到已打开的文件
  })

  it('应该拒绝打开文件夹', async () => {
    const mockFolderNode: FileNode = {
      name: 'folder',
      path: '/project/folder',
      isDirectory: true
    }

    const result = await fileManager.openFile(mockFolderNode)
    expect(result).toBe(false)
    expect(fileManager.openFiles.value.length).toBe(0)
  })

  it('应该处理文件打开失败', async () => {
    const mockFileNode: FileNode = {
      name: 'test.txt',
      path: '/project/test.txt',
      isDirectory: false
    }

    vi.mocked(readFileContent).mockResolvedValue({
      success: false,
      message: '文件不存在',
      content: '',
      encoding: 'UTF-8',
      is_binary: false,
      is_image: false
    })

    const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {})
    const result = await fileManager.openFile(mockFileNode)

    expect(result).toBe(false)
    expect(fileManager.openFiles.value.length).toBe(0)
    expect(alertSpy).toHaveBeenCalledWith('打开文件失败: 文件不存在')
    alertSpy.mockRestore()
  })

  it('应该处理二进制文件警告', async () => {
    const mockFileNode: FileNode = {
      name: 'binary.dat',
      path: '/project/binary.dat',
      isDirectory: false
    }

    vi.mocked(readFileContent).mockResolvedValue({
      success: true,
      message: '这是二进制文件',
      content: 'binary content',
      encoding: 'UTF-8',
      is_binary: true,
      is_image: false
    })

    const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {})
    const result = await fileManager.openFile(mockFileNode)

    expect(result).toBe(true)
    expect(alertSpy).toHaveBeenCalledWith('这是二进制文件\n文件可能包含二进制数据，显示可能不正确。')
    alertSpy.mockRestore()
  })

  it('应该处理图片文件警告', async () => {
    const mockFileNode: FileNode = {
      name: 'image.jpg',
      path: '/project/image.jpg',
      isDirectory: false
    }

    // 模拟 readImageAsBase64 返回图片文件警告
    vi.mocked(readImageAsBase64).mockResolvedValue({
      success: false,
      message: '这是图片文件',
      base64: '',
      mimeType: ''
    })

    const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {})
    const result = await fileManager.openFile(mockFileNode)

    expect(result).toBe(false)
    expect(alertSpy).toHaveBeenCalledWith('打开图片失败: 这是图片文件')
    alertSpy.mockRestore()
  })

  it('应该能够切换文件', async () => {
    // 先打开两个文件
    const mockFile1: FileNode = { name: 'file1.txt', path: '/file1.txt', isDirectory: false }
    const mockFile2: FileNode = { name: 'file2.txt', path: '/file2.txt', isDirectory: false }

    vi.mocked(readFileContent).mockResolvedValue({
      success: true,
      message: '',
      content: 'content',
      encoding: 'UTF-8',
      is_binary: false,
      is_image: false
    })

    await fileManager.openFile(mockFile1)
    await fileManager.openFile(mockFile2)

    expect(fileManager.activeFileIndex.value).toBe(1)

    // 切换到第一个文件，并提供更新内容
    // 这应该将当前文件（索引1）的内容更新为 'updated content'
    fileManager.switchToFile(0, 'updated content')
    expect(fileManager.activeFileIndex.value).toBe(0)
    expect(fileManager.currentFile.value).toStrictEqual(mockFile1)
    
    // 检查第二个文件（之前活动的文件）内容是否更新
    expect(fileManager.openFiles.value[1].content).toBe('updated content')
    // 检查第一个文件内容是否保持原来的值
    expect(fileManager.openFiles.value[0].content).toBe('content')
  })

  it('应该能够关闭文件', async () => {
    const mockFileNode: FileNode = {
      name: 'test.txt',
      path: '/project/test.txt',
      isDirectory: false
    }

    vi.mocked(readFileContent).mockResolvedValue({
      success: true,
      message: '',
      content: 'content',
      encoding: 'UTF-8',
      is_binary: false,
      is_image: false
    })

    await fileManager.openFile(mockFileNode)
    expect(fileManager.openFiles.value.length).toBe(1)

    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true)
    const result = fileManager.closeFile(0)

    expect(result).toBe(true)
    expect(fileManager.openFiles.value.length).toBe(0)
    expect(fileManager.activeFileIndex.value).toBe(-1)
    expect(fileManager.currentFile.value).toBeNull()
    confirmSpy.mockRestore()
  })

  it('应该处理有未保存更改的文件关闭', async () => {
    const mockFileNode: FileNode = {
      name: 'test.txt',
      path: '/project/test.txt',
      isDirectory: false
    }

    vi.mocked(readFileContent).mockResolvedValue({
      success: true,
      message: '',
      content: 'content',
      encoding: 'UTF-8',
      is_binary: false,
      is_image: false
    })

    await fileManager.openFile(mockFileNode)
    fileManager.updateFileState('modified content', true)

    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(false)
    const result = fileManager.closeFile(0)

    expect(result).toBe(false)
    expect(fileManager.openFiles.value.length).toBe(1) // 文件应该仍然打开
    confirmSpy.mockRestore()
  })

  it('应该能够关闭所有文件', async () => {
    // 打开多个文件
    const mockFiles = [
      { name: 'file1.txt', path: '/file1.txt', isDirectory: false },
      { name: 'file2.txt', path: '/file2.txt', isDirectory: false },
      { name: 'file3.txt', path: '/file3.txt', isDirectory: false }
    ]

    vi.mocked(readFileContent).mockResolvedValue({
      success: true,
      message: '',
      content: 'content',
      encoding: 'UTF-8',
      is_binary: false,
      is_image: false
    })

    for (const file of mockFiles) {
      await fileManager.openFile(file)
    }

    expect(fileManager.openFiles.value.length).toBe(3)

    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true)
    const result = fileManager.closeAllFiles()

    expect(result).toBe(true)
    expect(fileManager.openFiles.value.length).toBe(0)
    expect(fileManager.activeFileIndex.value).toBe(-1)
    confirmSpy.mockRestore()
  })

  it('应该能够关闭其他文件', async () => {
    // 打开多个文件
    const mockFiles = [
      { name: 'file1.txt', path: '/file1.txt', isDirectory: false },
      { name: 'file2.txt', path: '/file2.txt', isDirectory: false },
      { name: 'file3.txt', path: '/file3.txt', isDirectory: false }
    ]

    vi.mocked(readFileContent).mockResolvedValue({
      success: true,
      message: '',
      content: 'content',
      encoding: 'UTF-8',
      is_binary: false,
      is_image: false
    })

    for (const file of mockFiles) {
      await fileManager.openFile(file)
    }

    expect(fileManager.openFiles.value.length).toBe(3)

    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true)
    const result = fileManager.closeOtherFiles(1)

    expect(result).toBe(true)
    expect(fileManager.openFiles.value.length).toBe(1)
    expect(fileManager.activeFileIndex.value).toBe(0)
    expect(fileManager.currentFile.value?.name).toBe('file2.txt')
    confirmSpy.mockRestore()
  })

  it('应该能够保存文件', async () => {
    const mockFileNode: FileNode = {
      name: 'test.txt',
      path: '/project/test.txt',
      isDirectory: false
    }

    vi.mocked(readFileContent).mockResolvedValue({
      success: true,
      message: '',
      content: 'original content',
      encoding: 'UTF-8',
      is_binary: false,
      is_image: false
    })

    vi.mocked(writeFileContent).mockResolvedValue({
      success: true,
      message: ''
    })

    await fileManager.openFile(mockFileNode)
    fileManager.updateFileState('modified content', true)

    const result = await fileManager.saveFile('saved content')

    expect(result).toBe(true)
    expect(fileManager.openFiles.value[0].hasUnsavedChanges).toBe(false)
    expect(fileManager.openFiles.value[0].content).toBe('saved content')
    expect(writeFileContent).toHaveBeenCalledWith('/project/test.txt', 'saved content')
  })

  it('应该处理文件保存失败', async () => {
    const mockFileNode: FileNode = {
      name: 'test.txt',
      path: '/project/test.txt',
      isDirectory: false
    }

    vi.mocked(readFileContent).mockResolvedValue({
      success: true,
      message: '',
      content: 'content',
      encoding: 'UTF-8',
      is_binary: false,
      is_image: false
    })

    vi.mocked(writeFileContent).mockResolvedValue({
      success: false,
      message: '保存失败'
    })

    const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {})
    await fileManager.openFile(mockFileNode)
    const result = await fileManager.saveFile('content')

    expect(result).toBe(false)
    expect(alertSpy).toHaveBeenCalledWith('保存失败: 保存失败')
    alertSpy.mockRestore()
  })

  it('应该更新文件状态', () => {
    const mockFileNode: FileNode = {
      name: 'test.txt',
      path: '/project/test.txt',
      isDirectory: false
    }

    // 模拟打开文件
    fileManager.openFiles.value = [{
      node: mockFileNode,
      content: 'original',
      hasUnsavedChanges: false,
      cursorLine: 1,
      cursorColumn: 1
    }]
    fileManager.activeFileIndex.value = 0

    fileManager.updateFileState('modified', true)

    expect(fileManager.openFiles.value[0].content).toBe('modified')
    expect(fileManager.openFiles.value[0].hasUnsavedChanges).toBe(true)
  })

  it('应该处理图片打开失败', async () => {
    const mockFileNode: FileNode = {
      name: 'image.png',
      path: '/project/image.png',
      isDirectory: false
    }

    vi.mocked(readImageAsBase64).mockResolvedValue({
      success: false,
      message: '图片读取失败'
    })

    const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {})
    const result = await fileManager.openFile(mockFileNode)

    expect(result).toBe(false)
    expect(alertSpy).toHaveBeenCalledWith('打开图片失败: 图片读取失败')
    alertSpy.mockRestore()
  })

  it('应该处理文件打开异常', async () => {
    const mockFileNode: FileNode = {
      name: 'test.txt',
      path: '/project/test.txt',
      isDirectory: false
    }

    vi.mocked(readFileContent).mockRejectedValue(new Error('网络错误'))

    const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {})
    const result = await fileManager.openFile(mockFileNode)

    expect(result).toBe(false)
    expect(alertSpy).toHaveBeenCalledWith('打开文件失败: Error: 网络错误')
    expect(logger.error).toHaveBeenCalledWith('打开文件失败:', expect.any(Error))
    alertSpy.mockRestore()
  })

  it('应该处理文件切换时的边界情况', () => {
    // 测试无效索引
    fileManager.switchToFile(-1)
    fileManager.switchToFile(10)
    
    // 应该不会崩溃
    expect(fileManager.activeFileIndex.value).toBe(-1)
  })

  it('应该处理文件关闭时的索引调整', async () => {
    // 打开三个文件
    const mockFiles = [
      { name: 'file1.txt', path: '/file1.txt', isDirectory: false },
      { name: 'file2.txt', path: '/file2.txt', isDirectory: false },
      { name: 'file3.txt', path: '/file3.txt', isDirectory: false }
    ]

    vi.mocked(readFileContent).mockResolvedValue({
      success: true,
      message: '',
      content: 'content',
      encoding: 'UTF-8',
      is_binary: false,
      is_image: false
    })

    for (const file of mockFiles) {
      await fileManager.openFile(file)
    }

    expect(fileManager.activeFileIndex.value).toBe(2)

    // 关闭中间的文件
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true)
    fileManager.closeFile(1)

    expect(fileManager.openFiles.value.length).toBe(2)
    expect(fileManager.activeFileIndex.value).toBe(1) // 应该调整索引
    confirmSpy.mockRestore()
  })
})
