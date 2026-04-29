export type ChatRole = 'user' | 'assistant' | 'system'

export interface ChatMessage {
  id: string
  role: ChatRole
  content: string
  reasoning?: string
  showReasoning?: boolean
  startedAt?: number
  finishedAt?: number
  reasoningMs?: number
  createdAt: number
  pending?: boolean
}

export interface TodoItem {
  id: string
  content: string
  status: 'pending' | 'in_progress' | 'completed'
  priority: 'low' | 'medium' | 'high'
}

export interface ChatSession {
  id: string
  title: string
  createdAt: number
  updatedAt: number
  messages: ChatMessage[]
  projectTree?: string
  projectTreeInjected?: boolean
  todos?: TodoItem[]
}

export type ToolName = 'list_dir' | 'read_file' | 'edit_file' | 'search_field' | 'update_todos'

export interface ToolCallBase {
  tool: ToolName
}

export interface ToolCallListDir extends ToolCallBase {
  tool: 'list_dir'
  path?: string
  paths?: string[]
  start?: number
  end?: number
}

export interface ToolCallReadFile extends ToolCallBase {
  tool: 'read_file'
  path?: string
  paths?: string[]
  start?: number
  end?: number
}

export interface ToolCallEditFile extends ToolCallBase {
  tool: 'edit_file'
  path: string
  content: string
  confirm?: boolean
}

export interface ToolCallSearchField extends ToolCallBase {
  tool: 'search_field'
  query: string
  file_type?: string
  case_insensitive?: boolean
  regex?: boolean
  with_snippet?: boolean
  snippet_max_len?: number
}

export interface ToolCallUpdateTodos extends ToolCallBase {
  tool: 'update_todos'
  todos: TodoItem[]
}

export type ToolCall =
  | ToolCallListDir
  | ToolCallReadFile
  | ToolCallEditFile
  | ToolCallSearchField
  | ToolCallUpdateTodos
