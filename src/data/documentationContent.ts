export interface DocItem {
  id: string
  title: string
  summary: string
  details: string[]
}

export interface DocSection {
  id: string
  title: string
  items: DocItem[]
}

export const documentationSections: DocSection[] = [
  {
    id: 'getting-started',
    title: '快速开始',
    items: [
      {
        id: 'start-workflow',
        title: '项目创建与打开',
        summary: '从首页创建新项目或打开已有目录，必要时可将普通目录初始化为 HOICS 项目。',
        details: [
          '在首页可使用“创建新项目”或“打开项目”。',
          '打开目录后，若缺少项目元数据，应用会提示是否初始化为 HOI4 Code Studio 项目。',
          '最近项目列表会记录路径、最后打开时间、文件数量和体积信息，便于快速回到工作现场。',
          '建议首次使用时先在“设置”中配置 HOI4 游戏目录，以启用原版资源浏览和索引能力。'
        ]
      },
      {
        id: 'editor-layout',
        title: '编辑器布局与导航',
        summary: '编辑器由左侧资源面板、中间工作区、右侧信息面板组成，支持多窗格并行编辑。',
        details: [
          '左侧可切换项目文件、依赖文件和插件面板。',
          '中间工作区支持标签页和分栏编辑，可将文件移动到不同窗格查看。',
          '右侧面板包含项目信息、游戏目录、错误列表、搜索、AI 与插件页面。',
          '工具栏提供返回、启动游戏、依赖管理、打包、自动保存开关等常用操作。'
        ]
      },
      {
        id: 'shortcuts',
        title: '常用快捷键',
        summary: '应用支持常见编辑快捷键和错误跳转快捷键。',
        details: [
          'Ctrl+S：保存当前文件。',
          'Ctrl+F：打开搜索面板。',
          'Ctrl+E：跳到下一个错误。',
          'Ctrl+R：跳到上一个错误。',
          'Ctrl+Shift+T：切换主题面板。',
          '除以上快捷键外，复制、剪切、粘贴等基础操作遵循系统习惯。'
        ]
      },
      {
        id: 'file-operations',
        title: '文件与目录操作',
        summary: '支持创建、重命名、删除、复制路径和在系统资源管理器中打开。',
        details: [
          '在文件树节点或标签页上使用右键菜单进行文件操作。',
          '关闭文件前若存在未保存内容，会进行确认，避免误丢数据。',
          '从“游戏目录”面板打开的文件默认只读，用于参考原版实现。',
          '自动保存开启后，停止输入后会自动写盘，无需频繁手动保存。'
        ]
      }
    ]
  },
  {
    id: 'editing-and-preview',
    title: '编辑与预览',
    items: [
      {
        id: 'search-replace',
        title: '搜索与替换',
        summary: '支持项目、游戏目录、依赖目录范围搜索，可选正则和大小写匹配。',
        details: [
          '右侧“搜索”面板可切换搜索范围：project、game、dependencies。',
          '可开启 case sensitive 与 regex 选项进行精确匹配。',
          '点击结果后会自动打开目标文件并定位到匹配位置。',
          '替换操作会直接修改文件内容，执行前会弹出确认提示。'
        ]
      },
      {
        id: 'error-checking',
        title: '错误检查与定位',
        summary: '编辑器会收集脚本错误并在右侧错误列表中展示，支持快速跳转。',
        details: [
          '错误列表会展示行号、错误类型和信息。',
          '点击错误项可直接跳转到对应代码位置。',
          '可通过快捷键在错误之间前后跳转。',
          '设置中可关闭错误处理（不推荐，关闭后将失去实时提示）。'
        ]
      },
      {
        id: 'preview-tools',
        title: '内置预览工具',
        summary: '根据文件类型提供事件图、国策树、地图、GUI、MIO、GFX 等预览视图。',
        details: [
          '事件脚本支持事件关系图预览与节点跳转。',
          '国策脚本支持国策树预览与可视化编辑，并可回跳源文件行。',
          'map/default.map 可打开地图预览，并结合省份/州数据渲染。',
          '.gui 与 .gfx 文件支持资源解析和结构预览。',
          'MIO 文件支持 trait 关系预览与定位。',
          '图片资源（含 DDS/TGA）可直接在预览器中查看。'
        ]
      },
      {
        id: 'dependency-system',
        title: '依赖项、Tag 与 Idea 索引',
        summary: '可为项目配置依赖 Mod，并将其纳入标签与 Idea 的索引范围。',
        details: [
          '依赖管理支持添加、启用/禁用、移除与索引。',
          'Tag 与 Idea 数据会从项目、游戏目录、启用的依赖中联合加载。',
          '工具栏可打开加载监控面板，查看索引数量与刷新状态。',
          '正确配置依赖可显著提升补全、校验和阅读效率。'
        ]
      }
    ]
  },
  {
    id: 'plugin-dev',
    title: '插件开发',
    items: [
      {
        id: 'plugin-overview',
        title: '插件模型概览',
        summary: '插件通过 About.hoics 声明元数据、扩展点和权限，UI 运行在 iframe。',
        details: [
          '插件包可来自文件夹或 zip。',
          '根目录必须包含 About.hoics（JSON 格式）。',
          '插件可贡献左侧面板、右侧面板和顶部工具栏按钮。',
          '插件与宿主通过 postMessage 通信，后端调用由宿主代理执行。'
        ]
      },
      {
        id: 'plugin-about',
        title: 'About.hoics 最小示例',
        summary: '以下示例展示了插件基础字段、扩展点和命令白名单。',
        details: [
          '```json',
          '{',
          '  "id": "com.example.hello",',
          '  "name": "Hello Plugin",',
          '  "version": "0.1.0",',
          '  "main": "index.html",',
          '  "permissions": { "commands": ["load_settings", "save_settings"] },',
          '  "contributes": {',
          '    "left_sidebar": [{ "id": "hello-left", "title": "Hello" }],',
          '    "right_sidebar": [],',
          '    "toolbar": [{ "id": "open-hello", "title": "Open Hello", "open": { "side": "left", "panel": "hello-left" } }]',
          '  }',
          '}',
          '```'
        ]
      },
      {
        id: 'plugin-permission',
        title: '命令白名单与安全边界',
        summary: '插件不能任意调用后端命令，必须在 permissions.commands 中显式声明。',
        details: [
          '插件请求调用命令时，宿主会先校验白名单。',
          '未声明命令会被拒绝并返回错误。',
          '建议只开放必要命令，避免暴露高风险写入能力。',
          '安装前可使用 validate_plugin_package 进行包体校验。'
        ]
      },
      {
        id: 'plugin-message',
        title: '通信协议摘要',
        summary: '插件和宿主使用三类消息完成握手、请求和响应。',
        details: [
          '宿主 -> 插件：hoics.host.ready（包含插件上下文和 allowedCommands）。',
          '插件 -> 宿主：hoics.invoke（请求调用后端 command）。',
          '宿主 -> 插件：hoics.invoke.result（返回 success/error）。',
          '请求 id 建议全局唯一，用于正确匹配异步结果。'
        ]
      }
    ]
  },
  {
    id: 'settings-and-release',
    title: '设置、测试与发布',
    items: [
      {
        id: 'settings',
        title: '设置项说明',
        summary: '设置页面支持游戏目录、启动方式、主题、图标、字体、AI、地图性能和更新策略。',
        details: [
          '游戏目录：用于浏览原版资源、参与 Tag/Idea 索引。',
          '游戏启动：支持 Steam 模式与自定义可执行文件模式。',
          '编辑器：自动保存、字体、错误处理开关。',
          '界面：主题与文件树图标集。',
          'AI：API Key、Base URL、模型、渲染与请求选项。',
          '地图：性能模式与采样率。'
        ]
      },
      {
        id: 'package-and-launch',
        title: '启动与打包',
        summary: '编辑器支持一键启动游戏和项目打包，便于快速测试与分发。',
        details: [
          '工具栏“启动游戏”会读取当前设置并触发对应启动流程。',
          '“打包项目”会导出可分发压缩包，可选择是否排除依赖内容。',
          '在发布前建议先执行完整搜索与错误检查，避免将问题打进产物。'
        ]
      },
      {
        id: 'testing',
        title: '测试命令',
        summary: '前端和后端测试可分别执行，建议在提交前至少跑一次核心用例。',
        details: [
          '```bash',
          'npm run test:run',
          'npm run test:coverage',
          '```',
          '```bash',
          'cd src-tauri',
          'cargo test',
          '```'
        ]
      }
    ]
  }
]
