import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import FileTreeNode from '@/components/FileTreeNode.vue'

describe('FileTreeNode selection', () => {
  it('ctrl 点击时只触发选中事件，不直接打开文件', async () => {
    const node = {
      name: 'test.txt',
      path: '/test/test.txt',
      isDirectory: false,
      expanded: false
    }

    const openFile = vi.fn()
    const select = vi.fn()
    const wrapper = mount(FileTreeNode, {
      props: {
        node,
        level: 0,
        selectedPaths: [],
        onSelect: select,
        onOpenFile: openFile
      }
    })

    await wrapper.find('.file-tree-node').trigger('click', { ctrlKey: true })

    expect(select).toHaveBeenCalledWith(expect.any(MouseEvent), node)
    expect(openFile).not.toHaveBeenCalled()
  })
})
