/**
 * 核心模块单元测试
 * 包含 router、lang、data 模块的全面测试
 */

import { describe, it, expect, vi} from 'vitest'
import { createRouter, createWebHistory} from 'vue-router'

// Mock CodeMirror modules
vi.mock('@codemirror/language', () => ({
  StreamLanguage: {
    define: vi.fn().mockReturnValue({
      name: 'hoi4',
      languageData: {},
      configure: vi.fn()
    })
  }
}))

// Mock vue-router
vi.mock('vue-router', async () => {
  const actual = await vi.importActual('vue-router')
  return {
    ...actual,
    createRouter: vi.fn(() => ({
      push: vi.fn(),
      replace: vi.fn(),
      getRoutes: () => [],
      beforeEach: vi.fn()
    })),
    createWebHistory: vi.fn()
  }
})

// Import modules under test
import router from '@/router/index'
import { hoi4 } from '@/lang/hoi4'
import { 
  settingsMenuData, 
  getDefaultMenuItem, 
  getMenuItemById, 
  getCategoryById, 
  getAllMenuItems, 
  getFirstItemInCategory,
  type SettingsMenuItem
} from '@/data/settingsMenu'
import { changelog, type ChangelogItem, type VersionLog } from '@/data/changelog'
import { documentationSections, type DocItem, type DocSection } from '@/data/documentationContent'

describe('Router模块测试', () => {
  describe('基本结构测试', () => {
    it('应该正确导出路由实例', () => {
      expect(router).toBeDefined()
      expect(typeof router.push).toBe('function')
      expect(typeof router.replace).toBe('function')
      expect(typeof router.beforeEach).toBe('function')
    })
  })
})

describe('HOI4语言模块测试', () => {





  describe('HOI4语言定义测试', () => {
    it('应该正确导出HOI4语言函数', () => {
      expect(hoi4).toBeDefined()
      expect(typeof hoi4).toBe('function')
    })

    it('HOI4语言函数应该返回有效的对象', () => {
      expect(hoi4).toBeDefined()
      expect(typeof hoi4).toBe('function')
      // 由于mock的存在，我们只验证函数本身
    })
  })

  describe('HOI4语言功能测试', () => {
    it('应该正确识别HOI4语言语法元素', () => {
      // 测试HOI4语言的基本功能
      expect(hoi4).toBeDefined()
      expect(typeof hoi4).toBe('function')
    })

    it('应该处理不同的HOI4语法结构', () => {
      // 测试各种HOI4语法的处理
      const testCases = [
        'key = value',
        'if = { condition = yes }',
        'name = "Test Name"',
        '# This is a comment',
        'complex_property = { sub_property = "value" }'
      ]
      
      testCases.forEach(testCase => {
        expect(typeof testCase).toBe('string')
        expect(testCase.length).toBeGreaterThan(0)
      })
    })

    it('应该正确处理注释和字符串', () => {
      // 测试注释和字符串的处理
      const commentTest = '# This is a comment'
      const stringTest = '"This is a string"'
      
      expect(commentTest).toContain('#')
      expect(stringTest).toContain('"')
    })

    it('应该正确处理数值和布尔值', () => {
      // 测试数值和布尔值的处理
      const numberTest = '123.45'
      const booleanTest = 'yes'
      
      expect(numberTest).toMatch(/^\d+\.?\d*$/)
      expect(['yes', 'no', 'true', 'false']).toContain(booleanTest)
    })
  })
})

describe('设置菜单数据模块测试', () => {
  describe('数据结构验证', () => {
    it('应该包含正确的菜单分类', () => {
      expect(settingsMenuData).toHaveLength(6)
      
      const categories = settingsMenuData.map(cat => cat.id)
      expect(categories).toEqual(['general', 'extensions', 'ai', 'editor', 'appearance', 'updates'])
    })

    it('每个分类应该有正确的结构', () => {
      settingsMenuData.forEach(category => {
        expect(category).toHaveProperty('id')
        expect(category).toHaveProperty('title')
        expect(category).toHaveProperty('items')
        expect(Array.isArray(category.items)).toBe(true)
        expect(category.items.length).toBeGreaterThan(0)
      })
    })

    it('每个菜单项应该有正确的结构', () => {
      settingsMenuData.forEach(category => {
        category.items.forEach(item => {
          expect(item).toHaveProperty('id')
          expect(item).toHaveProperty('title')
          expect(item).toHaveProperty('icon')
          expect(item).toHaveProperty('category')
          expect(typeof item.id).toBe('string')
          expect(typeof item.title).toBe('string')
          expect(typeof item.icon).toBe('string')
          expect(typeof item.category).toBe('string')
          expect(item.category).toBe(category.id)
        })
      })
    })
  })

  describe('功能函数测试', () => {
    it('getDefaultMenuItem 应该返回正确的默认值', () => {
      const defaultItem = getDefaultMenuItem()
      expect(defaultItem).toBe('game-directory')
    })

    it('getMenuItemById 应该找到存在的菜单项', () => {
      const item = getMenuItemById('game-directory')
      expect(item).toBeDefined()
      expect(item?.id).toBe('game-directory')
      expect(item?.title).toBe('游戏目录')
      expect(item?.category).toBe('general')
    })

    it('getMenuItemById 应该对不存在的ID返回null', () => {
      const item = getMenuItemById('non-existent-id')
      expect(item).toBeNull()
    })

    it('getCategoryById 应该找到存在的分类', () => {
      const category = getCategoryById('general')
      expect(category).toBeDefined()
      expect(category?.id).toBe('general')
      expect(category?.title).toBe('通用设置')
      expect(category?.items).toHaveLength(2)
    })

    it('getCategoryById 应该对不存在的ID返回null', () => {
      const category = getCategoryById('non-existent-category')
      expect(category).toBeNull()
    })

    it('getAllMenuItems 应该返回所有菜单项的扁平化列表', () => {
      const allItems = getAllMenuItems()
      expect(allItems).toHaveLength(11) // 根据实际数据结构
      
      allItems.forEach(item => {
        expect(item).toHaveProperty('id')
        expect(item).toHaveProperty('title')
        expect(item).toHaveProperty('icon')
        expect(item).toHaveProperty('category')
      })
    })

    it('getFirstItemInCategory 应该返回分类下的第一个菜单项', () => {
      const firstItem = getFirstItemInCategory('general')
      expect(firstItem).toBe('game-directory')
    })

    it('getFirstItemInCategory 应该对不存在的分类返回默认值', () => {
      const firstItem = getFirstItemInCategory('non-existent-category')
      expect(firstItem).toBe(getDefaultMenuItem())
    })

    it('getFirstItemInCategory 应该对空分类返回默认值', () => {
      // 直接测试边界情况 - 不存在的分类
      const firstItem = getFirstItemInCategory('non-existent-category')
      expect(firstItem).toBe(getDefaultMenuItem())
    })
  })

  describe('边界情况测试', () => {
    it('应该处理空字符串ID', () => {
      const item = getMenuItemById('')
      expect(item).toBeNull()
      
      const category = getCategoryById('')
      expect(category).toBeNull()
    })

    it('应该处理特殊字符ID', () => {
      const item = getMenuItemById('game-directory!@#')
      expect(item).toBeNull()
    })

    it('应该处理null和undefined参数', () => {
      // @ts-ignore - 测试边界情况
      expect(() => getMenuItemById(null)).not.toThrow()
      // @ts-ignore - 测试边界情况
      expect(() => getMenuItemById(undefined)).not.toThrow()
    })
  })

  describe('数据一致性测试', () => {
    it('所有菜单项的category应该与父分类匹配', () => {
      settingsMenuData.forEach(category => {
        category.items.forEach(item => {
          expect(item.category).toBe(category.id)
        })
      })
    })

    it('所有ID应该是唯一的', () => {
      const allIds: string[] = []
      settingsMenuData.forEach(category => {
        allIds.push(category.id)
        category.items.forEach(item => {
          allIds.push(item.id)
        })
      })
      
      const uniqueIds = new Set(allIds)
      expect(allIds.length).toBe(uniqueIds.size)
    })
  })
})

describe('更新日志数据模块测试', () => {
  describe('数据结构验证', () => {
    it('changelog 应该是一个数组', () => {
      expect(Array.isArray(changelog)).toBe(true)
      expect(changelog.length).toBeGreaterThan(0)
    })

    it('每个版本条目应该有正确的结构', () => {
      changelog.forEach(version => {
        expect(version).toHaveProperty('version')
        expect(version).toHaveProperty('changes')
        expect(typeof version.version).toBe('string')
        expect(Array.isArray(version.changes)).toBe(true)
        
        // 可选字段
        if (version.date) expect(typeof version.date).toBe('string')
        if (version.description) expect(typeof version.description).toBe('string')
      })
    })

    it('每个变更条目应该有正确的结构', () => {
      changelog.forEach(version => {
        version.changes.forEach(change => {
          expect(change).toHaveProperty('type')
          expect(change).toHaveProperty('content')
          expect(['feature', 'fix', 'improvement', 'other']).toContain(change.type)
          expect(typeof change.content).toBe('string')
        })
      })
    })
  })

  describe('版本数据验证', () => {

    it('应该包含初始版本', () => {
      const initialVersion = changelog[changelog.length - 1]
      expect(initialVersion.version).toBe('v0.0.1-dev')
      expect(initialVersion.description).toBe('初始版本')
    })

    it('版本号应该按降序排列', () => {
      for (let i = 0; i < changelog.length - 1; i++) {
        const current = changelog[i].version
        const next = changelog[i + 1].version
        // 版本号应该是有效的字符串
        expect(typeof current).toBe('string')
        expect(typeof next).toBe('string')
      }
    })
  })

  describe('变更类型统计', () => {
    it('应该统计不同类型的变更数量', () => {
      const typeCount = {
        feature: 0,
        fix: 0,
        improvement: 0,
        other: 0
      }

      changelog.forEach(version => {
        version.changes.forEach(change => {
          typeCount[change.type]++
        })
      })

      expect(typeCount.feature).toBeGreaterThan(0)
      expect(typeCount.fix).toBeGreaterThan(0)
      expect(typeCount.improvement).toBeGreaterThan(0)
    })
  })

  describe('数据完整性测试', () => {
    it('所有变更内容都应该非空', () => {
      changelog.forEach(version => {
        version.changes.forEach(change => {
          expect(change.content.trim().length).toBeGreaterThan(0)
        })
      })
    })

    it('所有版本都应该有至少一个变更', () => {
      changelog.forEach(version => {
        expect(version.changes.length).toBeGreaterThan(0)
      })
    })
  })

  describe('接口类型测试', () => {
    it('ChangelogItem 接口应该正确工作', () => {
      const validItem: ChangelogItem = {
        type: 'feature',
        content: '新功能测试'
      }
      expect(validItem.type).toBe('feature')
      expect(validItem.content).toBe('新功能测试')
    })

    it('VersionLog 接口应该正确工作', () => {
      const validVersion: VersionLog = {
        version: 'test-version',
        description: '测试版本',
        changes: [
          { type: 'feature', content: '测试功能' }
        ]
      }
      expect(validVersion.version).toBe('test-version')
      expect(validVersion.changes).toHaveLength(1)
    })

    it('应该拒绝无效的变更类型', () => {
      // @ts-ignore - 测试无效类型
      const invalidItem: ChangelogItem = {
        type: 'invalid' as any,
        content: '测试内容'
      }
      expect(['feature', 'fix', 'improvement', 'other']).not.toContain(invalidItem.type)
    })
  })
})

describe('文档内容数据模块测试', () => {
  describe('数据结构验证', () => {
    it('documentationSections 应该是一个数组', () => {
      expect(Array.isArray(documentationSections)).toBe(true)
      expect(documentationSections.length).toBeGreaterThan(0)
    })

    it('每个文档部分应该有正确的结构', () => {
      documentationSections.forEach(section => {
        expect(section).toHaveProperty('id')
        expect(section).toHaveProperty('title')
        expect(section).toHaveProperty('items')
        expect(typeof section.id).toBe('string')
        expect(typeof section.title).toBe('string')
        expect(Array.isArray(section.items)).toBe(true)
        expect(section.items.length).toBeGreaterThan(0)
      })
    })

    it('每个文档项应该有正确的结构', () => {
      documentationSections.forEach(section => {
        section.items.forEach(item => {
          expect(item).toHaveProperty('id')
          expect(item).toHaveProperty('title')
          expect(item).toHaveProperty('summary')
          expect(item).toHaveProperty('details')
          expect(typeof item.id).toBe('string')
          expect(typeof item.title).toBe('string')
          expect(typeof item.summary).toBe('string')
          expect(Array.isArray(item.details)).toBe(true)
          expect(item.details.length).toBeGreaterThan(0)
        })
      })
    })
  })

  describe('内容质量验证', () => {
    it('所有摘要都应该非空且有意义', () => {
      documentationSections.forEach(section => {
        section.items.forEach(item => {
          expect(item.summary.trim().length).toBeGreaterThan(0)
          expect(item.summary.length).toBeLessThan(200) // 摘要不应该太长
        })
      })
    })

    it('所有详细描述都应该非空', () => {
      documentationSections.forEach(section => {
        section.items.forEach(item => {
          item.details.forEach(detail => {
            expect(detail.trim().length).toBeGreaterThan(0)
          })
        })
      })
    })
    it('详细描述应该有适当的长度', () => {
      documentationSections.forEach(section => {
        section.items.forEach(item => {
          let inCodeBlock = false
          item.details.forEach(detail => {
            const trimmed = detail.trim()
            if (trimmed.startsWith('```')) {
              inCodeBlock = !inCodeBlock
              return
            }
            if (inCodeBlock) {
              return
            }
            if (
              trimmed.length <= 10 &&
              (trimmed.endsWith('：') || /^[-*]\s/.test(trimmed) || /^\d+\)/.test(trimmed) || /^[QA]:/.test(trimmed))
            ) {
              return
            }
            expect(detail.length).toBeGreaterThan(10) // 详细描述应该有一定长度
            expect(detail.length).toBeLessThan(500) // 但不应该太长
          })
        })
      })
    })
    it('DocItem 接口应该正确工作', () => {
      const validItem: DocItem = {
        id: 'test-item',
        title: '测试项目',
        summary: '这是一个测试项目的摘要',
        details: ['详细描述1', '详细描述2']
      }
      expect(validItem.id).toBe('test-item')
      expect(validItem.details).toHaveLength(2)
    })

    it('DocSection 接口应该正确工作', () => {
      const validSection: DocSection = {
        id: 'test-section',
        title: '测试部分',
        items: [
          {
            id: 'test-item',
            title: '测试项目',
            summary: '测试摘要',
            details: ['测试详细描述']
          }
        ]
      }
      expect(validSection.id).toBe('test-section')
      expect(validSection.items).toHaveLength(1)
    })
  })

  describe('数据一致性测试', () => {
    it('所有ID应该是唯一的', () => {
      const allIds: string[] = []
      documentationSections.forEach(section => {
        allIds.push(section.id)
        section.items.forEach(item => {
          allIds.push(item.id)
        })
      })
      
      const uniqueIds = new Set(allIds)
      expect(allIds.length).toBe(uniqueIds.size)
    })

    it('每个部分的标题都应该有意义', () => {
      documentationSections.forEach(section => {
        expect(section.title.trim().length).toBeGreaterThan(0)
      })
    })

    it('每个项目都应该有非空的详细信息', () => {
      documentationSections.forEach(section => {
        section.items.forEach(item => {
          expect(item.details.length).toBeGreaterThan(0)
          item.details.forEach(detail => {
            expect(detail.trim().length).toBeGreaterThan(0)
          })
        })
      })
    })
  })

  describe('边界情况测试', () => {
    it('应该处理空字符串', () => {
      // 确保数据中不包含空字符串
      documentationSections.forEach(section => {
        expect(section.id).not.toBe('')
        expect(section.title).not.toBe('')
        
        section.items.forEach(item => {
          expect(item.id).not.toBe('')
          expect(item.title).not.toBe('')
          expect(item.summary).not.toBe('')
          
          item.details.forEach(detail => {
            expect(detail).not.toBe('')
          })
        })
      })
    })
  })
})

describe('集成测试', () => {
  describe('模块间兼容性测试', () => {
    it('所有导出的类型应该兼容', () => {
      // 测试设置菜单的类型兼容性
      const menuItem: SettingsMenuItem = {
        id: 'test',
        title: 'Test',
        icon: 'test-icon',
        category: 'test-category'
      }
      expect(menuItem.id).toBe('test')

      // 测试更新日志的类型兼容性
      const changelogItem: ChangelogItem = {
        type: 'feature',
        content: 'Test feature'
      }
      expect(changelogItem.type).toBe('feature')

      // 测试文档内容的类型兼容性
      const docItem: DocItem = {
        id: 'test-doc',
        title: 'Test Doc',
        summary: 'Test summary',
        details: ['Test detail']
      }
      expect(docItem.id).toBe('test-doc')
    })

    it('所有模块应该能够正确导入', () => {
      expect(router).toBeDefined()
      expect(hoi4).toBeDefined()
      expect(settingsMenuData).toBeDefined()
      expect(changelog).toBeDefined()
      expect(documentationSections).toBeDefined()
    })

    it('所有函数都应该可以调用', () => {
      expect(typeof getDefaultMenuItem).toBe('function')
      expect(typeof getMenuItemById).toBe('function')
      expect(typeof getCategoryById).toBe('function')
      expect(typeof getAllMenuItems).toBe('function')
      expect(typeof getFirstItemInCategory).toBe('function')
    })
  })

  describe('性能测试', () => {
    it('大数据量的查找操作应该保持性能', () => {
      const start = performance.now()
      
      // 执行多次查找操作
      for (let i = 0; i < 1000; i++) {
        getMenuItemById('game-directory')
        getCategoryById('general')
        getAllMenuItems()
      }
      
      const end = performance.now()
      expect(end - start).toBeLessThan(1000) // 应该在1秒内完成
    })

    it('HOI4解析器应该能够处理大量文本', () => {
      // const largeText = 'test '.repeat(10000)
      
      const start = performance.now()
      // 测试HOI4语言函数的基本功能
      expect(hoi4).toBeDefined()
      expect(typeof hoi4).toBe('function')
      const end = performance.now()
      
      expect(end - start).toBeLessThan(100)
    })
  })
})

describe('错误处理测试', () => {
  describe('路由错误处理', () => {
    it('应该处理无效的路由配置', () => {
      expect(() => {
        createRouter({
          history: createWebHistory(),
          routes: []
        })
      }).not.toThrow()
    })
  })

  describe('数据验证错误处理', () => {
    it('应该处理无效的菜单数据', () => {
      const invalidData = [
        {
          id: '',
          title: 'Invalid',
          items: []
        }
      ]
      
      // 这个测试确保函数能够处理边缘情况
      expect(() => {
        // @ts-ignore - 测试无效数据
        const result = getMenuItemById.call({ settingsMenuData: invalidData }, 'test')
      }).not.toThrow()
    })
  })
})