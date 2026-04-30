/**
 * cwtools CodeMirror Linter 集成
 * 
 * 提供完整的 cwtools 验证系统与 CodeMirror 6 的集成，包括：
 * - 实时语法验证
 * - 错误/警告标记和装饰
 * - 悬停提示
 * - Error Lens 行尾提示
 */

import { linter, lintGutter, type Diagnostic } from '@codemirror/lint'
import { EditorView, Decoration, type DecorationSet, hoverTooltip, WidgetType } from '@codemirror/view'
import { StateField, type Extension, type EditorState } from '@codemirror/state'
import { 
  validateScript, 
  type CWToolsDiagnostic, 
  toCodeMirrorDiagnostics,
  getDiagnosticsAtPosition,
  formatDiagnosticsForHover
} from './cwtoolsValidator'

/**
 * cwtools linter 配置选项
 */
export interface CWToolsLinterOptions {
  /** 获取当前文件路径 */
  getFilePath: () => string | undefined
  /** 获取当前文件版本号 */
  getVersion: () => number
  /** 获取项目根目录 */
  getProjectRoot?: () => string | undefined
  /** 获取游戏根目录 */
  getGameRoot?: () => string | undefined
  /** 验证延迟（毫秒） */
  delay?: number
  /** 是否启用 Error Lens */
  enableErrorLens?: boolean
  /** 是否启用行级装饰 */
  enableLineDecoration?: boolean
}

/**
 * 存储最新的诊断信息（用于悬停提示和装饰）
 * 使用 WeakMap 按 EditorView 实例隔离，避免多编辑器竞态
 */
const diagnosticsMap = new WeakMap<EditorView, CWToolsDiagnostic[]>()
let latestDiagnostics: CWToolsDiagnostic[] = []

/**
 * 创建 cwtools linter 扩展
 * 
 * @param options - 配置选项
 * @returns CodeMirror 扩展数组
 */
export function createCWToolsLinter(options: CWToolsLinterOptions): Extension[] {
  const { 
    getFilePath, 
    getVersion, 
    delay = 300,
    enableErrorLens = true,
    enableLineDecoration = true
  } = options
  
  // 验证函数
  const validator = async (view: EditorView): Promise<Diagnostic[]> => {
    try {
      const content = view.state.doc.toString()
      const filePath = getFilePath()
      const version = getVersion()
      
      const response = await validateScript(content, filePath, version)
      diagnosticsMap.set(view, response.diagnostics)
      latestDiagnostics = response.diagnostics
      
      return toCodeMirrorDiagnostics(response.diagnostics)
    } catch (error) {
      console.error('cwtools 验证失败:', error)
      diagnosticsMap.set(view, [])
      latestDiagnostics = []
      return []
    }
  }
  
  const extensions: Extension[] = [
    // 基础 linter
    linter(validator, { delay }),
    lintGutter({ hoverTime: 200 }),
    
    // 悬停提示
    hoverTooltip((view, pos) => {
      const diagnostics = diagnosticsMap.get(view) ?? latestDiagnostics
      const diagnosticsAtPos = getDiagnosticsAtPosition(diagnostics, pos)
      
      if (diagnosticsAtPos.length === 0) return null
      
      const text = formatDiagnosticsForHover(diagnosticsAtPos)
      
      return {
        pos,
        above: true,
        create() {
          const dom = document.createElement('div')
          dom.className = 'cm-cwtools-hover'
          dom.textContent = text
          return { dom }
        }
      }
    }),
    
    // 主题样式
    EditorView.baseTheme({
      '.cm-lintRange.cm-lintRange-error': {
        backgroundColor: 'rgba(255, 0, 0, 0.3)'
      },
      '.cm-lintRange.cm-lintRange-warning': {
        backgroundColor: 'rgba(255, 255, 0, 0.3)'
      },
      '.cm-lintRange.cm-lintRange-info': {
        backgroundColor: 'rgba(0, 150, 255, 0.2)'
      },
      '.cm-cwtools-hover': {
        padding: '8px 12px',
        backgroundColor: 'rgba(30, 30, 30, 0.95)',
        color: '#ffffff',
        borderRadius: '6px',
        fontSize: '13px',
        lineHeight: '1.5',
        maxWidth: '400px',
        whiteSpace: 'pre-wrap',
        boxShadow: '0 4px 12px rgba(0, 0, 0, 0.4)',
        backdropFilter: 'blur(10px)'
      },
      '.cm-error-line': {
        backgroundColor: 'rgba(255, 0, 0, 0.15)'
      },
      '.cm-warning-line': {
        backgroundColor: 'rgba(255, 255, 0, 0.1)'
      },
      '.cm-error-lens': {
        background: 'rgba(255, 0, 0, 0.9)',
        color: '#ffffff',
        fontSize: '12px',
        padding: '0 6px',
        marginLeft: '8px',
        borderRadius: '4px',
        lineHeight: '1.6',
        whiteSpace: 'pre-wrap',
        pointerEvents: 'auto'
      },
      '.cm-warning-lens': {
        background: 'rgba(255, 204, 0, 0.9)',
        color: '#ffffff',
        fontSize: '12px',
        padding: '0 6px',
        marginLeft: '8px',
        borderRadius: '4px',
        lineHeight: '1.6',
        whiteSpace: 'pre-wrap',
        pointerEvents: 'auto'
      }
    })
  ]
  
  // 添加行级装饰
  if (enableLineDecoration) {
    extensions.push(createErrorLineField())
  }
  
  // 添加 Error Lens
  if (enableErrorLens) {
    extensions.push(createErrorLensField())
  }
  
  return extensions
}

/**
 * 创建错误行装饰字段
 */
function createErrorLineField(): Extension {
  const errorLineMark = Decoration.line({ attributes: { class: 'cm-error-line' } })
  const warningLineMark = Decoration.line({ attributes: { class: 'cm-warning-line' } })
  
  return StateField.define<DecorationSet>({
    create(state: EditorState) {
      return getLineDecorations(state)
    },
    update(value, tr) {
      value = value.map(tr.changes)
      if (tr.docChanged) {
        return getLineDecorations(tr.state)
      }
      return value
    },
    provide: f => EditorView.decorations.from(f)
  })
  
  function getLineDecorations(state: EditorState) {
    const deco: ReturnType<typeof errorLineMark.range>[] = []
    
    // 记录每行最高的严重程度
    const lineSeverity = new Map<number, 'error' | 'warning'>()
    
    for (const d of latestDiagnostics) {
      const line = state.doc.lineAt(d.range.start.offset)
      const current = lineSeverity.get(line.number)
      const severity = d.severity === 'error' ? 'error' : 'warning'
      
      if (!current) {
        lineSeverity.set(line.number, severity)
      } else if (current === 'warning' && severity === 'error') {
        lineSeverity.set(line.number, 'error')
      }
    }
    
    for (const [lineNo, severity] of lineSeverity) {
      const line = state.doc.line(lineNo)
      if (severity === 'error') {
        deco.push(errorLineMark.range(line.from))
      } else {
        deco.push(warningLineMark.range(line.from))
      }
    }
    
    return Decoration.set(deco, true)
  }
}

/**
 * Error Lens 小部件
 */
class LensWidget extends WidgetType {
  constructor(
    readonly text: string, 
    readonly severity: 'error' | 'warning'
  ) { 
    super() 
  }
  
  eq(other: LensWidget) { 
    return other.text === this.text && other.severity === this.severity 
  }
  
  toDOM() {
    const span = document.createElement('span')
    span.className = this.severity === 'error' ? 'cm-error-lens' : 'cm-warning-lens'
    span.textContent = this.text
    return span
  }
}

/**
 * 创建 Error Lens 装饰字段
 */
function createErrorLensField(): Extension {
  return StateField.define<DecorationSet>({
    create(state: EditorState) {
      return getLensDecorations(state)
    },
    update(value, tr) {
      value = value.map(tr.changes)
      if (tr.docChanged) {
        return getLensDecorations(tr.state)
      }
      return value
    },
    provide: f => EditorView.decorations.from(f)
  })
  
  function getLensDecorations(state: EditorState) {
    const byLine = new Map<number, { msgs: string[], severity: 'error' | 'warning' }>()
    
    for (const d of latestDiagnostics) {
      const line = state.doc.lineAt(d.range.start.offset)
      const current = byLine.get(line.number) || { msgs: [], severity: 'warning' as const }
      
      current.msgs.push(d.message)
      if (d.severity === 'error') {
        current.severity = 'error'
      }
      
      byLine.set(line.number, current)
    }
    
    const deco: ReturnType<ReturnType<typeof Decoration.widget>['range']>[] = []
    for (const [lineNo, data] of byLine) {
      const line = state.doc.line(lineNo)
      const msg = data.msgs.slice(0, 3).join(' • ') + 
                  (data.msgs.length > 3 ? ` (+${data.msgs.length - 3})` : '')
      deco.push(
        Decoration.widget({ 
          widget: new LensWidget(msg, data.severity), 
          side: 1 
        }).range(line.to)
      )
    }
    
    return Decoration.set(deco, true)
  }
}
