import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import MarkdownIt from 'markdown-it'
import { loadSettings, readDirectory, readFileContent, saveSettings, writeFileContent } from '../api/tauri'
import { logger } from '../utils/logger'
import { useTodo } from './useTodo'
import type { ChatMessage, ChatRole, ChatSession, ToolCall } from '../types/aiChat'

export function useAiChat() {
  const MAX_CONTINUE_TURNS = 200

  const { updateTodos: _updateTodos, todos } = useTodo()

  function updateTodos(ts: any[]) {
    _updateTodos(ts)
    syncMessagesToCurrentSession()
  }

  watch(todos, () => {
    syncMessagesToCurrentSession()
  }, { deep: true })

  const messages = ref<ChatMessage[]>([])
  const input = ref('')
  const isSending = ref(false)
  const isOptimizing = ref(false)

  const settingsOpen = ref(false)
  const historyOpen = ref(false)
  const openaiApiKey = ref('')
  const openaiBaseUrl = ref('https://api.openai.com')
  const openaiModel = ref('gpt-4o-mini')
  const renderMarkdown = ref(false)
  const requestReasoning = ref(false)

  const SYSTEM_PROMPT = `你是 HOI4 Code Studio 内置 AI 助手。请根据用户提供的上下文与问题，给出准确、可执行的建议与修改方案。

【工具调用（你可以直接调用）】
你可以在回复中输出一个或多个工具块，前端会自动执行并把结果追加到对话上下文中。
**绝对禁止在回复中自行输出 TOOL_RESULT 格式的内容和他后续的内容，那是系统自动生成的，你只需输出工具块即可。**
工具块格式必须严格如下：
列出目录:
单文件:
\`\`\`tool
{"tool":"list_dir","path":绝对路径,"start":1,"end":-1}
\`\`\`
多文件:
\`\`\`tool
{"tool":"list_dir","paths":["相对或绝对路径1","相对或绝对路径2"],"start":1,"end":-1}
\`\`\`
读取文件:
单文件:
\`\`\`tool
{"tool":"read_file","path":绝对路径,"start":1,"end":-1}
多文件:
\`\`\`tool
{"tool":"read_file","paths":["相对或绝对路径1","相对或绝对路径2"],"start":1,"end":-1}
\`\`\`

\`\`\`tool
{"tool":"edit_file","path":"绝对路径","content":"新的完整文件内容","confirm":true}
\`\`\`

\`\`\`tool
{"tool":"search_field","query":"要搜索的字段","file_type":"*.yml,*.txt","case_insensitive":true,"regex":false,"with_snippet":true,"snippet_max_len":160}
\`\`\`

\`\`\`tool
{"tool":"update_todos","todos":[{"id":"任务ID","content":"任务内容","status":"pending","priority":"medium"}]}
\`\`\`

【工具说明】
(1) list_dir: 列出目录内容（支持 paths 多目录；支持 start/end 对条目分页；end=-1 表示 all）
(2) read_file: 读取文件内容（支持 paths 多文件；start/end 按“起始行号-结束行号”裁剪；end=-1 表示 all）
(3) edit_file: 写入文件内容（为了安全，必须带 confirm:true 才会执行）
(4) search_field: 按字段搜索项目内所有文本文件，返回每个文件命中的行号(不是次数!)（会跳过压缩包/图片等二进制；支持 file_type 文件筛选；支持大小写不敏感与正则；可返回命中行片段）
(5) update_todos: 更新侧边栏的 Todo 任务列表，用于在 Plan或Code 模式下记录和追踪执行计划。支持增删改查全量更新。请在完成后立刻更新Todo状态

【任务终止协议】
如果你已经完成了用户的所有请求，且不需要再进行任何操作（如工具调用、进一步解释等），请在回复的最后一行输出：<BREAK>。
如果你没有输出 <BREAK>，系统将认为你还需要继续执行，并会自动向你发送“请继续”的信号，直到你输出 <BREAK> 或达到最大轮数限制。

【输出约束】
- 禁止伪造工具输出：你绝对不能在回复中包含 "TOOL_RESULT" 字符串，也绝对不能编造工具的返回结果。工具结果只能由系统在执行你的工具块后自动注入。
- 不要编造不存在的文件/函数/字段；不确定就先用 list_dir/read_file。
- 避免一次性修改大量文件；必要时先给计划。
- 工具块尽量放在回复末尾，便于解析。`

  const aiRule = ref('')
  const aiAgentMode = ref<'plan' | 'code' | 'ask'>('plan')
  const agentModeMenuOpen = ref(false)
  const agentModeMenuOpenSettings = ref(false)

  const chatSessions = ref<ChatSession[]>([])
  const currentSessionId = ref('')

  const MAX_PROJECT_TREE_DEPTH = 4
  const MAX_PROJECT_TREE_ENTRIES = 600

  const route = useRoute()
  const projectRootPath = computed(() => {
    const p = route.query.path
    return typeof p === 'string' ? p : ''
  })

  const messagesEl = ref<HTMLElement | null>(null)

  const canSend = computed(() => {
    return !isSending.value && input.value.trim().length > 0
  })

  const groupedMessages = computed(() => {
    const groups: Array<{ user: ChatMessage; items: ChatMessage[] }> = []
    let current: { user: ChatMessage; items: ChatMessage[] } | null = null

    for (const m of messages.value) {
      if (m.role === 'user') {
        current = { user: m, items: [] }
        groups.push(current)
        continue
      }
      if (!current) {
        continue
      }
      current.items.push(m)
    }

    return groups
  })

  function createId() {
    return `${Date.now()}_${Math.random().toString(16).slice(2)}`
  }

  function makeSessionTitle(msgs: ChatMessage[]) {
    const firstUser = msgs.find(m => m.role === 'user')
    const raw = (firstUser?.content || '').trim()
    if (!raw) return '新对话'
    return raw.length > 24 ? `${raw.slice(0, 24)}...` : raw
  }

  function createNewSession() {
    const now = Date.now()
    return {
      id: createId(),
      title: '新对话',
      createdAt: now,
      updatedAt: now,
      messages: []
    } as ChatSession
  }

  function getCurrentSessionIndex() {
    return chatSessions.value.findIndex(s => s.id === currentSessionId.value)
  }

  function syncMessagesToCurrentSession() {
    const idx = getCurrentSessionIndex()
    if (idx === -1) return
    const now = Date.now()
    const title = makeSessionTitle(messages.value)
    chatSessions.value[idx] = {
      ...chatSessions.value[idx],
      title,
      updatedAt: now,
      messages: messages.value,
      todos: todos.value
    }
  }

  function getCurrentSession() {
    const idx = getCurrentSessionIndex()
    if (idx === -1) return null
    return chatSessions.value[idx]
  }

  function setCurrentSession(patch: Partial<ChatSession>) {
    const idx = getCurrentSessionIndex()
    if (idx === -1) return
    chatSessions.value[idx] = {
      ...chatSessions.value[idx],
      ...patch
    }
  }

  function loadSessionToMessages(sessionId: string) {
    const s = chatSessions.value.find(x => x.id === sessionId)
    if (!s) return
    currentSessionId.value = s.id
    messages.value = (s.messages || []).map(m => ({
      ...m,
      showReasoning: m.showReasoning || false
    }))
    updateTodos(s.todos || [])
  }

  function appendMessage(role: ChatRole, content: string, pending?: boolean) {
    messages.value.push({
      id: createId(),
      role,
      content,
      createdAt: Date.now(),
      pending,
      reasoning: '',
      showReasoning: false
    })

    syncMessagesToCurrentSession()
  }

  function removeMessage(id: string) {
    const idx = messages.value.findIndex(m => m.id === id)
    if (idx === -1) return
    messages.value.splice(idx, 1)
    syncMessagesToCurrentSession()
  }

  function setMessageContent(id: string, content: string) {
    const idx = messages.value.findIndex(m => m.id === id)
    if (idx === -1) return
    messages.value[idx] = {
      ...messages.value[idx],
      content,
      pending: false
    }

    syncMessagesToCurrentSession()
  }

  function appendToMessage(id: string, delta: { content?: string; reasoning?: string }) {
    const idx = messages.value.findIndex(m => m.id === id)
    if (idx === -1) return

    const prev = messages.value[idx]
    messages.value[idx] = {
      ...prev,
      content: `${prev.content || ''}${delta.content || ''}`,
      reasoning: `${prev.reasoning || ''}${delta.reasoning || ''}`
    }

    syncMessagesToCurrentSession()
  }

  function markMessageDone(id: string) {
    const idx = messages.value.findIndex(m => m.id === id)
    if (idx === -1) return
    const now = Date.now()
    const startedAt = messages.value[idx].startedAt
    messages.value[idx] = {
      ...messages.value[idx],
      pending: false,
      finishedAt: now,
      reasoningMs: typeof startedAt === 'number' ? Math.max(0, now - startedAt) : undefined
    }

    syncMessagesToCurrentSession()
  }

  function checkBreakToken(text: string) {
    const trimmed = (text || '').trimEnd()
    const has = trimmed.endsWith('<BREAK>')
    if (has) {
      const cleaned = trimmed.replace(/<BREAK>\s*$/i, '').trimEnd()
      return { text: cleaned, break: true }
    }
    return { text, break: false }
  }

  function removeBreakTokens(text: string) {
    return (text || '').replace(/<BREAK>/gi, '')
  }

  function extractThinkFromText(text: string) {
    let reasoning = ''
    let cleaned = text || ''

    cleaned = cleaned.replace(/<think>([\s\S]*?)<\/think>/gi, (_full, inner) => {
      const chunk = String(inner || '').trim()
      if (chunk) {
        reasoning += (reasoning ? '\n' : '') + chunk
      }
      return ''
    })

    // 对残留的单独标签做清理（避免 UI 泄漏）
    cleaned = cleaned.replace(/<\/think>/gi, '').replace(/<think>/gi, '')

    return { cleaned: cleaned, reasoning }
  }

  function extractToolCalls(text: string) {
    const toolRegex = /```tool\s*([\s\S]*?)```/gi
    const calls: ToolCall[] = []
    let cleaned = text

    cleaned = cleaned.replace(toolRegex, (_full, raw) => {
      try {
        const obj = JSON.parse(String(raw).trim()) as any
        if (obj && typeof obj.tool === 'string') {
          // offset/limit 已废弃：出现即不要执行（应当走 invalidToolCalls + 提示新格式）
          if (obj.tool === 'read_file' && ('offset' in obj || 'limit' in obj)) {
            return ''
          }
          calls.push(obj as ToolCall)
        }
      } catch {
        // ignore
      }
      return ''
    })

    // 兼容模型输出：
    // tool
    // {"tool":"read_file", ...}
    // （没有 ```tool 围栏）
    const looseRegex = /(^|\n)tool\s*\n\s*(\{[\s\S]*?\})/gi
    cleaned = cleaned.replace(looseRegex, (_full, _p1, rawJson) => {
      try {
        const obj = JSON.parse(String(rawJson).trim()) as any
        if (obj && typeof obj.tool === 'string') {
          if (obj.tool === 'read_file' && ('offset' in obj || 'limit' in obj)) {
            return ''
          }
          calls.push(obj as ToolCall)
        }
      } catch {
        // ignore
      }
      return ''
    })

    return { calls, cleaned: cleaned.trim() }
  }

  function extractInvalidToolCalls(text: string): Array<{ tool: string; raw: string }> {
    const toolRegex = /```tool\s*([\s\S]*?)```/gi
    const invalid: Array<{ tool: string; raw: string }> = []

    let m: RegExpExecArray | null
    while ((m = toolRegex.exec(text)) !== null) {
      const raw = String(m[1] || '').trim()
      try {
        const obj = JSON.parse(raw) as any
        if (obj && typeof obj.tool === 'string') {
          // 对“能解析但字段不完整”的情况做校验：仍然算无效，触发前端红灯与继续纠错
          const t = String(obj.tool)
          const ok = (() => {
            if (t === 'list_dir') {
              const paths = normalizePathsFromCall(obj)
              return paths.length > 0
            }
            if (t === 'read_file') {
              // offset/limit 已废弃：出现即判定为格式错误
              if ('offset' in obj || 'limit' in obj) return false
              const paths = normalizePathsFromCall(obj)
              if (paths.length === 0) return false
              const r = normalizeReadRangeFromCall(obj)
              if (typeof r.start === 'number' || typeof r.end === 'number') {
                if (typeof r.start === 'number' && r.start < 1) return false
                if (typeof r.end === 'number' && r.end !== -1 && r.end < 1) return false
              }
              return true
            }
            if (t === 'edit_file') {
              return (
                typeof obj.path === 'string' &&
                obj.path.trim().length > 0 &&
                typeof obj.content === 'string' &&
                typeof obj.confirm === 'boolean'
              )
            }
            if (t === 'search_field') {
              return (
                typeof obj.query === 'string' &&
                obj.query.trim().length > 0 &&
                (typeof obj.file_type === 'string' || typeof obj.file_type === 'undefined') &&
                (typeof obj.case_insensitive === 'boolean' || typeof obj.case_insensitive === 'undefined') &&
                (typeof obj.regex === 'boolean' || typeof obj.regex === 'undefined')
              )
            }
            if (t === 'update_todos') {
              return Array.isArray(obj.todos)
            }
            return false
          })()

          if (ok) {
            // 合法的会在 extractToolCalls 里处理，这里忽略
            continue
          }
          invalid.push({ tool: t, raw })
          continue
        }
      } catch {
        // ignore
      }

      // 兼容 raw 是 JSON 字符串（包含 \"tool\"）的情况
      const toolMatch = raw.match(/"tool"\s*:\s*"([^"]+)"/) || raw.match(/\\"tool\\"\s*:\s*\\"([^\\"]+)\\"/)
      const tool = (toolMatch?.[1] || '').trim() || 'unknown'
      invalid.push({ tool, raw })
    }

    // 同样兼容无围栏的 tool\n{...}，仅当解析失败时记为 invalid
    const looseRegex = /(^|\n)tool\s*\n\s*(\{[\s\S]*?\})/gi
    let lm: RegExpExecArray | null
    while ((lm = looseRegex.exec(text)) !== null) {
      const raw = String(lm[2] || '').trim()
      try {
        const obj = JSON.parse(raw) as any
        // 若能解析且字段完整，将由 extractToolCalls 执行；这里不记 invalid
        if (obj && typeof obj.tool === 'string') {
          const t = String(obj.tool)
          const ok = (() => {
            if (t === 'list_dir') {
              const paths = normalizePathsFromCall(obj)
              return paths.length > 0
            }
            if (t === 'read_file') {
              if ('offset' in obj || 'limit' in obj) return false
              const paths = normalizePathsFromCall(obj)
              return paths.length > 0
            }
            if (t === 'edit_file') {
              return (
                typeof obj.path === 'string' &&
                obj.path.trim().length > 0 &&
                typeof obj.content === 'string' &&
                typeof obj.confirm === 'boolean'
              )
            }
            if (t === 'search_field') {
              return typeof obj.query === 'string' && obj.query.trim().length > 0
            }
            return false
          })()
          if (ok) {
            continue
          }
          invalid.push({ tool: t, raw })
          continue
        }
      } catch {
        // fallthrough
      }

      const toolMatch = raw.match(/"tool"\s*:\s*"([^"]+)"/) || raw.match(/\\"tool\\"\s*:\s*\\"([^\\"]+)\\"/)
      const tool = (toolMatch?.[1] || '').trim() || 'unknown'
      invalid.push({ tool, raw })
    }

    return invalid
  }

  function toolFormatHint(tool: string) {
    if (tool === 'read_file') {
      return '```tool\n{"tool":"read_file","paths":["相对或绝对路径1","相对或绝对路径2"],"start":1,"end":-1}\n```'
    }
    if (tool === 'list_dir') {
      return '```tool\n{"tool":"list_dir","paths":["相对或绝对路径1","相对或绝对路径2"],"start":1,"end":-1}\n```'
    }
    if (tool === 'edit_file') {
      return '```tool\n{"tool":"edit_file","path":"相对或绝对路径","content":"新的完整文件内容","confirm":true}\n```'
    }
    if (tool === 'search_field') {
      return '```tool\n{"tool":"search_field","query":"要搜索的字段","file_type":"*.yml,*.txt","case_insensitive":true,"regex":false,"with_snippet":true,"snippet_max_len":160}\n```'
    }
    if (tool === 'update_todos') {
      return '```tool\n{"tool":"update_todos","todos":[{"id":"任务ID","content":"任务内容","status":"pending","priority":"medium"}]}\n```'
    }
    return '```tool\n{"tool":"read_file","paths":["相对或绝对路径"],"start":1,"end":-1}\n```'
  }

  function basename(p: string) {
    const norm = (p || '').replace(/\\/g, '/').replace(/\/+$/g, '')
    if (!norm) return ''
    const parts = norm.split('/')
    return parts[parts.length - 1] || ''
  }

  function looksLikeBinaryByExt(p: string) {
    const name = (p || '').toLowerCase()
    const exts = [
      '.zip',
      '.7z',
      '.rar',
      '.gz',
      '.tar',
      '.bz2',
      '.xz',
      '.png',
      '.jpg',
      '.jpeg',
      '.gif',
      '.webp',
      '.bmp',
      '.ico',
      '.tiff',
      '.psd',
      '.dds',
      '.mp3',
      '.wav',
      '.ogg',
      '.mp4',
      '.mkv',
      '.avi',
      '.mov',
      '.pdf',
      '.exe',
      '.dll',
      '.pdb',
      '.bin',
      '.dat',
      '.pak',
      '.rpf'
    ]
    return exts.some(ext => name.endsWith(ext))
  }

  function sliceByLineRange(content: string, start?: number, end?: number) {
    if (!content) return ''
    const lines = content.split(/\r?\n/)
    const s = Math.max(1, typeof start === 'number' ? start : 1)
    if (end === -1) {
      return lines.slice(s - 1).join('\n')
    }
    if (typeof end !== 'number') {
      return lines.slice(s - 1).join('\n')
    }
    const e = Math.max(s, end)
    return lines.slice(s - 1, Math.min(lines.length, e)).join('\n')
  }

  function normalizePathsFromCall(call: any): string[] {
    const ps = Array.isArray(call?.paths) ? call.paths.filter((x: any) => typeof x === 'string' && x.trim()) : []
    if (ps.length > 0) return ps.map((x: string) => x.trim())
    if (typeof call?.path === 'string' && call.path.trim()) return [call.path.trim()]
    return []
  }

  function normalizeReadRangeFromCall(call: any): { start?: number; end?: number } {
    const start = typeof call?.start === 'number' ? call.start : undefined
    const end = typeof call?.end === 'number' ? call.end : undefined
    if (typeof start === 'number' || typeof end === 'number') {
      return { start: typeof start === 'number' ? start : 1, end: typeof end === 'number' ? end : -1 }
    }
    // 默认读取全部
    return { start: 1, end: -1 }
  }

  function isAbsolutePath(p: string) {
    if (!p) return false
    if (/^[a-zA-Z]:[\\/]/.test(p)) return true
    if (p.startsWith('\\\\')) return true
    if (p.startsWith('/')) return true
    return false
  }

  function resolveAgainst(base: string, inputPath: string) {
    const raw = (inputPath || '').trim()
    if (!raw) return ''
    if (!base) return isAbsolutePath(raw) ? raw : raw

    const root = (base || '').trim()
    const preferBackslash = root.includes('\\')
    const sep = preferBackslash ? '\\' : '/'
    const baseNorm = root.replace(/[\\/]+$/g, '')
    const baseLower = baseNorm.toLowerCase()

    // 绝对路径：必须在项目根目录内
    if (isAbsolutePath(raw)) {
      const rawLower = raw.toLowerCase().replace(/[\\/]+$/g, '')
      if (!rawLower.startsWith(baseLower) && rawLower !== baseLower) {
        return ''
      }
      return raw
    }

    let rel = raw.replace(/^\.[\\/]/, '')
    rel = rel.replace(/^[\\/]+/g, '')
    const combined = `${baseNorm}${sep}${rel}`

    const parts = combined.split(/[\\/]+/g)
    const out: string[] = []
    for (const part of parts) {
      if (!part || part === '.') continue
      if (part === '..') {
        if (out.length > 0 && out[out.length - 1] !== '..' && !/^[a-zA-Z]:$/.test(out[out.length - 1])) {
          out.pop()
        } else {
          out.push('..')
        }
        continue
      }
      out.push(part)
    }
    const normalized = out.join(sep)

    // 防止路径穿越：结果必须在 base 目录内
    const normalizedLower = normalized.toLowerCase().replace(/[\\/]+$/g, '')
    if (!normalizedLower.startsWith(baseLower) && normalizedLower !== baseLower) {
      return ''
    }

    if (preferBackslash && baseNorm.startsWith('\\\\') && !normalized.startsWith('\\\\')) {
      return `\\\\${normalized.replace(/^\\+/, '')}`
    }
    return normalized
  }

  function normalizePathForRel(p: string) {
    return (p || '').replace(/\\/g, '/').replace(/\/+$/g, '')
  }

  function toProjectRelativePath(absPath: string) {
    const root = projectRootPath.value
    if (!root) return absPath

    const absNorm = normalizePathForRel(absPath)
    const rootNorm = normalizePathForRel(root)
    const absKey = absNorm.toLowerCase()
    const rootKey = rootNorm.toLowerCase()
    if (absKey === rootKey) return '.'
    if (!absKey.startsWith(`${rootKey}/`)) return absPath
    const rel = absNorm.slice(rootNorm.length).replace(/^\//, '')
    return rel || '.'
  }

  function parseFileTypePatterns(fileType: string | undefined): string[] {
    const raw = (fileType || '').trim()
    if (!raw) return []
    return raw
      .split(/[;,\n\r]+/g)
      .map(x => x.trim())
      .filter(Boolean)
  }

  function globToRegExp(glob: string) {
    const g = (glob || '').trim()
    if (!g) return null
    const escaped = g.replace(/[.+^${}()|[\]\\]/g, '\\$&')
    const re = `^${escaped.replace(/\*/g, '.*').replace(/\?/g, '.')}$$`
    try {
      return new RegExp(re, 'i')
    } catch {
      return null
    }
  }

  function matchFileType(pathOrName: string, patterns: string[]) {
    if (patterns.length === 0) return true
    const candidate = pathOrName.replace(/\\/g, '/')
    for (const p of patterns) {
      const re = globToRegExp(p)
      if (!re) continue
      if (re.test(candidate)) return true
    }
    return false
  }

  function shouldIgnoreDir(name: string) {
    const n = (name || '').toLowerCase()
    return (
      n === '.git' ||
      n === 'node_modules' ||
      n === 'dist' ||
      n === 'build' ||
      n === 'target' ||
      n === '.tauri' ||
      n === '.idea' ||
      n === '.vscode'
    )
  }

  async function buildProjectTree(rootPath: string) {
    const root = resolveAgainst(projectRootPath.value, rootPath)
    if (!root) return ''

    let count = 0
    const lines: string[] = []

    async function walk(dir: string, depth: number, prefix: string) {
      if (depth > MAX_PROJECT_TREE_DEPTH) return
      if (count >= MAX_PROJECT_TREE_ENTRIES) return

      const res = await readDirectory(dir)
      const entries = ((res as any)?.entries as any[]) || ((res as any)?.files as any[]) || []

      const sorted = entries
        .slice()
        .sort((a, b) => {
          const ad = !!a.is_directory
          const bd = !!b.is_directory
          if (ad !== bd) return ad ? -1 : 1
          return String(a.name || '').localeCompare(String(b.name || ''), undefined, { sensitivity: 'base' })
        })

      for (let i = 0; i < sorted.length; i++) {
        if (count >= MAX_PROJECT_TREE_ENTRIES) return
        const e = sorted[i]
        const name = String(e.name || '')
        const isDir = !!e.is_directory
        if (isDir && shouldIgnoreDir(name)) {
          continue
        }

        const isLast = i === sorted.length - 1
        const branch = isLast ? '└─ ' : '├─ '
        lines.push(`${prefix}${branch}${name}${isDir ? '/' : ''}`)
        count += 1

        if (isDir && e.path && depth < MAX_PROJECT_TREE_DEPTH) {
          const nextPrefix = `${prefix}${isLast ? '   ' : '│  '}`
          await walk(String(e.path), depth + 1, nextPrefix)
        }
      }
    }

    lines.push(root)
    await walk(root, 1, '')
    if (count >= MAX_PROJECT_TREE_ENTRIES) {
      lines.push('...（目录树已截断）')
    }
    return lines.join('\n')
  }

  async function ensureProjectTreeForCurrentSession() {
    const s = getCurrentSession()
    if (!s) return
    if (!projectRootPath.value) {
      setCurrentSession({ projectTreeInjected: true, projectTree: '' })
      return
    }

    const tree = await buildProjectTree(projectRootPath.value)
    setCurrentSession({ projectTreeInjected: true, projectTree: tree })
    await saveAiChatSessions()
  }

  async function executeToolCall(call: ToolCall) {
    if (call.tool === 'list_dir') {
      const paths = normalizePathsFromCall(call)
      if (paths.length === 0) {
        return { success: false, message: 'list_dir 缺少 path/paths' }
      }

      const range = normalizeReadRangeFromCall(call)

      function sliceEntries(entries: any[]) {
        const list = Array.isArray(entries) ? entries : []
        const s = typeof range.start === 'number' ? Math.max(1, range.start) : 1
        const e = typeof range.end === 'number' ? range.end : -1
        if (e === -1) {
          return list.slice(s - 1)
        }
        return list.slice(s - 1, Math.min(list.length, Math.max(s, e)))
      }

      if (paths.length === 1) {
        const resolvedPath = resolveAgainst(projectRootPath.value, paths[0])
        const res = await readDirectory(resolvedPath)
        const entries = ((res as any)?.entries as any[]) || ((res as any)?.files as any[]) || []
        const sliced = sliceEntries(entries)
        return { ...res, resolved_path: resolvedPath, entries: sliced, total: entries.length, start: range.start, end: range.end }
      }

      const results: any[] = []
      for (const p of paths) {
        const resolvedPath = resolveAgainst(projectRootPath.value, p)
        const res = await readDirectory(resolvedPath)
        const entries = ((res as any)?.entries as any[]) || ((res as any)?.files as any[]) || []
        const sliced = sliceEntries(entries)
        results.push({ ...res, resolved_path: resolvedPath, entries: sliced, total: entries.length, start: range.start, end: range.end })
      }
      const success = results.every(r => r && r.success !== false)
      return { success, message: success ? 'OK' : '部分失败', results }
    }
    if (call.tool === 'read_file') {
      const paths = normalizePathsFromCall(call)
      if (paths.length === 0) {
        return { success: false, message: 'read_file 缺少 path/paths' }
      }

      const range = normalizeReadRangeFromCall(call)

      async function readOne(p: string) {
        const resolvedPath = resolveAgainst(projectRootPath.value, p)
        if (!resolvedPath) return { success: false, message: `路径越界，拒绝读取: ${p}` }
        const res = await readFileContent(resolvedPath)
        if (res?.success && typeof (res as any).content === 'string') {
          const raw = (res as any).content as string
          const sliced =
            typeof range.start === 'number' || typeof range.end === 'number'
              ? sliceByLineRange(raw, range.start, range.end)
              : raw
          return { ...res, resolved_path: resolvedPath, content: sliced }
        }
        return { ...res, resolved_path: resolvedPath }
      }

      if (paths.length === 1) {
        return await readOne(paths[0])
      }

      const results: any[] = []
      for (const p of paths) {
        results.push(await readOne(p))
      }
      const success = results.every(r => r && r.success !== false)
      return { success, message: success ? 'OK' : '部分失败', results }
    }
    if (call.tool === 'edit_file') {
      if (aiAgentMode.value !== 'code') {
        return {
          success: false,
          message: `拒绝执行 edit_file：当前模式为 ${aiAgentMode.value}，仅 Code 模式允许写入文件`
        }
      }
      if (!(call as any).confirm) {
        return {
          success: false,
          message: '拒绝执行 edit_file：缺少 confirm:true'
        }
      }
      const resolvedPath = resolveAgainst(projectRootPath.value, call.path)
      if (!resolvedPath) return { success: false, message: '路径越界，拒绝写入' }
      const res = await writeFileContent(resolvedPath, (call as any).content)
      return { ...res, resolved_path: resolvedPath }
    }

    if (call.tool === 'search_field') {
      const query = typeof (call as any).query === 'string' ? (call as any).query : ''
      const q = query.trim()
      if (!q) {
        return { success: false, message: 'search_field 缺少 query' }
      }
      if (!projectRootPath.value) {
        return { success: false, message: '项目根目录为空，无法搜索' }
      }

      const caseInsensitive = !!(call as any).case_insensitive
      const useRegex = !!(call as any).regex
      const withSnippet = !!(call as any).with_snippet
      const snippetMaxLenRaw = typeof (call as any).snippet_max_len === 'number' ? (call as any).snippet_max_len : 160
      const snippetMaxLen = Math.max(20, Math.min(800, snippetMaxLenRaw))
      const fileType = typeof (call as any).file_type === 'string' ? (call as any).file_type : ''
      const patterns = parseFileTypePatterns(fileType)

      const matcher = (() => {
        if (useRegex) {
          try {
            const flags = caseInsensitive ? 'i' : ''
            const re = new RegExp(q, flags)
            return (line: string) => re.test(line)
          } catch {
            return null
          }
        }
        if (caseInsensitive) {
          const needle = q.toLowerCase()
          return (line: string) => line.toLowerCase().includes(needle)
        }
        return (line: string) => line.includes(q)
      })()

      if (!matcher) {
        return { success: false, message: 'search_field 正则表达式无效' }
      }

      const matchLine = matcher

      const MAX_FILES_SCANNED = 5000
      const MAX_MATCH_FILES = 500
      const MAX_MATCH_LINES_PER_FILE = 200

      const matched: Array<{ file: string; rel: string; lines: number[]; snippets?: Array<{ line: number; snippet: string }> }> = []
      let scannedFiles = 0
      let skippedFiles = 0
      let scannedDirs = 0

      async function scanFile(filePath: string) {
        if (matched.length >= MAX_MATCH_FILES) return
        if (looksLikeBinaryByExt(filePath)) {
          skippedFiles += 1
          return
        }

        const rel = toProjectRelativePath(filePath)
        if (patterns.length > 0) {
          const name = basename(filePath) || filePath
          if (!matchFileType(rel, patterns) && !matchFileType(name, patterns)) {
            skippedFiles += 1
            return
          }
        }

        const res = await readFileContent(filePath)
        if (!res?.success) {
          skippedFiles += 1
          return
        }
        if ((res as any).is_binary || (res as any).is_image) {
          skippedFiles += 1
          return
        }
        const content = typeof (res as any).content === 'string' ? ((res as any).content as string) : ''
        if (!content) return

        const lines = content.split(/\r?\n/)
        const hitLines: number[] = []
        const hitSnippets: Array<{ line: number; snippet: string }> = []
        for (let i = 0; i < lines.length; i++) {
          if (hitLines.length >= MAX_MATCH_LINES_PER_FILE) break
          const lineText = lines[i] || ''
          if (matchLine(lineText)) {
            const ln = i + 1
            hitLines.push(ln)
            if (withSnippet) {
              const snippet = lineText.length > snippetMaxLen ? `${lineText.slice(0, snippetMaxLen)}...` : lineText
              hitSnippets.push({ line: ln, snippet })
            }
          }
        }
        if (hitLines.length > 0) {
          matched.push({ file: filePath, rel, lines: hitLines, snippets: withSnippet ? hitSnippets : undefined })
        }
      }

      async function walk(dir: string) {
        if (scannedFiles >= MAX_FILES_SCANNED) return
        if (matched.length >= MAX_MATCH_FILES) return
        scannedDirs += 1

        const res = await readDirectory(dir)
        const entries = ((res as any)?.entries as any[]) || ((res as any)?.files as any[]) || []
        for (const e of entries) {
          if (scannedFiles >= MAX_FILES_SCANNED) return
          if (matched.length >= MAX_MATCH_FILES) return
          const name = String(e?.name || '')
          const isDir = !!e?.is_directory
          const p = String(e?.path || '')
          if (!p) continue
          if (isDir) {
            if (shouldIgnoreDir(name)) continue
            await walk(p)
            continue
          }
          scannedFiles += 1
          await scanFile(p)
        }
      }

      await walk(projectRootPath.value)

      const results = matched.map(x => {
        const rel = x.rel || toProjectRelativePath(x.file)
        return `${rel}(${x.lines.join(',')})`
      })

      return {
        success: true,
        message: 'OK',
        query: q,
        file_type: fileType || undefined,
        case_insensitive: caseInsensitive,
        regex: useRegex,
        with_snippet: withSnippet,
        snippet_max_len: snippetMaxLen,
        results,
        matches: withSnippet
          ? matched.map(x => ({ path: x.rel || toProjectRelativePath(x.file), hits: (x.snippets || []).slice(0, MAX_MATCH_LINES_PER_FILE) }))
          : undefined,
        matched_files: matched.length,
        scanned_files: scannedFiles,
        scanned_dirs: scannedDirs,
        skipped_files: skippedFiles,
        truncated: scannedFiles >= MAX_FILES_SCANNED || matched.length >= MAX_MATCH_FILES
      }
    }

    if (call && (call as any).tool === 'update_todos') {
      const ts = (call as any).todos
      if (!Array.isArray(ts)) {
        return { success: false, message: 'update_todos 缺少 todos 数组' }
      }
      updateTodos(ts)
      return { success: true, message: 'Todo 列表已同步', hide_result: true }
    }

    return { success: false, message: '未知工具' }
  }

  function appendToolResultMessage(call: ToolCall, result: any) {
    if (result && typeof result === 'object' && result.hide_result === true) {
      return
    }

    const safeResult = (() => {
      try {
        return JSON.stringify(result, null, 2)
      } catch {
        return String(result)
      }
    })()

    appendMessage(
      'system',
      `TOOL_RESULT\ntool: ${call.tool}\ninput: ${JSON.stringify(call)}\noutput:\n${safeResult}`
    )
  }

  function appendInvalidToolFormatMessage(tool: string, raw: string) {
    const safeRaw = (raw || '').slice(0, 800)
    const payload = {
      success: false,
      message: '输出格式错误',
      raw: safeRaw
    }

    appendMessage(
      'system',
      `TOOL_RESULT\ntool: ${tool || 'unknown'}\ninput: ${JSON.stringify({ tool: tool || 'unknown' })}\noutput:\n${JSON.stringify(payload, null, 2)}`
    )
  }

  function isToolResultMessage(m: ChatMessage) {
    return (m.role === 'system' || m.role === 'assistant') && typeof m.content === 'string' && m.content.startsWith('TOOL_RESULT')
  }

  function toolResultInfo(m: ChatMessage): { text: string; status: 'success' | 'fail' | 'unknown' } {
    const lines = (m.content || '').split(/\r?\n/)
    const toolLine = lines.find(l => l.startsWith('tool:')) || ''
    const tool = toolLine.replace(/^tool:\s*/i, '').trim() || 'tool'

    const inputLine = lines.find(l => l.startsWith('input:')) || ''
    const rawInput = inputLine.replace(/^input:\s*/i, '').trim()

    const outputLine = lines.find(l => l.startsWith('output:'))
    const rawOutput = outputLine ? lines.slice(lines.indexOf(outputLine) + 1).join('\n') : ''

    let input: any = null
    try {
      input = rawInput ? JSON.parse(rawInput) : null
    } catch {
      input = null
    }

    let output: any = null
    let outputParseOk = false
    try {
      output = rawOutput ? JSON.parse(rawOutput) : null
      outputParseOk = true
    } catch {
      output = null
      outputParseOk = false
    }

    const success = typeof output?.success === 'boolean' ? output.success : undefined
    const status: 'success' | 'fail' | 'unknown' = success === true ? 'success' : success === false ? 'fail' : 'unknown'

    function normalizePath(p: string) {
      return (p || '').replace(/\\/g, '/').replace(/\/+$/g, '')
    }

    function toRelativePath(absPath: string) {
      const root = projectRootPath.value
      if (!root) return absPath

      const absNorm = normalizePath(absPath)
      const rootNorm = normalizePath(root)

      // Windows 盘符路径做大小写不敏感处理
      const absKey = absNorm.toLowerCase()
      const rootKey = rootNorm.toLowerCase()
      if (absKey === rootKey) return '.'
      if (!absKey.startsWith(`${rootKey}/`)) return absPath

      const rel = absNorm.slice(rootNorm.length).replace(/^\//, '')
      return rel || '.'
    }

    const resolvedPath = typeof output?.resolved_path === 'string' ? output.resolved_path : ''
    const rawPath = typeof input?.path === 'string' ? input.path : ''
    const displayPath = resolvedPath ? toRelativePath(resolvedPath) : rawPath ? toRelativePath(rawPath) : ''

    const message = typeof output?.message === 'string' ? output.message : ''
    if (message === '输出格式错误') {
      return { text: `${tool} 输出格式错误`, status: 'fail' }
    }

    if (tool === 'tool_calling') {
      return { text: 'tool_calling', status: 'unknown' }
    }

    const outputIncompleteHint = !outputParseOk && rawOutput.trim().startsWith('{') ? '（输出未完整）' : ''

    const inputPaths = normalizePathsFromCall(input)
    const range = normalizeReadRangeFromCall(input)

    function rangeText() {
      const s = typeof range.start === 'number' ? range.start : undefined
      const e = typeof range.end === 'number' ? range.end : undefined
      if (e === -1) return `(all)`
      const startNum = typeof s === 'number' ? s : 1
      const endNum = typeof e === 'number' ? e : -1
      if (endNum === -1) return `(all)`
      return `(${startNum}-${endNum})`
    }

    const actionText = (() => {
      if (tool === 'read_file') {
        const toolName = 'read_file'
        if (inputPaths.length <= 1) {
          const pAbs = inputPaths[0] ? resolveAgainst(projectRootPath.value, inputPaths[0]) : displayPath
          const p = basename(pAbs) || basename(displayPath) || displayPath
          return `${toolName} ${p}${rangeText()}${outputIncompleteHint}`.trim()
        }
        const lines = [toolName]
        for (const p0 of inputPaths) {
          const pAbs = resolveAgainst(projectRootPath.value, p0)
          const name = basename(pAbs) || pAbs
          lines.push(`${name}${rangeText()}`.trim())
        }
        if (outputIncompleteHint) lines[0] = `${lines[0]}${outputIncompleteHint}`
        return lines.join('\n')
      }
      if (tool === 'list_dir') {
        const toolName = 'list_dir'
        if (inputPaths.length <= 1) {
          const pAbs = inputPaths[0] ? resolveAgainst(projectRootPath.value, inputPaths[0]) : displayPath
          const p = basename(pAbs) || basename(displayPath) || displayPath
          return `${toolName} ${p}${outputIncompleteHint}`.trim()
        }
        const lines = [toolName]
        for (const p0 of inputPaths) {
          const pAbs = resolveAgainst(projectRootPath.value, p0)
          lines.push(basename(pAbs) || pAbs)
        }
        if (outputIncompleteHint) lines[0] = `${lines[0]}${outputIncompleteHint}`
        return lines.join('\n')
      }
      if (tool === 'edit_file') return `写入文件 ${displayPath}`.trim()
      if (tool === 'search_field') {
        const q = typeof input?.query === 'string' ? input.query : ''
        const fileType = typeof input?.file_type === 'string' ? input.file_type : ''
        const ci = !!input?.case_insensitive
        const re = !!input?.regex
        const matchedCount = typeof output?.matched_files === 'number' ? output.matched_files : undefined
        const truncated = !!output?.truncated
        const flags = `${ci ? ' i' : ''}${re ? ' re' : ''}`.trim()
        const scope = fileType ? ` ${fileType}` : ''
        const suffix = typeof matchedCount === 'number' ? `（命中 ${matchedCount} 个文件${truncated ? '，已截断' : ''}）` : ''
        return `search_field${scope} ${q}${flags ? ` [${flags}]` : ''}${suffix}`.trim()
      }
      return tool
    })()

    return { text: actionText || tool, status }
  }

  function getChatRequestUrl() {
    const base = (openaiBaseUrl.value || '').trim().replace(/\/+$/, '')
    return `${base}/v1/chat/completions`
  }

  function isOfficialOpenAiBaseUrl() {
    const base = (openaiBaseUrl.value || '').trim().replace(/\/+$/, '')
    return base === 'https://api.openai.com'
  }

  const md = new MarkdownIt({
    html: false,
    linkify: true,
    breaks: true
  })

  function assistantHtml(m: ChatMessage) {
    return md.render(m.content || '')
  }

  function getAgentPrompt(mode: 'plan' | 'code' | 'ask') {
    if (mode === 'code') {
      return `你现在处于 Code 模式。你的目标是在保证正确性的前提下，尽量给出可直接应用的代码改动

【核心原则】
1 直接可用：优先给出可粘贴/可替换的代码（或精确到文件/函数的修改点）。
2 最小解释：只解释“为什么必须这样改”和“用户需要注意什么”，避免长篇理论。
3 不做假设：遇到缺少信息时，先给出你需要的最少信息；但如果能通过合理默认值完成，也可先实现并标注默认值。
4 错误处理与边界：必须考虑空值、异常、加载失败、兼容旧配置等边界情况。
5 保持一致：遵循项目现有风格（命名、状态管理、UI 类名、持久化方式）。

【输出结构（严格）】
A 变更摘要
- 说明改了什么、影响面多大

B 修改清单（按文件）
- 文件路径：
  - 改动点（函数/组件/关键段落）
  - 关键代码片段

C 验证方式
- 如何手动验证（步骤）
- 若有，如何观察日志/报错点

D 注意事项（可选）
- 兼容性/迁移/可能影响的行为

【约束】
- 不输出会破坏项目结构的大改（除非用户明确要求）。
- 不编造不存在的字段/组件；不确定就先问或先查。`
    }
    if (mode === 'ask') {
      return `你现在处于 Ask 模式（澄清优先）。你的目标是回复用户的问题、给出代码辅助用户编码

【风险控制】
- 禁止调用 edit_file（Ask 模式不允许写入文件）。
- 不得编造项目结构/函数名/接口字段。
- 若用户描述与现有实现可能冲突，必须指出冲突点并询问确认。
- 若存在多种实现路线，给出推荐路线 + 选择理由 + 风险对比。`
    }
    return `你现在处于 Plan 模式。你的目标是先输出可执行的计划，再按步骤推进实现，并在每一步保持可验证性与最小改动。

【核心原则】
0 禁止写入：禁止调用 edit_file（Plan 模式不允许写入文件）。
1 先计划后实施：先给清晰计划，再给实施细节。
2 记录计划：在输出计划后，必须调用 update_todos 工具将计划同步到侧边栏。
3 小步可验证：每一步都应能通过“观察 UI / 运行 / 日志 / 单测”验证。
4 最小必要改动：优先选择影响面小、风险低的改动；避免大范围重构，除非明确必要。
5 明确依赖与风险：对高风险点或不确定点提前说明，并提供替代方案。
6 结构化输出：让用户能快速扫读并知道你接下来要做什么。

【输出结构（严格）】
A 目标复述
- 复述用户目标与边界，确保对齐

B 执行计划（必须）
- 2-6 个步骤，按顺序编号
- 每步包含：
  - 目标（这一步做什么）
  - 触达点（会改哪些模块/文件/组件/函数，尽量具体）
  - 验证方式（如何确认这一步成功）

C 任务列表同步（必须）
- 调用 update_todos 工具，将上述计划转换为结构化任务。

D 实施细节（按需）
- 列出关键逻辑/关键数据流/关键 UI 状态
- 若涉及配置或持久化，说明字段名与迁移策略

E 输出与回滚
- 最终会得到什么效果
- 若用户不满意，如何回滚或调整（例如开关/配置/可逆改动）

【编码风格约束】
- 避免无意义的抽象；可读性优先。
- 若需要改动 API/数据结构，明确所有调用方需要同步调整的位置。
- 不输出未经验证的“可能存在的文件/函数”。`
  }

  function buildOpenAiMessages(): Array<{ role: ChatRole; content: string }> {
    const rule = aiRule.value.trim()
    const agentPrompt = getAgentPrompt(aiAgentMode.value).trim()

    const session = getCurrentSession()
    const projectTree = (session?.projectTree || '').trim()

    const injected: Array<{ role: ChatRole; content: string }> = [{ role: 'system', content: SYSTEM_PROMPT }]
    if (rule) {
      injected.push({ role: 'system', content: `规则：${rule}` })
    }
    if (agentPrompt) {
      injected.push({ role: 'system', content: agentPrompt })
    }
    if (todos.value.length > 0) {
      injected.push({
        role: 'system',
        content: `当前任务计划状态 (Todos):\n${JSON.stringify(todos.value, null, 2)}`
      })
    }
    if (projectTree) {
      injected.push({
        role: 'system',
        content: `当前项目目录结构：\n\n${projectTree}`
      })
    }

    const base = messages.value
      .filter(m => !m.pending)
      .filter(m => m.role === 'user' || m.role === 'assistant' || m.role === 'system')
      .map(m => ({ role: m.role, content: m.content }))

    return [...injected, ...base]
  }

  async function callOpenAiChatCompletion(userContent: string) {
    const apiKey = openaiApiKey.value.trim()
    if (!apiKey) {
      throw new Error('未配置 OpenAI API Key')
    }

    const url = getChatRequestUrl()
    const payload: Record<string, unknown> = {
      model: openaiModel.value.trim() || 'gpt-4o-mini',
      stream: true,
      messages: [...buildOpenAiMessages(), { role: 'user' as const, content: userContent }]
    }

    if (requestReasoning.value && !isOfficialOpenAiBaseUrl()) {
      payload.include_reasoning = true
    }

    const res = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${apiKey}`
      },
      body: JSON.stringify(payload)
    })

    if (!res.ok) {
      let detail = ''
      try {
        const errJson = await res.json()
        detail = (errJson?.error?.message as string) || JSON.stringify(errJson)
      } catch {
        detail = await res.text()
      }
      throw new Error(`请求失败 (${res.status}): ${detail || res.statusText}`)
    }

    if (!res.body) {
      throw new Error('响应不支持流式读取')
    }

    return res.body
  }

  function normalizeSseLines(buffer: string) {
    return buffer.split(/\r?\n/)
  }

  function tryParseSseDataLine(line: string) {
    const trimmed = line.trim()
    if (!trimmed.startsWith('data:')) return null
    return trimmed.slice('data:'.length).trim()
  }

  function extractDeltaFromChunk(json: any): { content?: string; reasoning?: string } {
    const delta = json?.choices?.[0]?.delta || json?.choices?.[0]?.message || {}
    const content = (delta?.content as string) || ''

    const reasoning =
      (delta?.reasoning as string) ||
      (delta?.reasoning_content as string) ||
      (delta?.reasoningContent as string) ||
      ''

    return {
      content: content || undefined,
      reasoning: reasoning || undefined
    }
  }

  async function consumeSseStream(
    stream: ReadableStream<Uint8Array>,
    onDelta: (delta: { content?: string; reasoning?: string }) => void
  ) {
    const reader = stream.getReader()
    const decoder = new TextDecoder('utf-8')
    let buffer = ''

    while (true) {
      const { value, done } = await reader.read()
      if (done) break

      buffer += decoder.decode(value, { stream: true })
      const lines = normalizeSseLines(buffer)
      buffer = lines.pop() || ''

      for (const line of lines) {
        const data = tryParseSseDataLine(line)
        if (!data) continue
        if (data === '[DONE]') return

        try {
          const json = JSON.parse(data)
          const delta = extractDeltaFromChunk(json)
          if (delta.content || delta.reasoning) {
            onDelta(delta)
          }
        } catch {
          // ignore
        }
      }
    }
  }

  function toggleReasoning(m: ChatMessage) {
    m.showReasoning = !m.showReasoning
  }

  async function handleSend() {
    if (!canSend.value) return

    const content = input.value.trim()
    input.value = ''

    appendMessage('user', content)

    await runAiTurns(content)
  }

  async function runAiTurns(firstUserContent: string) {
    let turns = 0
    let nextUserContent: string | null = firstUserContent

    while (nextUserContent !== null && turns < MAX_CONTINUE_TURNS) {
      if (turns === 0) {
        await ensureProjectTreeForCurrentSession()
      }
      const userContent = nextUserContent
      nextUserContent = null
      turns += 1

      isSending.value = true
      const pendingId = createId()
      let toolCallingBubbleId: string | null = null

      // 流式阶段同时维护“完整文本（用于解析）”与“展示文本（避免 tool/continue 影响 UI）”
      let fullAssistantText = ''
      let rollingWindow = ''
      let suppressVisibleContent = false
      let sawFence = false

      function ensureToolCallingBubble() {
        if (toolCallingBubbleId) return
        toolCallingBubbleId = createId()
        const bubble = {
          id: toolCallingBubbleId,
          role: 'system' as const,
          content: `TOOL_RESULT\ntool: tool_calling\ninput: {}\noutput:\n${JSON.stringify({ message: '正在调用...' }, null, 2)}`,
          createdAt: Date.now(),
          pending: true
        }

        // 插入到当前 assistant 流式消息之后，避免气泡跑到最底部
        const assistantIdx = messages.value.findIndex(x => x.id === pendingId)
        if (assistantIdx >= 0) {
          messages.value.splice(assistantIdx + 1, 0, bubble)
        } else {
          messages.value.push(bubble)
        }
        syncMessagesToCurrentSession()
      }

      function clearToolCallingBubble() {
        if (!toolCallingBubbleId) return
        removeMessage(toolCallingBubbleId)
        toolCallingBubbleId = null
      }

      messages.value.push({
        id: pendingId,
        role: 'assistant',
        content: '',
        createdAt: Date.now(),
        pending: true,
        reasoning: '',
        showReasoning: false,
        startedAt: Date.now()
      })

      try {
        const stream = await callOpenAiChatCompletion(userContent)
        await consumeSseStream(stream, (delta) => {
          // 先累积完整文本（包含 tool/continue），用于最终解析
          if (delta.content) {
            fullAssistantText += delta.content

            // 用滑动窗口识别跨 chunk 的 ```tool
            rollingWindow = `${rollingWindow}${delta.content}`
            if (rollingWindow.length > 24) {
              rollingWindow = rollingWindow.slice(-24)
            }

            if (rollingWindow.includes('```')) {
              sawFence = true
            }

            // 两阶段：先看到 ```，再看到 tool（允许被拆分到不同 chunk）
            const fenceTool = /```\s*tool/i.test(rollingWindow) || /```tool/i.test(fullAssistantText)
            const fenceThenTool = sawFence && /tool/i.test(rollingWindow)
            if (fenceTool || fenceThenTool) {
              suppressVisibleContent = true
              ensureToolCallingBubble()
            }
          }

          // reasoning 直接展示
          if (delta.reasoning) {
            appendToMessage(pendingId, { reasoning: delta.reasoning })
          }

          // content：
          // - 隐藏 <BREAK>
          // - 隐藏 <think>/<\/think>
          // - 隐藏 TOOL_RESULT（防止模型伪造输出泄漏到 UI）
          // - 一旦进入 tool 输出段，就不再向可见内容追加（避免 UI 看到半截 tool 导致体验差）
          if (delta.content && !suppressVisibleContent) {
            const visible = delta.content
              .replace(/<BREAK>/gi, '')
              .replace(/<think>/gi, '')
              .replace(/<\/think>/gi, '')
              .replace(/TOOL_RESULT/gi, '')
            if (visible) {
              appendToMessage(pendingId, { content: visible })
            }
          }
        })

        markMessageDone(pendingId)
        const finalFull = fullAssistantText
        const { text: noBreakText0, break: isFinished } = checkBreakToken(finalFull)
        const { cleaned: noBreakText, reasoning: thinkReasoning } = extractThinkFromText(noBreakText0)
        if (thinkReasoning) {
          appendToMessage(pendingId, { reasoning: thinkReasoning })
        }
        const invalidToolCalls = extractInvalidToolCalls(noBreakText)
        const { calls, cleaned } = extractToolCalls(noBreakText)

        if (invalidToolCalls.length > 0) {
          for (const inv of invalidToolCalls.slice(0, 3)) {
            appendInvalidToolFormatMessage(inv.tool, inv.raw)
          }
        }

        // 用“清洗后的文本”覆盖展示内容（移除 tool 块与 BREAK）
        const currentVisible = messages.value.find(m => m.id === pendingId)?.content || ''
        const { cleaned: cleanedNoThink } = extractThinkFromText(cleaned)
        const displayText = removeBreakTokens(cleanedNoThink).trimEnd()
        if (displayText !== currentVisible) {
          setMessageContent(pendingId, displayText || '（无内容返回）')
        } else if (!displayText.trim()) {
          setMessageContent(pendingId, '（无内容返回）')
        }

        for (const call of calls) {
          try {
            const result = await executeToolCall(call)
            appendToolResultMessage(call, result)
          } catch (err) {
            const msg = err instanceof Error ? err.message : String(err)
            appendToolResultMessage(call, { success: false, message: msg })
          }
        }

        clearToolCallingBubble()

        if (!isFinished) {
          const invalidHint = invalidToolCalls.length > 0
            ? `\n\n你调用的 ${invalidToolCalls.map(x => `"${x.tool}"`).join('、')} 工具格式不对，应该严格使用如下格式之一：\n\n${invalidToolCalls
              .slice(0, 2)
              .map(x => toolFormatHint(x.tool))
              .join('\n\n')}`
            : ''

          const invalidExtra = invalidToolCalls.length > 0
            ? `\n\n额外提醒：不要把 JSON 再包成字符串（例如 "{...}"），tool 块里应该直接是对象 JSON。`
            : ''

          nextUserContent = `你已经进行了以下操作：\n\n${cleaned || '（上次输出为空）'}\n\n现在你需要继续下一步。${invalidHint}${invalidExtra}\n若已完成，请在回复末尾输出 <BREAK>。`
        }
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err)
        logger.error('AI 发送失败:', err)
        setMessageContent(pendingId, `请求失败：${msg}`)
        clearToolCallingBubble()
        nextUserContent = null
      } finally {
        isSending.value = false
      }
    }
  }

  /**
   * 提示词优化
   * 收集当前上下文、智能体模式、项目目录和待办事项，请求 AI 优化用户输入的提示词
   */
  async function optimizePrompt() {
    if (isOptimizing.value || !input.value.trim()) return

    const apiKey = openaiApiKey.value.trim()
    if (!apiKey) {
      logger.error('未配置 OpenAI API Key')
      return
    }

    isOptimizing.value = true
    try {
      await ensureProjectTreeForCurrentSession()
      const session = getCurrentSession()
      const projectTree = (session?.projectTree || '').trim()
      
      const contextMessages = messages.value
        .filter(m => !m.pending)
        .filter(m => m.role === 'user' || m.role === 'assistant')
        .map(m => ({ role: m.role, content: m.content }))

      const optimizationSystemPrompt = `你是一个专业的提示词优化专家。你的任务是将用户输入的原始提示词优化为结构化、模块化且高质量的提示词。

【优化标准】
1. 结构化：使用清晰的标题、列表和分段。
2. 明确性：消除歧义，明确任务目标、约束条件和输出格式。
3. 模块化：将不同类型的信息（背景、任务、要求、示例等）分开。
4. 针对性：结合当前的项目上下文、待办事项和智能体模式进行优化。

【输入数据】
- 原始提示词：${input.value}
- 当前智能体模式：${aiAgentMode.value}
- 项目目录结构：${projectTree || '暂无'}
- 待办事项：${JSON.stringify(todos.value, null, 2)}
- 历史对话上下文：${JSON.stringify(contextMessages)}

【输出要求】
- 只输出优化后的提示词内容，不要包含任何解释、开场白或结束语。
- 优化后的内容应直接可用于 AI 聊天。
- 保持原始意图，但提升表达的专业性和准确性。`

      const url = getChatRequestUrl()
      const payload = {
        model: openaiModel.value.trim() || 'gpt-4o-mini',
        stream: false,
        messages: [
          { role: 'system', content: optimizationSystemPrompt },
          { role: 'user', content: `请优化以下提示词：\n\n${input.value}` }
        ]
      }

      const res = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${apiKey}`
        },
        body: JSON.stringify(payload)
      })

      if (!res.ok) {
        let detail = ''
        try {
          const errJson = await res.json()
          detail = (errJson?.error?.message as string) || JSON.stringify(errJson)
        } catch {
          detail = await res.text()
        }
        throw new Error(`请求失败 (${res.status}): ${detail}`)
      }

      const data = await res.json()
      const optimizedContent = data?.choices?.[0]?.message?.content || ''
      if (optimizedContent) {
        input.value = optimizedContent.trim()
      }
    } catch (err) {
      logger.error('提示词优化失败:', err)
    } finally {
      isOptimizing.value = false
    }
  }

  function handleInputKeydown(e: KeyboardEvent) {
    if (e.key !== 'Enter') return

    if (e.ctrlKey) {
      e.preventDefault()
      const target = e.target as HTMLTextAreaElement | null
      if (!target) {
        input.value += '\n'
        return
      }

      const start = target.selectionStart ?? input.value.length
      const end = target.selectionEnd ?? input.value.length
      input.value = `${input.value.slice(0, start)}\n${input.value.slice(end)}`
      void nextTick(() => {
        try {
          target.selectionStart = target.selectionEnd = start + 1
        } catch {
          // ignore
        }
      })
      return
    }

    e.preventDefault()
    void handleSend()
  }

  async function loadAiSettings() {
    try {
      const result = await loadSettings()
      if (result.success && result.data) {
        const data = result.data as Record<string, unknown>
        openaiApiKey.value = (data.openaiApiKey as string) || ''
        openaiBaseUrl.value = (data.openaiBaseUrl as string) || 'https://api.openai.com'
        openaiModel.value = (data.openaiModel as string) || 'gpt-4o-mini'
        renderMarkdown.value = (data.aiRenderMarkdown as boolean) || false
        requestReasoning.value = (data.aiRequestReasoning as boolean) || false
        aiRule.value = (data.aiRule as string) || (data.aiSystemPrompt as string) || ''
        aiAgentMode.value = ((data.aiAgentMode as 'plan' | 'code' | 'ask') || 'plan')

        const sessions = (data.aiChatSessions as any) || []
        if (Array.isArray(sessions)) {
          chatSessions.value = sessions as ChatSession[]
        }
        const current = (data.aiChatCurrentSessionId as string) || ''
        if (current && chatSessions.value.some(s => s.id === current)) {
          currentSessionId.value = current
        }
      }

      if (!currentSessionId.value || !chatSessions.value.some(s => s.id === currentSessionId.value)) {
        const s = createNewSession()
        chatSessions.value = [s, ...chatSessions.value]
        currentSessionId.value = s.id
      }
      loadSessionToMessages(currentSessionId.value)
    } catch (err) {
      logger.error('加载AI设置失败:', err)
    }
  }

  async function saveAiChatSessions() {
    try {
      const result = await loadSettings()
      const current = (result.success && result.data) ? (result.data as Record<string, unknown>) : {}

      const maxSessions = 30
      const trimmed = chatSessions.value
        .slice()
        .sort((a, b) => (b.updatedAt || 0) - (a.updatedAt || 0))
        .slice(0, maxSessions)

      await saveSettings({
        ...current,
        aiChatSessions: trimmed,
        aiChatCurrentSessionId: currentSessionId.value
      } as any)
    } catch (err) {
      logger.error('保存聊天历史失败:', err)
    }
  }

  async function saveAiSettings() {
    try {
      const result = await loadSettings()
      const current = (result.success && result.data) ? (result.data as Record<string, unknown>) : {}
      await saveSettings({
        ...current,
        openaiApiKey: openaiApiKey.value,
        openaiBaseUrl: openaiBaseUrl.value,
        openaiModel: openaiModel.value,
        aiRenderMarkdown: renderMarkdown.value,
        aiRequestReasoning: requestReasoning.value,
        aiRule: aiRule.value,
        aiAgentMode: aiAgentMode.value
      })
    } catch (err) {
      logger.error('保存AI设置失败:', err)
    }
  }

  function openSettings() {
    settingsOpen.value = true
  }

  function openHistory() {
    historyOpen.value = true
  }

  async function closeHistory() {
    historyOpen.value = false
    await saveAiChatSessions()
  }

  async function startNewChat() {
    syncMessagesToCurrentSession()

    const s = createNewSession()
    chatSessions.value = [s, ...chatSessions.value]
    currentSessionId.value = s.id
    messages.value = []
    updateTodos([])

    await saveAiChatSessions()
  }

  async function selectChatSession(sessionId: string) {
    syncMessagesToCurrentSession()
    loadSessionToMessages(sessionId)
    await saveAiChatSessions()
  }

  async function deleteChatSessions(sessionIds: string[]) {
    const ids = Array.isArray(sessionIds) ? sessionIds.filter(x => typeof x === 'string' && x.trim()) : []
    if (ids.length === 0) return

    const idSet = new Set(ids)
    const deletingCurrent = idSet.has(currentSessionId.value)

    chatSessions.value = (chatSessions.value || []).filter(s => !idSet.has(s.id))

    if (deletingCurrent) {
      // 当前会话被删：切到最近更新时间最新的会话；若无则创建新会话
      const next = (chatSessions.value || [])
        .slice()
        .sort((a, b) => (b.updatedAt || 0) - (a.updatedAt || 0))[0]
      if (next) {
        currentSessionId.value = next.id
        loadSessionToMessages(next.id)
      } else {
        const s = createNewSession()
        chatSessions.value = [s]
        currentSessionId.value = s.id
        messages.value = []
      }
    }

    await saveAiChatSessions()
  }

  async function closeSettings() {
    settingsOpen.value = false
    await saveAiSettings()
  }

  onMounted(() => {
    void loadAiSettings()
  })

  watch(
    () => messages.value.length,
    async () => {
      await nextTick()
      const el = messagesEl.value
      if (!el) return
      el.scrollTop = el.scrollHeight
    }
  )

  return {
    messages,
    input,
    isSending,
    isOptimizing,
    settingsOpen,
    historyOpen,
    openaiApiKey,
    openaiBaseUrl,
    openaiModel,
    renderMarkdown,
    requestReasoning,
    aiRule,
    aiAgentMode,
    agentModeMenuOpen,
    agentModeMenuOpenSettings,
    chatSessions,
    currentSessionId,
    messagesEl,
    canSend,
    groupedMessages,
    assistantHtml,
    openSettings,
    openHistory,
    closeSettings,
    closeHistory,
    startNewChat,
    selectChatSession,
    deleteChatSessions,
    handleSend,
    optimizePrompt,
    handleInputKeydown,
    toggleReasoning,
    isToolResultMessage,
    toolResultInfo,
    saveAiSettings
  }
}
