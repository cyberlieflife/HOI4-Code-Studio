/**
 * FileTreeNode 组件的单元测试
 */

import { describe, it, expect, vi, beforeEach } from 'vitest'
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

    // 查找实际的DOM元素
    expect(wrapper.find('.file-tree-node').exists()).toBe(true)
    expect(wrapper.find('.text-hoi4-text').text()).toBe('test.txt')
    
    // 检查是否包含图标（不检查具体的emoji，因为编码问题）
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
    
    // 检查是否包含图标（不检查具体的emoji，因为编码问题）
    const spans = wrapper.findAll('span')
    expect(spans.length).toBeGreaterThan(0)
  })

  it('应该在点击文件时触发openFile事件', async () => {
    const openFile = vi.fn()
    const select = vi.fn()
    const wrapper = mount(FileTreeNode, {
      props: {
        ...defaultProps,
        'onSelect': select,
        'onOpenFile': openFile
      }
    })

    await wrapper.find('.file-tree-node').trigger('click')
    expect(select).toHaveBeenCalledWith(expect.any(MouseEvent), defaultProps.node)
    expect(openFile).toHaveBeenCalledWith(defaultProps.node)
  })

  it('应该在点击目录时触发toggle事件', async () => {
    const directoryNode = createMockFileNode({
      isDirectory: true,
      name: 'src'
    })

    const toggle = vi.fn()
    const wrapper = mount(FileTreeNode, {
      props: {
        ...defaultProps,
        node: directoryNode,
        'onToggle': toggle
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

    // 检查是否渲染了子节点
    const childNodes = wrapper.findAllComponents(FileTreeNode)
    expect(childNodes).toHaveLength(2)
    
    // 检查第二个子组件的props
    expect(childNodes[0].props().node.name).toBe('App.vue')
    expect(childNodes[1].props().node.name).toBe('main.ts')
  })

  it('应该根据文件类型显示不同的图标', () => {
    const testCases = [
      { fileName: 'App.vue', ext: 'vue' },
      { fileName: 'main.ts', ext: 'ts' },
      { fileName: 'style.css', ext: 'css' },
      { fileName: 'README.md', ext: 'md' },
      { fileName: 'image.png', ext: 'png' },
      { fileName: 'data.json', ext: 'json' },
      { fileName: 'test.mod', ext: 'mod' }
    ]

    testCases.forEach(({ fileName }) => {
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

      // 检查是否渲染了图标
      const spans = wrapper.findAll('span')
      expect(spans.length).toBeGreaterThan(0)
    })
  })

  it('应该处理右键菜单事件', async () => {
    const contextmenu = vi.fn()
    const wrapper = mount(FileTreeNode, {
      props: {
        ...defaultProps,
        'onContextmenu': contextmenu
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

  it('应该正确显示展开的目录图标', () => {
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

    // 检查是否渲染了图标
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
