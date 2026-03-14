<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import {
  startTerminalSession,
  stopTerminalSession,
  writeTerminalInput,
  type TerminalSessionInfo,
  type TerminalShell
} from '../../api/tauri'

const props = defineProps<{
  projectPath: string
}>()

const emit = defineEmits<{
  close: []
}>()

interface TerminalOutputEvent {
  sessionId: string
  stream: 'stdout' | 'stderr' | 'system'
  text: string
}

interface TerminalExitEvent {
  sessionId: string
  exitCode: number | null
}

const outputRef = ref<HTMLElement | null>(null)
const inlineInputRef = ref<HTMLElement | null>(null)
const currentShell = ref<TerminalShell>('powershell')
const session = ref<TerminalSessionInfo | null>(null)
const output = ref('')
const commandInput = ref('')
const commandHistory = ref<string[]>([])
const historyIndex = ref(-1)
const panelHeight = ref(260)
const status = ref<'starting' | 'ready' | 'closed'>('starting')

let removeOutputListener: (() => void) | null = null
let removeExitListener: (() => void) | null = null

const shellLabel = computed(() => currentShell.value === 'powershell' ? 'PowerShell' : 'CMD')
const statusLabel = computed(() => {
  if (status.value === 'starting') return '连接中'
  if (status.value === 'ready') return '运行中'
  return '已关闭'
})

function appendOutput(text: string) {
  if (!text) return
  output.value += text
  nextTick(() => {
    if (outputRef.value) {
      outputRef.value.scrollTop = outputRef.value.scrollHeight
    }
  })
}

function focusInlineInput() {
  if (status.value !== 'ready') return
  nextTick(() => {
    const element = inlineInputRef.value
    if (!element) return
    element.focus()

    const selection = window.getSelection()
    if (!selection) return

    const range = document.createRange()
    range.selectNodeContents(element)
    range.collapse(false)
    selection.removeAllRanges()
    selection.addRange(range)
  })
}

async function stopSession() {
  const activeSessionId = session.value?.sessionId
  session.value = null
  status.value = 'closed'

  if (!activeSessionId) return

  try {
    await stopTerminalSession(activeSessionId)
  } catch {
    // 会话可能已经自行退出，这里静默处理。
  }
}

async function startSession(shell: TerminalShell) {
  status.value = 'starting'
  currentShell.value = shell

  try {
    const nextSession = await startTerminalSession(shell, props.projectPath)
    session.value = nextSession
    status.value = 'ready'
    appendOutput(`\n[系统] 已启动 ${shell === 'powershell' ? 'PowerShell' : 'CMD'}，工作目录：${nextSession.cwd}\n`)
    focusInlineInput()
  } catch (error) {
    session.value = null
    status.value = 'closed'
    appendOutput(`\n[系统] 启动终端失败：${String(error)}\n`)
  }
}

async function restartSession(shell = currentShell.value) {
  await stopSession()
  output.value = ''
  commandInput.value = ''
  commandHistory.value = []
  historyIndex.value = -1
  await startSession(shell)
}

async function switchShell(shell: TerminalShell) {
  if (currentShell.value === shell && session.value) return
  await restartSession(shell)
}

async function executeCommand() {
  const command = commandInput.value.trim()
  const activeSessionId = session.value?.sessionId
  if (!command || !activeSessionId || status.value !== 'ready') return

  appendOutput(`\n[${shellLabel.value}] ${command}\n`)

  try {
    await writeTerminalInput(activeSessionId, `${command}\r\n`)
    commandHistory.value.push(command)
    historyIndex.value = commandHistory.value.length
    commandInput.value = ''
    if (inlineInputRef.value) {
      inlineInputRef.value.textContent = ''
    }
    focusInlineInput()
  } catch (error) {
    appendOutput(`\n[系统] 命令发送失败：${String(error)}\n`)
  }
}

function syncInlineInput() {
  const rawText = inlineInputRef.value?.textContent ?? ''
  commandInput.value = rawText.replace(/\r?\n/g, '')
}

function updateInlineInput(value: string) {
  commandInput.value = value
  if (inlineInputRef.value && inlineInputRef.value.textContent !== value) {
    inlineInputRef.value.textContent = value
  }
  focusInlineInput()
}

function handleInlineInput() {
  syncInlineInput()
}

function handleInlinePaste(event: ClipboardEvent) {
  event.preventDefault()
  const text = event.clipboardData?.getData('text/plain') ?? ''
  document.execCommand('insertText', false, text.replace(/\r?\n/g, ''))
  syncInlineInput()
}

function handleInputKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter') {
    event.preventDefault()
    void executeCommand()
    return
  }

  if (event.key === 'ArrowUp') {
    if (commandHistory.value.length === 0) return
    event.preventDefault()
    historyIndex.value = Math.max(0, historyIndex.value - 1)
    updateInlineInput(commandHistory.value[historyIndex.value] ?? '')
    return
  }

  if (event.key === 'ArrowDown') {
    if (commandHistory.value.length === 0) return
    event.preventDefault()
    historyIndex.value = Math.min(commandHistory.value.length, historyIndex.value + 1)
    updateInlineInput(historyIndex.value >= commandHistory.value.length
      ? ''
      : (commandHistory.value[historyIndex.value] ?? ''))
  }
}

function clearOutput() {
  output.value = ''
}

async function closePanel() {
  await stopSession()
  emit('close')
}

function startResize(event: MouseEvent) {
  const startY = event.clientY
  const startHeight = panelHeight.value

  const handleMouseMove = (moveEvent: MouseEvent) => {
    const nextHeight = startHeight - (moveEvent.clientY - startY)
    panelHeight.value = Math.min(480, Math.max(180, nextHeight))
  }

  const handleMouseUp = () => {
    document.removeEventListener('mousemove', handleMouseMove)
    document.removeEventListener('mouseup', handleMouseUp)
  }

  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', handleMouseUp)
}

onMounted(async () => {
  removeOutputListener = await listen<TerminalOutputEvent>('terminal-output', (event) => {
    if (event.payload.sessionId !== session.value?.sessionId) return
    appendOutput(event.payload.text)
  })

  removeExitListener = await listen<TerminalExitEvent>('terminal-exit', (event) => {
    if (event.payload.sessionId !== session.value?.sessionId) return
    appendOutput(`\n[系统] 终端已退出，退出码：${event.payload.exitCode ?? '未知'}\n`)
    session.value = null
    status.value = 'closed'
  })

  await startSession(currentShell.value)
})

onBeforeUnmount(async () => {
  removeOutputListener?.()
  removeExitListener?.()
  removeOutputListener = null
  removeExitListener = null
  await stopSession()
})
</script>

<template>
  <div
    class="ui-island mt-2 mx-2 mb-2 rounded-xl overflow-hidden flex flex-col"
    :style="{ height: `${panelHeight}px` }"
  >
    <div
      class="h-1 bg-hoi4-border hover:bg-hoi4-accent cursor-row-resize flex-shrink-0"
      @mousedown="startResize"
    ></div>

    <div class="px-4 py-2 ui-separator-bottom flex items-center justify-between gap-3 flex-wrap">
      <div class="flex items-center gap-2 flex-wrap">
        <span class="text-sm font-semibold text-hoi4-text">终端</span>
        <span class="text-xs text-hoi4-text-dim">
          {{ shellLabel }} · {{ statusLabel }}
        </span>
        <button
          class="px-2 py-1 rounded text-xs transition-colors"
          :class="currentShell === 'powershell' ? 'bg-hoi4-accent text-hoi4-text' : 'bg-hoi4-border/40 text-hoi4-text-dim hover:text-hoi4-text'"
          @click="switchShell('powershell')"
        >
          PowerShell
        </button>
        <button
          class="px-2 py-1 rounded text-xs transition-colors"
          :class="currentShell === 'cmd' ? 'bg-hoi4-accent text-hoi4-text' : 'bg-hoi4-border/40 text-hoi4-text-dim hover:text-hoi4-text'"
          @click="switchShell('cmd')"
        >
          CMD
        </button>
      </div>

      <div class="flex items-center gap-2">
        <button
          class="px-2 py-1 rounded text-xs bg-hoi4-border/40 text-hoi4-text-dim hover:text-hoi4-text transition-colors"
          @click="clearOutput"
        >
          清空
        </button>
        <button
          class="px-2 py-1 rounded text-xs bg-hoi4-border/40 text-hoi4-text-dim hover:text-hoi4-text transition-colors"
          @click="restartSession()"
        >
          重启
        </button>
        <button
          class="px-2 py-1 rounded text-xs bg-red-600/80 text-white hover:bg-red-500 transition-colors"
          @click="closePanel"
        >
          关闭
        </button>
      </div>
    </div>

    <div
      ref="outputRef"
      class="terminal-screen flex-1 overflow-auto px-4 py-3 font-mono text-sm leading-6 cursor-text"
      @click="focusInlineInput"
    >
      <div class="whitespace-pre-wrap break-words min-h-full">
        <div
          v-if="status === 'ready'"
          class="text-hoi4-text"
        >
          <span class="whitespace-pre-wrap break-words">{{ output }}</span>
          <span
            ref="inlineInputRef"
            contenteditable="true"
            spellcheck="false"
            class="inline-block min-h-6 min-w-[1ch] break-all outline-none align-baseline"
            @input="handleInlineInput"
            @keydown="handleInputKeydown"
            @paste="handleInlinePaste"
          ></span>
        </div>
        <div
          v-else
          class="text-hoi4-text-dim"
        >
          <span class="whitespace-pre-wrap break-words">{{ output }}</span>
          <br v-if="output" />
          终端未连接，可使用右上角的“重启”重新建立会话。
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.cursor-row-resize {
  cursor: row-resize;
}

.terminal-screen {
  background: linear-gradient(180deg, var(--theme-surface-1), var(--theme-bg-secondary));
  color: var(--theme-fg);
}

.terminal-screen ::selection {
  background: var(--theme-selection);
  color: var(--theme-fg);
}
</style>
