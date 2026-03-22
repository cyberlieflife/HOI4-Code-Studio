import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import ContextMenu from '@/components/editor/ContextMenu.vue'

describe('ContextMenu', () => {
  const originalInnerWidth = window.innerWidth
  const originalInnerHeight = window.innerHeight
  const originalOffsetWidth = Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'offsetWidth')
  const originalOffsetHeight = Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'offsetHeight')

  beforeEach(() => {
    Object.defineProperty(window, 'innerWidth', {
      configurable: true,
      writable: true,
      value: originalInnerWidth
    })
    Object.defineProperty(window, 'innerHeight', {
      configurable: true,
      writable: true,
      value: originalInnerHeight
    })
  })

  afterEach(() => {
    if (originalOffsetWidth) {
      Object.defineProperty(HTMLElement.prototype, 'offsetWidth', originalOffsetWidth)
    }
    if (originalOffsetHeight) {
      Object.defineProperty(HTMLElement.prototype, 'offsetHeight', originalOffsetHeight)
    }
  })

  it('项目树节点右键菜单应包含复制操作', async () => {
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        x: 16,
        y: 24,
        menuType: 'tree',
        treeNodePath: '/project/test.txt',
        treeNodeIsDirectory: false,
        treeSupportsFileOperations: true,
        projectRoot: '/project'
      }
    })

    const buttons = wrapper.findAll('button')
    expect(buttons).toHaveLength(8)

    await buttons[4].trigger('click')
    expect(wrapper.emitted('action')).toEqual([['copy', undefined]])
  })

  it('项目树节点右键菜单应包含剪切操作', async () => {
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        x: 16,
        y: 24,
        menuType: 'tree',
        treeNodePath: '/project/test.txt',
        treeNodeIsDirectory: false,
        treeSupportsFileOperations: true,
        projectRoot: '/project'
      }
    })

    const buttons = wrapper.findAll('button')
    expect(buttons).toHaveLength(8)

    await buttons[5].trigger('click')
    expect(wrapper.emitted('action')).toEqual([['cut', undefined]])
  })

  it('存在树剪贴板内容时应显示粘贴操作', async () => {
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        x: 16,
        y: 24,
        menuType: 'tree',
        treeNodePath: '/project/folder',
        treeNodeIsDirectory: true,
        hasTreeClipboard: true,
        treeSupportsFileOperations: true,
        projectRoot: '/project'
      }
    })

    const buttons = wrapper.findAll('button')
    expect(buttons).toHaveLength(9)

    await buttons[6].trigger('click')
    expect(wrapper.emitted('action')).toEqual([['paste', undefined]])
  })

  it('只读文件树右键菜单不应显示文件操作', async () => {
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        x: 16,
        y: 24,
        menuType: 'tree',
        treeNodePath: '/game/common/test.txt',
        treeNodeIsDirectory: false,
        treeSupportsFileOperations: false,
        projectRoot: '/project'
      }
    })

    const buttons = wrapper.findAll('button')
    expect(buttons).toHaveLength(2)

    await buttons[0].trigger('click')
    await buttons[1].trigger('click')

    expect(wrapper.emitted('action')).toEqual([
      ['copyPath', undefined],
      ['showInExplorer', undefined]
    ])
  })

  it('菜单超出窗口时应自动夹取到可见区域', async () => {
    Object.defineProperty(window, 'innerWidth', {
      configurable: true,
      writable: true,
      value: 320
    })
    Object.defineProperty(window, 'innerHeight', {
      configurable: true,
      writable: true,
      value: 260
    })
    Object.defineProperty(HTMLElement.prototype, 'offsetWidth', {
      configurable: true,
      get: () => 180
    })
    Object.defineProperty(HTMLElement.prototype, 'offsetHeight', {
      configurable: true,
      get: () => 200
    })

    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        x: 280,
        y: 240,
        menuType: 'tree',
        treeNodePath: '/project/folder',
        treeNodeIsDirectory: true,
        treeSupportsFileOperations: true,
        projectRoot: '/project'
      }
    })

    await nextTick()
    await nextTick()

    const style = wrapper.find('.fixed').attributes('style')
    expect(style).toContain('left: 128px;')
    expect(style).toContain('top: 48px;')
  })
})
