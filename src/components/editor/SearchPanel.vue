<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { SearchResult, SearchScope } from '../../composables/useSearch'

const props = defineProps<{
  searchQuery: string
  searchResults: SearchResult[]
  isSearching: boolean
  searchCaseSensitive: boolean
  searchRegex: boolean
  searchScope: SearchScope
  includeAllFiles: boolean
  projectPath: string
  gameDirectory: string
}>()

const emit = defineEmits<{
  jumpToResult: [result: SearchResult]
  'update:searchQuery': [value: string]
  'update:searchCaseSensitive': [value: boolean]
  'update:searchRegex': [value: boolean]
  'update:searchScope': [value: SearchScope]
  'update:includeAllFiles': [value: boolean]
  performSearch: []
  performReplace: [replaceText: string]
}>()

const replaceText = ref('')
const selectedIndex = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)
const debounceTimer = ref<number | null>(null)

const isOpenFileScope = computed(() => props.searchScope === 'currentFile' || props.searchScope === 'openFiles')

function escapeHtml(input: string): string {
  return input
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

function getReplacePreviewHtml(result: SearchResult): string {
  const content = result.content ?? ''
  const start = Math.max(0, Math.min(result.matchStart ?? 0, content.length))
  const end = Math.max(start, Math.min(result.matchEnd ?? start, content.length))

  const before = escapeHtml(content.slice(0, start))
  const matched = escapeHtml(content.slice(start, end))
  const after = escapeHtml(content.slice(end))
  const replacement = escapeHtml(replaceText.value)

  if (!replacement) {
    return `${before}<span class="px-1 rounded bg-red-600/30 text-hoi4-text">${matched}</span>${after}`
  }

  return `${before}`
    + `<span class="px-1 rounded bg-red-600/30 text-hoi4-text">${matched}</span>`
    + `<span class="mx-1 text-[10px] text-hoi4-text-dim">→</span>`
    + `<span class="px-1 rounded bg-green-600/30 text-hoi4-text">${replacement}</span>`
    + `${after}`
}

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    selectedIndex.value = Math.min(selectedIndex.value + 1, props.searchResults.length - 1)
    return
  }

  if (event.key === 'ArrowUp') {
    event.preventDefault()
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
    return
  }

  if (event.key === 'Enter' && props.searchResults[selectedIndex.value]) {
    emit('jumpToResult', props.searchResults[selectedIndex.value])
  }
}

watch(
  [
    () => props.searchQuery,
    () => props.searchCaseSensitive,
    () => props.searchRegex,
    () => props.searchScope,
    () => props.includeAllFiles
  ],
  () => {
    if (debounceTimer.value) {
      clearTimeout(debounceTimer.value)
      debounceTimer.value = null
    }

    if (!props.searchQuery.trim()) {
      return
    }

    debounceTimer.value = window.setTimeout(() => {
      emit('performSearch')
    }, 1000)
  },
  { flush: 'post' }
)

function handleReplaceAll() {
  if (!props.searchQuery.trim()) return
  emit('performReplace', replaceText.value)
}

setTimeout(() => {
  inputRef.value?.focus()
}, 100)
</script>

<template>
  <div
    class="h-full flex flex-col overflow-hidden bg-hoi4-gray text-hoi4-text"
    @keydown="handleKeyDown"
  >
    <div class="p-4 ui-separator-bottom">
      <input
        ref="inputRef"
        :value="searchQuery"
        @input="emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
        type="text"
        placeholder="搜索..."
        class="ui-input w-full px-3 py-2"
      />

      <div class="mt-3 flex gap-2">
        <input
          v-model="replaceText"
          type="text"
          placeholder="替换为..."
          class="ui-input flex-1 px-2 py-1 text-sm"
        />
        <button
          type="button"
          class="px-2 py-1 rounded bg-hoi4-border/40 hover:bg-hoi4-border/60 text-hoi4-text text-xs transition-colors whitespace-nowrap"
          :disabled="isSearching || !searchQuery.trim() || searchResults.length === 0"
          @click="handleReplaceAll"
        >
          替换全部
        </button>
      </div>

      <div class="flex items-center gap-3 mt-3 text-xs flex-wrap">
        <label class="flex items-center gap-2 text-hoi4-text cursor-pointer">
          <input
            :checked="searchCaseSensitive"
            @change="emit('update:searchCaseSensitive', ($event.target as HTMLInputElement).checked)"
            type="checkbox"
            class="accent-hoi4-accent"
          />
          <span>区分大小写</span>
        </label>
        <label class="flex items-center gap-2 text-hoi4-text cursor-pointer">
          <input
            :checked="searchRegex"
            @change="emit('update:searchRegex', ($event.target as HTMLInputElement).checked)"
            type="checkbox"
            class="accent-hoi4-accent"
          />
          <span>正则表达式</span>
        </label>
        <div class="flex items-center gap-2 flex-wrap">
          <label class="flex items-center gap-2 text-hoi4-text cursor-pointer">
            <input
              :checked="searchScope === 'currentFile'"
              @change="emit('update:searchScope', 'currentFile')"
              type="radio"
              name="searchScope"
              class="accent-hoi4-accent"
            />
            <span>当前文件</span>
          </label>
          <label class="flex items-center gap-2 text-hoi4-text cursor-pointer">
            <input
              :checked="searchScope === 'openFiles'"
              @change="emit('update:searchScope', 'openFiles')"
              type="radio"
              name="searchScope"
              class="accent-hoi4-accent"
            />
            <span>已打开文件</span>
          </label>
          <label class="flex items-center gap-2 text-hoi4-text cursor-pointer">
            <input
              :checked="searchScope === 'project'"
              @change="emit('update:searchScope', 'project')"
              type="radio"
              name="searchScope"
              class="accent-hoi4-accent"
            />
            <span>项目目录</span>
          </label>
          <label class="flex items-center gap-2 text-hoi4-text cursor-pointer">
            <input
              :checked="searchScope === 'game'"
              @change="emit('update:searchScope', 'game')"
              type="radio"
              name="searchScope"
              class="accent-hoi4-accent"
            />
            <span>游戏目录</span>
          </label>
          <label class="flex items-center gap-2 text-hoi4-text cursor-pointer">
            <input
              :checked="searchScope === 'dependencies'"
              @change="emit('update:searchScope', 'dependencies')"
              type="radio"
              name="searchScope"
              class="accent-hoi4-accent"
            />
            <span>依赖目录</span>
          </label>
        </div>
        <label class="flex items-center gap-2 text-hoi4-text cursor-pointer">
          <input
            :checked="includeAllFiles"
            @change="emit('update:includeAllFiles', ($event.target as HTMLInputElement).checked)"
            type="checkbox"
            class="accent-hoi4-accent"
            :disabled="isOpenFileScope"
          />
          <span :class="{ 'opacity-50': isOpenFileScope }">所有文件类型</span>
        </label>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto">
      <div v-if="isSearching" class="p-4 text-center text-hoi4-text-dim">
        搜索中...
      </div>
      <div v-else-if="searchResults.length === 0 && searchQuery" class="p-4 text-center text-hoi4-text-dim">
        未找到结果
      </div>
      <div v-else-if="searchResults.length > 0" class="p-2 space-y-2">
        <div class="text-xs text-hoi4-text-dim px-2 mb-2">
          找到 {{ searchResults.length }} 个结果
        </div>
        <div
          v-for="(result, index) in searchResults"
          :key="`${result.file.path}-${result.line}-${index}`"
          :class="[
            'px-4 py-3 rounded-lg cursor-pointer transition-all duration-200 shadow-sm border-transparent',
            index === selectedIndex
              ? 'bg-hoi4-accent/30 shadow-md'
              : 'bg-hoi4-dark/70 hover:bg-hoi4-dark/90 hover:shadow-md'
          ]"
          @click="emit('jumpToResult', result)"
        >
          <div class="text-xs font-semibold text-hoi4-accent mb-1 flex items-center gap-2">
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4"></path>
            </svg>
            {{ result.file.name }}:{{ result.line }}
          </div>
          <div class="text-xs text-hoi4-text font-mono bg-hoi4-dark/30 p-2 rounded">
            <span v-html="getReplacePreviewHtml(result)"></span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
