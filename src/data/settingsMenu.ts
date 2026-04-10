// 设置页面菜单数据结构
export interface SettingsMenuItem {
  id: string
  title: string
  icon: string
  description?: string
  category: string
}

export interface SettingsMenuCategory {
  id: string
  title: string
  items: SettingsMenuItem[]
}

// 设置菜单数据
export const settingsMenuData: SettingsMenuCategory[] = [
  {
    id: 'general',
    title: '通用设置',
    items: [
      {
        id: 'game-directory',
        title: '游戏目录',
        icon: 'folder',
        description: 'HOI4 游戏安装目录配置',
        category: 'general'
      },
      {
        id: 'game-launch',
        title: '游戏启动',
        icon: 'play',
        description: '游戏启动方式配置',
        category: 'general'
      }
    ]
  },
  {
    id: 'extensions',
    title: '扩展',
    items: [
      {
        id: 'plugins',
        title: '插件',
        icon: 'puzzle',
        description: '管理已安装的插件',
        category: 'extensions'
      }
    ]
  },
  {
    id: 'ai',
    title: 'AI',
    items: [
      {
        id: 'ai-settings',
        title: 'AI 设置',
        icon: 'ai',
        description: '配置 OpenAI 兼容接口与渲染选项',
        category: 'ai'
      }
    ]
  },
  {
    id: 'editor',
    title: '编辑器',
    items: [
      {
        id: 'editor-font',
        title: '字体设置',
        icon: 'text',
        description: '编辑器字体配置',
        category: 'editor'
      },
      {
        id: 'editor-save',
        title: '保存设置',
        icon: 'save',
        description: '自动保存和错误处理',
        category: 'editor'
      },
      {
        id: 'map-settings',
        title: '地图预览',
        icon: 'map',
        description: '配置地图渲染性能',
        category: 'editor'
      }
    ]
  },
  {
    id: 'cache',
    title: '缓存设置',
    items: [
      {
        id: 'cache-directory',
        title: '缓存目录',
        icon: 'cache',
        description: '配置缓存存储位置',
        category: 'cache'
      }
    ]
  },
  {
    id: 'appearance',
    title: '外观',
    items: [
      {
        id: 'theme',
        title: '主题',
        icon: 'palette',
        description: '界面主题配置',
        category: 'appearance'
      },
      {
        id: 'icons',
        title: '图标',
        icon: 'icons',
        description: '文件树图标配置',
        category: 'appearance'
      }
    ]
  },
  {
    id: 'updates',
    title: '更新',
    items: [
      {
        id: 'update-settings',
        title: '更新设置',
        icon: 'refresh',
        description: '应用更新相关配置',
        category: 'updates'
      },
      {
        id: 'version-info',
        title: '版本信息',
        icon: 'info',
        description: '当前版本和更新检查',
        category: 'updates'
      }
    ]
  }
]

// 获取默认选中的菜单项
export function getDefaultMenuItem(): string {
  return 'game-directory'
}

// 根据ID获取菜单项
export function getMenuItemById(id: string): SettingsMenuItem | null {
  for (const category of settingsMenuData) {
    const item = category.items.find(item => item.id === id)
    if (item) return item
  }
  return null
}

// 根据ID获取分类
export function getCategoryById(id: string): SettingsMenuCategory | null {
  return settingsMenuData.find(category => category.id === id) || null
}

// 获取所有菜单项（扁平化）
export function getAllMenuItems(): SettingsMenuItem[] {
  return settingsMenuData.flatMap(category => category.items)
}

// 获取分类下的第一个菜单项
export function getFirstItemInCategory(categoryId: string): string {
  const category = getCategoryById(categoryId)
  return category?.items[0]?.id || getDefaultMenuItem()
}