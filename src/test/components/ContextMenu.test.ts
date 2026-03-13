import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ContextMenu from '@/components/editor/ContextMenu.vue'

describe('ContextMenu', () => {
  it('文件树节点右键菜单应包含复制操作', async () => {
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        x: 16,
        y: 24,
        menuType: 'tree',
        treeNodePath: '/project/test.txt',
        treeNodeIsDirectory: false,
        projectRoot: '/project'
      }
    })

    const buttons = wrapper.findAll('button')
    expect(buttons).toHaveLength(8)

    await buttons[4].trigger('click')
    expect(wrapper.emitted('action')).toEqual([['copy', undefined]])
  })

  it('文件树节点右键菜单应包含剪切操作', async () => {
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        x: 16,
        y: 24,
        menuType: 'tree',
        treeNodePath: '/project/test.txt',
        treeNodeIsDirectory: false,
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
        projectRoot: '/project'
      }
    })

    const buttons = wrapper.findAll('button')
    expect(buttons).toHaveLength(9)

    await buttons[6].trigger('click')
    expect(wrapper.emitted('action')).toEqual([['paste', undefined]])
  })
})
