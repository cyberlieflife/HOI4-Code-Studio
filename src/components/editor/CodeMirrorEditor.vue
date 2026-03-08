<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { EditorState } from '@codemirror/state'
import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter } from '@codemirror/view'
import { defaultKeymap, history, historyKeymap, indentMore, indentLess } from '@codemirror/commands'
import { bracketMatching, indentOnInput, indentUnit } from '@codemirror/language'
import { closeBrackets, autocompletion, type CompletionContext } from '@codemirror/autocomplete'
import { json } from '@codemirror/lang-json'
import { yaml } from '@codemirror/lang-yaml'
import { javascript } from '@codemirror/lang-javascript'
import { hoi4 } from '../../lang/hoi4'
// import { createLinter } from '../../utils/ErrorTip' // 旧版错误检测系统（已废弃，已被 cwtools 替代）
import { createCWToolsLinter } from '../../utils/cwtoolsLinter' // 新版 cwtools 验证系统
import { setIdeaRoots, ensureIdeaRegistry } from '../../composables/useIdeaRegistry'
import { useGrammarCompletion } from '../../composables/useGrammarCompletion'
import { rainbowBrackets, rainbowTheme } from './rainbowBrackets'
import { useEditorTheme } from '../../composables/useEditorTheme'
import { useRGBColorDisplay } from '../../composables/useRGBColorDisplay'
import { useEditorFont } from '../../composables/useEditorFont'

const props = defineProps<{
  content: string
  fileName?: string
  filePath?: string
  projectRoot?: string
  gameDirectory?: string
  disableErrorHandling?: boolean
}>()

const emit = defineEmits<{
  'update:content': [value: string]
  cursorChange: [line: number, column: number]
  scroll: []
  contextmenu: [event: MouseEvent]
}>()

const editorContainer = ref<HTMLDivElement | null>(null)
let editorView: EditorView | null = null
let currentCoreExtensions: any[] = []
let fileVersion = ref(0) // 文件版本号，用于增量验证

// GrammarCompletion 组合式，提供统一的补全项视图
const { allItems } = useGrammarCompletion()

// 编辑器主题
const { editorThemeVersion, getCurrentEditorTheme } = useEditorTheme()

// RGB颜色显示
const { 
  createRGBColorField, 
  loadSettingsFromStorage,
  getEnabled,
} = useRGBColorDisplay()

// 编辑器字体设置
const { 
  fontConfig, 
  fontConfigVersion, 
  createEditorFontTheme 
} = useEditorFont()

/**
 * 基于 CodeMirror CompletionContext 的补全源
 * 通过匹配前缀筛选 GrammarCompletion 项目，并在显式触发时展示全集
 */
function grammarCompletionSource(context: CompletionContext) {
  const word = context.matchBefore(/[A-Za-z0-9_\.\-]+/)
  if (!word) {
    if (!context.explicit) {
      return null
    }
    return {
      from: context.pos,
      options: allItems.value
    }
  }

  const prefix = word.text.toUpperCase()
  const filtered = allItems.value.filter((item) => item.label.toUpperCase().startsWith(prefix))

  if (filtered.length === 0 && !context.explicit) {
    return null
  }

  return {
    from: word.from,
    options: filtered.length > 0 ? filtered : allItems.value
  }
}

// 根据文件名获取语言扩展
function getLanguageExtension() {
  if (!props.fileName) return []
  
  const ext = props.fileName.split('.').pop()?.toLowerCase()
  
  switch (ext) {
    case 'json':
      return [json()]
    case 'yaml':
    case 'yml':
      return [yaml()]
    case 'js':
    case 'ts':
      return [javascript()]
    case 'txt':
      // HOI4 脚本 - 仅加载必要的扩展，延迟加载耗时的扩展
      return [
        hoi4(),
        rainbowBrackets,
        rainbowTheme,
        // 延迟设置 Idea 注册表根并触发扫描，避免阻塞编辑器初始化
        EditorView.updateListener.of((_update) => {
          // 只在首次加载时执行一次
          setIdeaRoots(props.projectRoot, props.gameDirectory)
          ensureIdeaRegistry()
          // 移除监听器，避免重复执行
          return EditorView.updateListener.of(() => {})
        })
      ]
    default:
      return []
  }
}

// 自定义换行后自动缩进的扩展
const autoIndentOnEnter = EditorView.domEventHandlers({
  keydown: (event, view) => {
    if (event.key === 'Enter' && !event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) {
      const { state } = view
      const { from, to } = state.selection.main
      
      // 获取当前行
      const line = state.doc.lineAt(from)
      const lineText = line.text
      
      // 获取当前行的缩进
      const indent = lineText.match(/^\s*/)?.[0] || ''
      
      // 检查是否在大括号、中括号等后面
      const beforeCursor = lineText.slice(0, from - line.from).trim()
      const afterCursor = lineText.slice(to - line.from).trim()
      
      let extraIndent = ''
      let needsExtraLine = false
      
      // 如果光标前是开括号 { [ (，增加缩进
      if (beforeCursor.endsWith('{') || beforeCursor.endsWith('[') || beforeCursor.endsWith('(')) {
        extraIndent = '    ' // 4个空格
        
        // 如果光标后是闭括号，需要在中间插入额外的空行
        if (afterCursor.startsWith('}') || afterCursor.startsWith(']') || afterCursor.startsWith(')')) {
          needsExtraLine = true
        }
      }
      
      // 构建要插入的文本
      let insertText = '\n' + indent + extraIndent
      
      if (needsExtraLine) {
        insertText += '\n' + indent
      }
      
      // 插入换行和缩进
      view.dispatch({
        changes: { from, to, insert: insertText },
        selection: { anchor: from + insertText.length - (needsExtraLine ? indent.length : 0) },
        scrollIntoView: true
      })
      
      event.preventDefault()
      return true
    }
    return false
  }
})

// 自定义 Tab 键处理，支持多行缩进
const smartTab = keymap.of([
  {
    key: 'Tab',
    run: (view) => {
      const { state } = view
      const { from, to } = state.selection.main
      
      // 如果有选中文本，缩进所有选中的行
      if (from !== to) {
        return indentMore(view)
      }
      
      // 否则插入缩进
      view.dispatch(state.update(state.replaceSelection('    '), { scrollIntoView: true }))
      return true
    }
  },
  {
    key: 'Shift-Tab',
    run: indentLess
  }
])

// 初始化编辑器
async function initEditor() {
  if (!editorContainer.value) return
  // 清空容器，避免残留的旧编辑器 DOM 节点导致多个滚动条/多实例叠加
  editorContainer.value.innerHTML = ''
  
  // 构建编辑器扩展 - 仅包含核心功能
  const coreExtensions: any[] = [
    lineNumbers(),
    highlightActiveLine(),
    highlightActiveLineGutter(),
    history(),
    bracketMatching(),
    closeBrackets(),
    indentOnInput(),
    indentUnit.of('    '), // 4 spaces
    EditorView.lineWrapping,
    EditorView.editable.of(true),
    autoIndentOnEnter,
    smartTab,
    keymap.of([...defaultKeymap, ...historyKeymap]),
    EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        const newContent = update.state.doc.toString()
        emit('update:content', newContent)
        // 文档变更时增加版本号
        fileVersion.value++
      }
      
      if (update.selectionSet) {
        const pos = update.state.selection.main.head
        const line = update.state.doc.lineAt(pos)
        const lineNumber = line.number
        const column = pos - line.from + 1
        emit('cursorChange', lineNumber, column)
      }
    }),
    EditorView.domEventHandlers({
      scroll: () => {
        emit('scroll')
      },
      contextmenu: (event: MouseEvent) => {
        event.preventDefault()
        emit('contextmenu', event)
        return true
      }
    }),
    // 添加主题设置
    getCurrentEditorTheme(),
    // 添加语言扩展（包含语法高亮）
    ...getLanguageExtension(),
    // 添加字体设置，确保覆盖语法高亮样式
    createEditorFontTheme(fontConfig.value)
  ]
  
  // 保存核心扩展到全局变量，以便在延迟加载时使用
  currentCoreExtensions = coreExtensions
  
  const startState = EditorState.create({
    doc: props.content,
    extensions: coreExtensions
  })
  
  editorView = new EditorView({
    state: startState,
    parent: editorContainer.value
  })
  
  // 延迟加载非核心功能，避免阻塞编辑器初始化
  setTimeout(async () => {
    if (!editorView) return
    
    // 加载RGB颜色显示设置
    await loadSettingsFromStorage()
    
    // 条件性添加RGB颜色显示扩展
    if (getEnabled()) {
      // 重新创建编辑器视图，添加RGB颜色显示扩展
      const newExtensions = [...currentCoreExtensions, createRGBColorField()]
      const newState = EditorState.create({
        doc: editorView.state.doc.toString(),
        extensions: newExtensions
      })
      
      editorView.destroy()
      editorView = new EditorView({
        state: newState,
        parent: editorContainer.value!
      })
    }
    
    // 延迟添加自动补全和Linter功能（仅对HOI4脚本文件）
    const ext = props.fileName?.split('.').pop()?.toLowerCase()
    if (ext === 'txt') {
      setTimeout(() => {
        if (!editorView) return
        
        // 重新创建编辑器视图，添加自动补全功能
        const newExtensions = [...currentCoreExtensions, 
          autocompletion({
            override: [grammarCompletionSource]
          })
        ]
        
        // 根据disableErrorHandling属性决定是否添加Linter功能
        if (!props.disableErrorHandling) {
          // 使用新的 cwtools 验证系统
          const cwtoolsLinter = createCWToolsLinter({
            getFilePath: () => props.filePath,
            getVersion: () => fileVersion.value,
            getProjectRoot: () => props.projectRoot,
            getGameRoot: () => props.gameDirectory,
            delay: 300,
            enableErrorLens: true,
            enableLineDecoration: true
          })
          newExtensions.push(...cwtoolsLinter)
        }
        
        // 如果已添加RGB颜色显示扩展，也要包含进去
        if (getEnabled()) {
          newExtensions.push(createRGBColorField())
        }
        
        const newState = EditorState.create({
          doc: editorView.state.doc.toString(),
          extensions: newExtensions
        })
        
        editorView.destroy()
        editorView = new EditorView({
          state: newState,
          parent: editorContainer.value!
        })
      }, 500)
    }
  }, 100)
}

// 监听内容变化
watch(() => props.content, (newContent) => {
  if (!editorView) return
  
  const currentContent = editorView.state.doc.toString()
  if (currentContent !== newContent) {
    const transaction = editorView.state.update({
      changes: { from: 0, to: currentContent.length, insert: newContent }
    })
    editorView.dispatch(transaction)
    // 外部内容变更时重置版本号
    fileVersion.value = 0
  }
})

// 监听文件名变化（切换语言）
watch(() => props.fileName, () => {
  if (!editorView) return
  
  // 重新初始化编辑器以应用新的语言扩展
  editorView.destroy()
  nextTick(() => {
    initEditor()
  })
})

// 监听文件路径或项目根变化（刷新 Linter 上下文）
watch(() => props.filePath, () => {
  if (!editorView) return
  fileVersion.value = 0 // 重置版本号
  editorView.destroy()
  nextTick(() => {
    initEditor()
  })
})

watch(() => props.projectRoot, () => {
  if (!editorView) return
  fileVersion.value = 0 // 重置版本号
  editorView.destroy()
  nextTick(() => {
    initEditor()
  })
})

watch(() => props.gameDirectory, () => {
  if (!editorView) return
  fileVersion.value = 0 // 重置版本号
  editorView.destroy()
  nextTick(() => {
    initEditor()
  })
})

// 监听主题变化，重新初始化编辑器
watch(editorThemeVersion, () => {
  if (!editorView) return
  editorView.destroy()
  nextTick(() => {
    initEditor()
  })
})

// 监听字体设置变化，重新初始化编辑器
watch(fontConfigVersion, () => {
  if (!editorView) return
  editorView.destroy()
  nextTick(() => {
    initEditor()
  })
})

onMounted(() => {
  initEditor()
})

onBeforeUnmount(() => {
  if (editorView) {
    editorView.destroy()
    editorView = null
  }
})

defineExpose({
  getEditorView: () => editorView,
  getSelectedText: () => {
    if (!editorView) return ''
    const selection = editorView.state.selection.main
    return editorView.state.doc.sliceString(selection.from, selection.to)
  },
  insertText: (text: string) => {
    if (!editorView) return
    const selection = editorView.state.selection.main
    editorView.dispatch({
      changes: { from: selection.from, to: selection.to, insert: text }
    })
  },
  getCursorPosition: () => {
    if (!editorView) return { line: 1, column: 1 }
    const pos = editorView.state.selection.main.head
    const line = editorView.state.doc.lineAt(pos)
    return {
      line: line.number,
      column: pos - line.from + 1
    }
  },
  cutSelection: () => {
    if (!editorView) return ''
    const selection = editorView.state.selection.main
    const text = editorView.state.doc.sliceString(selection.from, selection.to)
    editorView.dispatch({
      changes: { from: selection.from, to: selection.to, insert: '' }
    })
    return text
  },
  copySelection: () => {
    if (!editorView) return ''
    const selection = editorView.state.selection.main
    return editorView.state.doc.sliceString(selection.from, selection.to)
  },
  selectAll: () => {
    if (!editorView) {
      return
    }
    
    // 先聚焦编辑器
    editorView.focus()
    
    // 使用 CodeMirror 的内置 selectAll 命令
    import('@codemirror/commands').then(({ selectAll }) => {
      if (!editorView) return
      selectAll({
        state: editorView.state,
        dispatch: editorView.dispatch.bind(editorView)
      })
    }).catch(() => {
      // 备用方案：手动选择
      if (!editorView) return
      const doc = editorView.state.doc
      editorView.dispatch({
        selection: {
          anchor: 0,
          head: doc.length
        },
        scrollIntoView: true
      })
    })
  }
})
</script>

<template>
  <div 
    ref="editorContainer" 
    class="codemirror-editor w-full h-full"
  ></div>
</template>

<style>
.codemirror-editor {
  height: 100%;
  overflow: hidden;
}

.codemirror-editor .cm-editor {
  height: 100%;
  border-radius: 0.75rem;
  box-shadow: 0 14px 30px rgba(0, 0, 0, 0.45);
}

.codemirror-editor .cm-scroller {
  overflow: auto;
}

.codemirror-editor .cm-tooltip.cm-tooltip-autocomplete {
  border-radius: 0.75rem;
  padding: 0.25rem;
  box-shadow: 0 18px 40px rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(10px);
}

.codemirror-editor .cm-tooltip.cm-tooltip-autocomplete ul {
  padding: 0.125rem;
}

.codemirror-editor .cm-tooltip.cm-tooltip-autocomplete li {
  border-radius: 0.5rem;
  margin: 0.125rem 0;
  padding: 0.125rem 0.375rem;
}

.codemirror-editor .cm-tooltip.cm-tooltip-autocomplete .cm-completionLabel {
  font-weight: 500;
}

.codemirror-editor .cm-tooltip.cm-tooltip-autocomplete .cm-completionDetail {
  opacity: 0.7;
}
</style>
