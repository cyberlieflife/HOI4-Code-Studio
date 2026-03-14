<script setup lang="ts">
import { computed } from 'vue'
import packageJson from '../../../package.json'
import type { PluginToolbarItem } from '../../composables/usePluginManager'

// 程序版本号
const appVersion = packageJson.version

const props = defineProps<{
  projectName?: string
  rightPanelExpanded: boolean
  terminalVisible?: boolean
  isLaunchingGame?: boolean
  tagCount?: number
  ideaCount?: number
  autoSave?: boolean
  pluginToolbarItems?: PluginToolbarItem[]
}>()

const emit = defineEmits<{
  goBack: []
  toggleRightPanel: []
  launchGame: []
  manageDependencies: []
  toggleLoadingMonitor: []
  packageProject: []
  toggleAutoSave: []
  openModifierSheet: []
  toggleTerminalPanel: []
  pluginToolbarClick: [uid: string, open?: { side: 'left' | 'right'; panelUid: string }]
}>()

// 计算总加载数量
const totalLoadedCount = computed(() => {
  return (props.tagCount || 0) + (props.ideaCount || 0)
})
</script>

<template>
  <div class="flex items-center justify-between px-4 py-2 ui-topbar">
    <div class="flex items-center space-x-4">
      <button
        @click="emit('goBack')"
        class="p-2 bg-hoi4-accent/80 hover:bg-hoi4-border/40 active:bg-hoi4-border/60 rounded-md transition-colors"
        title="返回主页"
      >
        <svg class="w-5 h-5 text-hoi4-text" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18"></path>
        </svg>
      </button>
      <!-- 应用图标 -->
      <img 
        src="/HOICS.png" 
        alt="HOI4 Code Studio" 
        class="w-8 h-8 drop-shadow-sm"
      />
      <div class="flex items-baseline gap-2">
        <h1 class="text-hoi4-text font-bold text-lg">{{ projectName || '项目编辑器' }}</h1>
        <span class="text-hoi4-text-dim text-sm">v{{ appVersion }}</span>
      </div>
    </div>
    
    <div class="flex items-center space-x-2">
      <!-- 加载监控按钮 -->
      <button
        @click="emit('toggleLoadingMonitor')"
        class="p-2 bg-hoi4-accent/80 hover:bg-hoi4-border/40 active:bg-hoi4-border/60 rounded-md transition-colors relative"
        :title="`已加载 ${tagCount || 0} 个Tag, ${ideaCount || 0} 个Idea`"
      >
        <svg class="w-5 h-5 text-hoi4-text" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path>
        </svg>
        <!-- 数字徽章 -->
        <span 
          v-if="totalLoadedCount > 0"
          class="absolute -top-1 -right-1 bg-hoi4-accent text-hoi4-text text-xs font-bold rounded-full min-w-[1.25rem] h-5 flex items-center justify-center px-1"
        >
          {{ totalLoadedCount > 999 ? '999+' : totalLoadedCount }}
        </span>
      </button>
      <button
        @click="emit('launchGame')"
        :disabled="isLaunchingGame"
        class="p-2 bg-hoi4-accent/80 hover:bg-hoi4-border/40 active:bg-hoi4-border/60 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        :title="isLaunchingGame ? '启动中...' : '启动游戏'"
      >
        <svg 
          class="w-5 h-5 text-hoi4-text" 
          :class="{ 'animate-spin': isLaunchingGame }"
          fill="none" 
          stroke="currentColor" 
          viewBox="0 0 24 24"
        >
          <path v-if="!isLaunchingGame" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"></path>
          <path v-if="!isLaunchingGame" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
          <path v-if="isLaunchingGame" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
        </svg>
      </button>
      <button
        @click="emit('openModifierSheet')"
        class="p-2 bg-hoi4-accent/80 hover:bg-hoi4-border/40 active:bg-hoi4-border/60 rounded-md transition-colors"
        title="Modifier 速查表"
      >
        <svg class="w-5 h-5 text-hoi4-text" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012-2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01"></path>
        </svg>
      </button>

      <button
        v-for="it in (props.pluginToolbarItems || [])"
        :key="it.uid"
        @click="emit('pluginToolbarClick', it.uid, it.open)"
        class="p-2 bg-hoi4-accent/80 hover:bg-hoi4-border/40 active:bg-hoi4-border/60 rounded-md transition-colors"
        :title="`${it.title} (${it.pluginName})`"
      >
        <svg class="w-5 h-5 text-hoi4-text" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v2h1a2 2 0 012 2v1h-2a2 2 0 100 4h2v1a2 2 0 01-2 2h-1v2a2 2 0 11-4 0v-2H9a2 2 0 01-2-2v-1h2a2 2 0 100-4H7V8a2 2 0 012-2h2V4z"></path>
        </svg>
      </button>
      <button
        @click="emit('manageDependencies')"
        class="p-2 bg-hoi4-accent/80 hover:bg-hoi4-border/40 active:bg-hoi4-border/60 rounded-md transition-colors"
        title="依赖项管理"
      >
        <svg class="w-5 h-5 text-hoi4-text" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path>
        </svg>
      </button>
      <button
        @click="emit('packageProject')"
        class="p-2 bg-hoi4-accent/80 hover:bg-hoi4-border/40 active:bg-hoi4-border/60 rounded-md transition-colors"
        title="打包项目"
      >
        <svg class="w-5 h-5 text-hoi4-text" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"></path>
        </svg>
      </button>
      <button
        @click="emit('toggleTerminalPanel')"
        class="p-2 rounded-md transition-colors"
        :class="terminalVisible ? 'bg-green-600/80 hover:bg-green-700/80' : 'bg-hoi4-accent/80 hover:bg-hoi4-border/40 active:bg-hoi4-border/60'"
        :title="terminalVisible ? '隐藏终端面板' : '显示终端面板'"
      >
        <svg class="w-5 h-5 text-hoi4-text" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16M8 9l3 3-3 3"></path>
        </svg>
      </button>
      <button
        @click="emit('toggleAutoSave')"
        class="p-2 rounded-md transition-colors"
        :class="autoSave ? 'bg-green-600/80 hover:bg-green-700/80' : 'bg-hoi4-accent/80 hover:bg-hoi4-border/80'"
        :title="autoSave ? '自动保存已启用' : '自动保存已禁用'"
      >
        <svg class="w-5 h-5 text-hoi4-text" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4"></path>
        </svg>
      </button>
      <button
        @click="emit('toggleRightPanel')"
        class="p-2 bg-hoi4-accent/80 hover:bg-hoi4-border/40 active:bg-hoi4-border/60 rounded-md transition-colors"
        :title="rightPanelExpanded ? '隐藏侧边栏' : '显示侧边栏'"
      >
        <svg v-if="rightPanelExpanded" class="w-5 h-5 text-hoi4-text" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M5 5l7 7-7 7"></path>
        </svg>
        <svg v-else class="w-5 h-5 text-hoi4-text" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7"></path>
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
/* 悬停放大动画 */
.hover-scale {
  transition: background-color 0.2s ease;
}

.hover-scale:hover {
  transform: none;
  box-shadow: none;
}

.hover-scale:active {
  transform: none;
  transition: background-color 0.1s ease;
}
</style>
