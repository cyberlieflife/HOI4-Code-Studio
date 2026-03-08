/**
 * 更新日志数据
 * 每个版本包含版本号、日期和更新内容列表
 */

export interface ChangelogItem {
  type: 'feature' | 'fix' | 'improvement' | 'other'
  content: string
}

export interface VersionLog {
  version: string
  date?: string
  description?: string
  changes: ChangelogItem[]
}

export const changelog: VersionLog[] = [
  {
    version: 'v0.3.4-dev',
    description: '优化编辑器',
    changes: [
      { type: 'improvement', content: '删除游戏目录的只读限制' },
      { type: 'improvement', content: '优化加载速度' },
    ]
  },
  {
    version: 'v0.3.3-dev',
    description: '更改项目地址',
    changes: [
      { type: 'improvement', content: '更改项目地址' },
    ]
  },
  {
    version: 'v0.3.2-dev',
    description: '回滚...',
    changes: [
      { type: 'feature', content: '回滚' },
    ]
  },
  {
    version: 'v0.3.1-dev',
    description: '优化UI',
    changes: [
      { type: 'improvement', content: '合并主界面和最近项目页' },
    ]
  },
  {
    version: 'v0.3.0-dev',
    description: '支持插件系统',
    changes: [
      { type: 'feature', content: '支持基本插件功能' },
      { type: 'improvement', content: '文档界面支持代码块展示' },
    ]
  },
  {
    version: 'v0.2.20-dev',
    description: '移植Vscode HOI工具插件',
    changes: [
      { type: 'feature', content: '支持MIO军工组织预览' },
    ]
  },
  {
    version: 'v0.2.19-dev',
    description: '移植Vscode HOI工具插件',
    changes: [
      { type: 'feature', content: '支持Map预览' },
      { type: 'feature', content: '支持GUI预览' },
    ]
  },
  {
    version: 'v0.2.18-dev',
    description: 'Modifier、AI更新',
    changes: [
      { type: 'feature', content: '支持Modifier速查表功能' },
      { type: 'feature', content: 'AI支持Todo功能' },
      { type: 'improvement', content: '优化AI运行' },
      { type: 'feature', content: '增加AI提示词优化功能' },
    ]
  },
  {
    version: 'v0.2.17-dev',
    description: '国策!',
    changes: [
      { type: 'feature', content: '国策预览升级为内置国策编辑器' }
    ]
  },
  {
    version: 'v0.2.16-dev',
    description: '国策和优化',
    changes: [
      { type: 'improvement', content: '国策预览支持本地化国策名显示' },
      { type: 'improvement', content: '搜索/国策预览跳行现在会跳转到第一行而非最后一行' },
      { type: 'feature', content: '国策预览支持左键转跳源码' },
      { type: 'feature', content: '国策预览支持缩略显示国策信息' },
      { type: 'fix', content: '修复国策预览图标缓存无法使用的问题' },
    ]
  },
  {
    version: 'v0.2.15-dev',
    description: '编辑页 UI 岛屿无边框化',
    changes: [
      { type: 'improvement', content: '优化编辑页 UI 样式为无边框设计' },
      { type: 'improvement', content: '同步更新项目信息模块内置岛为无边框样式' },
    ]
  },
  {
    version: 'v0.2.14-dev',
    description: '紧急修复',
    changes: [
      { type: 'improvement', content: '优化样式' },
    ]
  },
  {
    version: 'v0.2.13-dev',
    description: '图片加载修复',
    changes: [
      { type: 'fix', content: '修复图片预览高分辨率图片时会出现图片加载错误的问题' },
    ]
  },
  {
    version: 'v0.2.12-dev',
    description: '更新提示优化',
    changes: [
      { type: 'feature', content: '更新提示时增加最新的更新内容' },
    ]
  },
  {
    version: 'v0.2.11-dev',
    description: '主页面新增按钮、AI功能',
    changes: [
      { type: 'feature', content: '主页面增加issue和pr直达按钮' },
      { type: 'feature', content: '加入未完善的AI AGENT功能' },
      { type: 'feature', content: '加入文件/文件夹右键删除功能' },
      { type: 'improvement', content: '新建文件支持withBOM' },
      { type: 'improvement', content: '搜索支持替换功能' },
      { type: 'fix', content: '修复依赖项数字显示问题' },
    ]
  },
  {
    version: 'v0.2.10-dev',
    description: '最近项目搜索、界面优化、设置优化、图片预览移动',
    changes: [
      { type: 'feature', content: '最近项目增加搜索功能' },
      { type: 'feature', content: '图片预览缩放移动功能' },
      { type: 'improvement', content: '优化项目信息显示UI' },
      { type: 'improvement', content: '优化字体粗细选择' },
    ]
  },
  {
    version: 'v0.2.9-dev',
    description: '右键优化、界面优化、修复问题',
    changes: [
      { type: 'feature', content: '编辑页右键增加全选功能' },
      { type: 'improvement', content: '优化文档UI' },
      { type: 'improvement', content: '优化更新日志UI' },
      { type: 'improvement', content: '优化设置UI' },
      { type: 'fix', content: '修复部分国策树无法打开的bug' },
      { type: 'fix', content: '修复图标无法加载的bug' },
      { type: 'fix', content: '修复搜索点击结果有时无法跳转到对应行的问题' }
    ]
  },
  {
    version: 'v0.2.8-dev',
    description: '更多主题、一键启动支持调试',
    changes: [
      { type: 'improvement', content: '增加更多主题' },
      { type: 'feature', content: '一键启动支持调试启动(学习版未验证)' },
      { type: 'feature', content: '增加图标选择功能(未完善)' },
    ]
  },
  {
    version: 'v0.2.7-dev',
    description: '搜索重构、字体修复、RGB优化',
    changes: [
      { type: 'improvement', content: '重构搜索功能UI、移动到侧边栏、增加搜索范围选项、增加类型限制' },
      { type: 'improvement', content: '修复字体加载问题' },
      { type: 'improvement', content: '优化RGB显示的交互' },
      ]
  },
  {
    version: 'v0.2.6-dev',
    description: '字体、RGB、优化加载',
    changes: [
      { type: 'feature', content: '删除便捷模式设置存储方式' },
      { type: 'feature', content: '增加RGB颜色显示功能' },
      { type: 'feature', content: '增加字体修改初步支持' },
      { type: 'improvement', content: '优化启动速度' },
      ]
  },
  {
    version: 'v0.2.5-dev',
    description: '主题与功能增强',
    changes: [
      { type: 'feature', content: '实现预览文件内容同步和图标缓存功能' },
      { type: 'feature', content: '为依赖项添加类型图标显示' },
      { type: 'feature', content: '添加禁用错误处理的配置选项' },
      { type: 'feature', content: '为侧边栏和面板切换添加过渡动画效果' },
      { type: 'feature', content: '改进主题相关组件的样式和交互' },
      { type: 'feature', content: '添加页面过渡动画效果' },
      { type: 'fix', content: '修复主题设置持久化' },
      { type: 'feature', content: '新增多种代码主题配色方案' }
    ]
  },
  {
    version: 'v0.2.4-dev',
    description: '主题系统更新',
    changes: [
      { type: 'feature', content: '实现主题系统，支持7种内置主题切换' }
    ]
  },
  {
    version: 'v0.2.3-dev',
    description: '自动保存和文档功能',
    changes: [
      { type: 'feature', content: '支持自动保存' },
      { type: 'feature', content: '添加程序内置文档' }
    ]
  },
  {
    version: 'v0.2.2-dev',
    description: '启动器支持',
    changes: [
      { type: 'feature', content: '支持学习版dowser.exe/hoi4.exe双切换启动' }
    ]
  },
  {
    version: 'v0.2.1-dev',
    description: '修复与优化',
    changes: [
      { type: 'fix', content: '修复了Ctrl+S无法保存问题' },
      { type: 'improvement', content: '禁用浏览器自带右键，依然可用F12打开开发者工具' },
      { type: 'feature', content: '增加游戏目录提醒弹窗' },
      { type: 'feature', content: '支持Tab' }
    ]
  },
  {
    version: 'v0.2.0-dev',
    description: '大版本更新 - 可视化预览与编辑增强',
    changes: [
      { type: 'feature', content: '实现国策树可视化预览功能，支持从.gfx文件加载国策图标' },
      { type: 'feature', content: '添加事件关系图预览功能，支持在分割窗格中可视化事件依赖' },
      { type: 'feature', content: '实现窗格间文件移动功能，优化预览窗格复用逻辑' },
      { type: 'feature', content: '实现编辑器右键菜单，支持复制/剪切/粘贴和模板插入功能' },
      { type: 'feature', content: '添加权力平衡(BOP)模板插入功能，优化模板菜单显示逻辑' },
      { type: 'feature', content: '添加Tag初始态定义模板插入功能，支持在history/countries目录下使用' },
      { type: 'feature', content: '实现彩虹括号功能，优化语法高亮和值标识符识别' },
      { type: 'feature', content: '实现文件/文件夹重命名功能，优化右键菜单和路径操作' },
      { type: 'feature', content: '添加错误导航快捷键，支持在编辑器中快速跳转错误' },
      { type: 'feature', content: '添加编辑器错误状态同步机制' },
      { type: 'improvement', content: '重构错误提示系统，引入词法分析器和警告级别支持' },
      { type: 'improvement', content: '优化错误显示UI，实现错误列表跳转到代码行功能' },
      { type: 'improvement', content: '统一错误提示颜色方案，使用纯红色替代橙红色' },
      { type: 'feature', content: '添加 DDS 和 TGA 图片格式支持，实现自动转换为 PNG' }
    ]
  },
  {
    version: 'v0.1.3-dev',
    description: '打包与图片预览',
    changes: [
      { type: 'feature', content: '新增项目一键打包功能' },
      { type: 'feature', content: '实现图片文件预览功能，支持 PNG、JPG 等格式（tga dds暂未支持）' },
      { type: 'improvement', content: '替换所有应用图标文件（PNG、ICO、ICNS 格式）' },
      { type: 'improvement', content: '在主页和编辑器工具栏添加应用图标显示' },
      { type: 'feature', content: '新增加载监控面板功能，显示已加载的 Tag 和 Idea 数量' },
      { type: 'improvement', content: '优化编辑器左侧面板，移除冗余的加载状态提示' },
      { type: 'improvement', content: '在工具栏添加加载监控按钮' },
      { type: 'feature', content: '新增依赖项管理模块' },
      { type: 'improvement', content: '优化项目信息显示' },
      { type: 'fix', content: '修复编辑器误标记文件为未保存状态的问题' }
    ]
  },
  {
    version: 'v0.1.2-dev',
    description: '配置与更新系统',
    changes: [
      { type: 'feature', content: '实现便携模式和 AppData 模式的配置文件管理' },
      { type: 'feature', content: '添加自动更新检查功能，支持启动时检查更新' },
      { type: 'feature', content: '新增最近项目显示布局设置（四列/三列/两列/单列/瀑布流）' },
      { type: 'improvement', content: '优化设置界面，移除无用选项，增加配置位置选择' },
      { type: 'improvement', content: '改进最近项目页面，支持响应式布局和多种显示模式' },
      { type: 'feature', content: '添加版本信息显示和手动检查更新功能' },
      { type: 'feature', content: '实现外部链接打开功能' }
    ]
  },
  {
    version: 'v0.1.1-dev',
    description: 'UI改进',
    changes: [
      { type: 'improvement', content: '改进UI' }
    ]
  },
  {
    version: 'v0.1.0-dev',
    description: '分页功能',
    changes: [
      { type: 'feature', content: '分页功能' }
    ]
  },
  {
    version: 'v0.0.1-dev',
    description: '初始版本',
    changes: [
      { type: 'feature', content: '基本的代码高亮配色(OneDark)' },
      { type: 'feature', content: '搜索功能' },
      { type: 'feature', content: '简单的语法错误提示(只实现了部分提示且不完善)' },
      { type: 'feature', content: '关键字、Idea、Tag补全（按照秋起图书馆代码提词器为蓝本添加）' },
      { type: 'feature', content: '项目内查看游戏目录' },
      { type: 'other', content: 'HOI4CS项目（按项目管理打开的Mod，类似Vscode的工作区）' }
    ]
  }
]
