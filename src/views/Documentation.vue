<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { documentationSections, type DocItem } from '../data/documentationContent'
import hljs from 'highlight.js'
import '../composables/useSyntaxHighlight'

type DetailBlock =
  | { type: 'text'; text: string }
  | { type: 'code'; language: string | null; code: string; html: string }

const router = useRouter()
const isLoading = ref(true)
const hoveredSection = ref<string | null>(null)
const hoveredItem = ref<string | null>(null)

const firstSection = documentationSections[0]
const firstItem: DocItem | null = firstSection && firstSection.items.length > 0
  ? firstSection.items[0]
  : null

const activeItemId = ref(firstItem ? firstItem.id : '')

const activeItem = computed<DocItem | null>(() => {
  for (const section of documentationSections) {
    const found = section.items.find(item => item.id === activeItemId.value)
    if (found) {
      return found
    }
  }
  return firstItem
})

function parseDetailBlocks(lines: string[]): DetailBlock[] {
  const blocks: DetailBlock[] = []

  function normalizeLanguage(lang: string | null): string | null {
    if (!lang) return null
    const l = lang.trim().toLowerCase()
    if (l.length === 0) return null
    if (l === 'js') return 'javascript'
    if (l === 'ts') return 'typescript'
    if (l === 'yml') return 'yaml'
    if (l === 'text') return 'plaintext'
    return l
  }

  function escapeHtml(s: string): string {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')
  }

  function highlightCodeBlock(code: string, lang: string | null): string {
    const normalized = normalizeLanguage(lang)
    try {
      if (!normalized) {
        return hljs.highlightAuto(code).value
      }

      const supported = hljs.getSupportedLanguages()
      if (supported.includes(normalized)) {
        return hljs.highlight(code, { language: normalized }).value
      }

      return hljs.highlightAuto(code).value
    } catch (_e) {
      return escapeHtml(code)
    }
  }

  let inCode = false
  let language: string | null = null
  let codeLines: string[] = []

  for (const rawLine of lines) {
    const line = rawLine ?? ''
    const trimmed = line.trim()
    const isFenceStart = trimmed.startsWith('```')
    const isFenceEnd = trimmed === '```'

    if (!inCode) {
      if (isFenceStart) {
        inCode = true
        const lang = trimmed.slice(3).trim()
        language = lang.length > 0 ? lang : null
        codeLines = []
        continue
      }

      blocks.push({ type: 'text', text: line })
      continue
    }

    if (isFenceEnd) {
      const code = codeLines.join('\n')
      blocks.push({ type: 'code', language, code, html: highlightCodeBlock(code, language) })
      inCode = false
      language = null
      codeLines = []
      continue
    }

    codeLines.push(line)
  }

  if (inCode) {
    const code = codeLines.join('\n')
    blocks.push({ type: 'code', language, code, html: highlightCodeBlock(code, language) })
  }

  return blocks
}

const activeDetailBlocks = computed<DetailBlock[]>(() => {
  if (!activeItem.value) return []
  return parseDetailBlocks(activeItem.value.details)
})



function handleSelectItem(id: string) {
  activeItemId.value = id
}

function goBack() {
  router.push('/')
}

onMounted(() => {
  setTimeout(() => {
    isLoading.value = false
  }, 300)
})
</script>

<template>
  <div class="h-full w-full flex flex-col bg-theme-bg relative overflow-hidden">
    <!-- 背景装饰 -->
    <div class="absolute inset-0 opacity-10">
      <div class="absolute top-10 left-10 w-64 h-64 bg-theme-accent/20 rounded-full filter blur-3xl animate-pulse"></div>
      <div class="absolute bottom-10 right-10 w-96 h-96 bg-theme-accent/10 rounded-full filter blur-3xl animate-pulse delay-1000"></div>
    </div>

    <!-- 加载状态 -->
    <div v-if="isLoading" class="absolute inset-0 flex items-center justify-center bg-theme-bg/80 backdrop-blur-sm z-50">
      <div class="text-center">
        <div class="inline-block animate-spin rounded-full h-12 w-12 border-4 border-theme-accent border-t-transparent mb-4"></div>
        <p class="text-theme-fg text-sm animate-pulse">正在加载文档...</p>
      </div>
    </div>

    <!-- 顶部导航岛 -->
    <header class="relative z-10 p-6">
      <div class="island-container max-w-7xl mx-auto">
        <div class="island inline-flex items-center space-x-4 p-4 backdrop-blur-xl bg-theme-bg-secondary border border-theme-border/20">
          <button
            @click="goBack"
            class="btn-island flex items-center space-x-2 px-4 py-2 rounded-xl bg-theme-accent/10 hover:bg-theme-accent/20 border border-theme-accent/30 hover:border-theme-accent/50 text-theme-fg transition-all duration-300 hover:scale-105 hover:shadow-lg"
          >
            <svg class="w-5 h-5 transition-transform duration-300 group-hover:-translate-x-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
            </svg>
            <span class="font-medium text-theme-fg">返回</span>
          </button>
          <div class="h-8 w-px bg-theme-border/30"></div>
          <h1 class="text-2xl font-bold text-theme-fg">
            使用文档
          </h1>
        </div>
      </div>
    </header>

    <!-- 主体内容区域 -->
    <main class="flex-1 relative z-10 px-6 pb-6 overflow-hidden">
      <div class="max-w-7xl mx-auto h-full flex gap-6">
        <!-- 左侧导航岛 -->
        <aside class="w-80 flex-shrink-0 h-full scroll-container">
          <div class="island h-full p-6 backdrop-blur-xl flex flex-col">
            <div class="flex-1 overflow-y-auto custom-scrollbar pr-2 space-y-6 scroll-content">
              <div
                v-for="section in documentationSections"
                :key="section.id"
                class="section-island"
                @mouseenter="hoveredSection = section.id"
                @mouseleave="hoveredSection = null"
              >
                <div 
                  class="section-header"
                  :class="{ 'section-header-active': hoveredSection === section.id }"
                >
                  <h3 class="text-theme-fg font-semibold text-base flex items-center">
                    <span class="w-2 h-2 rounded-full mr-2 transition-all duration-300"
                          :class="hoveredSection === section.id ? 'bg-theme-accent scale-125' : 'bg-theme-border'"></span>
                    {{ section.title }}
                  </h3>
                </div>
                <div class="mt-3 space-y-1">
                  <div
                    v-for="item in section.items"
                    :key="item.id"
                    @click="handleSelectItem(item.id)"
                    @mouseenter="hoveredItem = item.id"
                    @mouseleave="hoveredItem = null"
                    class="item-card cursor-pointer"
                    :class="{
                      'item-card-active': item.id === activeItemId,
                      'item-card-hover': hoveredItem === item.id && item.id !== activeItemId
                    }"
                  >
                    <div class="flex items-center space-x-3">
                      <div class="w-1.5 h-1.5 rounded-full transition-all duration-300"
                           :class="item.id === activeItemId ? 'bg-theme-accent' : 'bg-theme-border/50'"></div>
                      <span class="text-sm font-medium text-theme-fg">{{ item.title }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </aside>

        <!-- 右侧内容岛 -->
        <section class="flex-1 min-w-0 h-full scroll-container">
          <div class="island h-full p-8 backdrop-blur-xl flex flex-col">
            <div class="flex-1 overflow-y-auto custom-scrollbar pr-2 scroll-content">
              <div v-if="activeItem" class="content-animate space-y-8">
                <!-- 标题区域 -->
                <div class="content-header-island p-6 bg-theme-accent/10 border-l-4 border-theme-accent rounded-r-xl">
                  <h2 class="text-3xl font-bold text-theme-fg mb-4">
                    {{ activeItem.title }}
                  </h2>
                  <p class="text-theme-comment leading-relaxed text-base">
                    {{ activeItem.summary }}
                  </p>
                </div>

                <!-- 详细内容 -->
                <div class="content-body-island">
                  <ul class="space-y-4">
                    <li
                      v-for="(block, index) in activeDetailBlocks"
                      :key="index"
                      class="content-item p-4 bg-theme-bg-secondary/50 rounded-xl border border-theme-border/20 hover:border-theme-accent/30 transition-all duration-300 hover:shadow-lg"
                    >
                      <div v-if="block.type === 'text'" class="flex items-start space-x-3">
                        <div class="flex-shrink-0 w-6 h-6 rounded-full bg-theme-accent/20 flex items-center justify-center mt-0.5">
                          <div class="w-2 h-2 rounded-full bg-theme-accent"></div>
                        </div>
                        <p class="text-theme-fg leading-relaxed flex-1">
                          {{ block.text }}
                        </p>
                      </div>

                      <div v-else class="w-full">
                        <div class="text-xs text-theme-comment mb-2" v-if="block.language">{{ block.language }}</div>
                        <pre class="w-full overflow-x-auto rounded-lg ui-surface-1 border border-theme-border/20 p-4"><code class="text-theme-fg whitespace-pre hljs" v-html="block.html"></code></pre>
                      </div>
                    </li>
                  </ul>
                </div>
              </div>

              <div v-else class="flex items-center justify-center h-full">
                <div class="text-center">
                  <div class="w-20 h-20 mx-auto mb-4 rounded-full bg-theme-bg-secondary/30 flex items-center justify-center">
                    <svg class="w-10 h-10 text-theme-comment" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                    </svg>
                  </div>
                  <p class="text-theme-comment text-sm">暂无可用文档条目</p>
                </div>
              </div>
            </div>
          </div>
        </section>
      </div>
    </main>
  </div>
</template>
