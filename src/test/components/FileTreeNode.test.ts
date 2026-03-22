/**
 * FileTreeNode 组件单元测试
 */

import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import FileTreeNode from '@/components/FileTreeNode.vue'

interface FileNode {
  name: string
  path: string
  isDirectory: boolean
  children?: FileNode[]
  expanded?: boolean
}

const createMockFileNode = (overrides: Partial<FileNode> = {}): FileNode => ({
  name: 'test.txt',
  path: '/test/test.txt',
  isDirectory: false,
  children: undefined,
  expanded: false,
  ...overrides
})

const defaultProps = {
  node: createMockFileNode(),
  level: 0,
  selectedPaths: [] as string[]
}

describe('FileTreeNode', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('应该正确渲染文件节点', () => {
    const wrapper = mount(FileTreeNode, {
      props: defaultProps
    })

    expect(wrapper.find('.file-tree-node').exists()).toBe(true)
    expect(wrapper.find('.text-hoi4-text').text()).toBe('test.txt')

    const spans = wrapper.findAll('span')
    expect(spans.length).toBeGreaterThan(0)
  })

  it('应该正确渲染目录节点', () => {
    const directoryNode = createMockFileNode({
      isDirectory: true,
      name: 'src',
      expanded: false
    })

    const wrapper = mount(FileTreeNode, {
      props: {
        ...defaultProps,
        node: directoryNode
      }
    })

    expect(wrapper.find('.file-tree-node').exists()).toBe(true)
    expect(wrapper.find('.text-hoi4-text').text()).toBe('src')

    const spans = wrapper.findAll('span')
    expect(spans.length).toBeGreaterThan(0)
  })

  it('应该在点击文件时触发 openFile 事件', async () => {
    const openFile = vi.fn()
    const select = vi.fn()
    const wrapper = mount(FileTreeNode, {
      props: {
        ...defaultProps,
        onSelect: select,
        onOpenFile: openFile
      }
    })

    await wrapper.find('.file-tree-node').trigger('click')
    expect(select).toHaveBeenCalledWith(expect.any(MouseEvent), defaultProps.node)
    expect(openFile).toHaveBeenCalledWith(defaultProps.node)
  })

  it('应该在点击目录时触发 toggle 事件', async () => {
    const directoryNode = createMockFileNode({
      isDirectory: true,
      name: 'src'
    })

    const toggle = vi.fn()
    const wrapper = mount(FileTreeNode, {
      props: {
        ...defaultProps,
        node: directoryNode,
        onToggle: toggle
      }
    })

    await wrapper.find('.file-tree-node').trigger('click')
    expect(toggle).toHaveBeenCalledWith(directoryNode)
  })

  it('应该正确处理子节点', () => {
    const parentNode = createMockFileNode({
      isDirectory: true,
      name: 'src',
      expanded: true,
      children: [
        createMockFileNode({ name: 'App.vue', path: '/src/App.vue' }),
        createMockFileNode({ name: 'main.ts', path: '/src/main.ts' })
      ]
    })

    const wrapper = mount(FileTreeNode, {
      props: {
        ...defaultProps,
        node: parentNode
      }
    })

    const childNodes = wrapper.findAllComponents(FileTreeNode)
    expect(childNodes).toHaveLength(2)
    expect(childNodes[0].props().node.name).toBe('App.vue')
    expect(childNodes[1].props().node.name).toBe('main.ts')
  })

  it('应该将深层目录节点原样透传给父级监听器', async () => {
    const nestedDirectoryNode = createMockFileNode({
      isDirectory: true,
      name: 'nested',
      path: '/src/nested',
      expanded: false
    })
    const parentNode = createMockFileNode({
      isDirectory: true,
      name: 'src',
      path: '/src',
      expanded: true,
      children: [nestedDirectoryNode]
    })

    const wrapper = mount(FileTreeNode, {
      props: {
        ...defaultProps,
        node: parentNode
      }
    })

    const nodes = wrapper.findAll('.file-tree-node')
    await nodes[1].trigger('click')

    const selectEvents = wrapper.emitted('select')
    const toggleEvents = wrapper.emitted('toggle')

    expect(selectEvents).toBeTruthy()
    expect(toggleEvents).toBeTruthy()
    expect(selectEvents?.[0]?.[1]).toEqual(nestedDirectoryNode)
    expect(toggleEvents?.[0]?.[0]).toEqual(nestedDirectoryNode)
  })

  it('应该根据文件类型显示图标', () => {
    const testCases = [
      'App.vue',
      'main.ts',
      'style.css',
      'README.md',
      'image.png',
      'data.json',
      'test.mod'
    ]

    testCases.forEach((fileName) => {
      const fileNode = createMockFileNode({
        name: fileName,
        path: `/test/${fileName}`
      })

      const wrapper = mount(FileTreeNode, {
        props: {
          ...defaultProps,
          node: fileNode
        }
      })

      const spans = wrapper.findAll('span')
      expect(spans.length).toBeGreaterThan(0)
    })
  })

  it('应该处理右键菜单事件', async () => {
    const contextmenu = vi.fn()
    const wrapper = mount(FileTreeNode, {
      props: {
        ...defaultProps,
        onContextmenu: contextmenu
      }
    })

    await wrapper.find('.file-tree-node').trigger('contextmenu')
    expect(contextmenu).toHaveBeenCalledWith(expect.any(MouseEvent), defaultProps.node)
  })

  it('应该正确处理选中状态', () => {
    const wrapper = mount(FileTreeNode, {
      props: {
        ...defaultProps,
        selectedPaths: ['/test/test.txt']
      }
    })

    const nodeElement = wrapper.find('.file-tree-node')
    expect(nodeElement.classes()).toContain('bg-hoi4-selected')
    expect(nodeElement.classes()).toContain('text-white')
  })

  it('应该根据层级调整缩进', () => {
    const level2Node = createMockFileNode({
      name: 'nested-file.txt',
      path: '/test/level1/level2/nested-file.txt'
    })

    const wrapper = mount(FileTreeNode, {
      props: {
        ...defaultProps,
        node: level2Node,
        level: 2
      }
    })

    const nodeElement = wrapper.find('.file-tree-node')
    expect(nodeElement.attributes('style')).toContain('padding-left: 40px')
  })

  it('应该正确显示展开目录的图标', () => {
    const expandedDirectoryNode = createMockFileNode({
      isDirectory: true,
      name: 'src',
      expanded: true
    })

    const wrapper = mount(FileTreeNode, {
      props: {
        ...defaultProps,
        node: expandedDirectoryNode
      }
    })

    const spans = wrapper.findAll('span')
    expect(spans.length).toBeGreaterThan(0)
  })

  it('应该正确处理空文件节点', () => {
    const emptyNode = createMockFileNode({
      name: '',
      path: ''
    })

    const wrapper = mount(FileTreeNode, {
      props: {
        ...defaultProps,
        node: emptyNode
      }
    })

    expect(wrapper.find('.text-hoi4-text').text()).toBe('')
  })
})
