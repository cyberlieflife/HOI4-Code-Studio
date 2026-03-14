<script setup lang="ts">
import { computed } from 'vue'
import type { Dependency } from '../../types/dependency'

const props = defineProps<{
  activeTab: 'project' | 'search' | 'dependencies' | 'plugins'
  activeDependencyId?: string
  dependencies: Dependency[]
}>()

const emit = defineEmits<{
  switchToProject: []
  switchToSearch: []
  switchToDependency: [id: string]
  switchToPlugins: []
  manageDependencies: []
}>()

// 已启用的依赖项
const enabledDependencies = computed(() => 
  props.dependencies.filter(dep => dep.enabled)
)
</script>

<template>
  <div class="ui-island-header ui-separator-bottom p-1 flex items-center gap-1 overflow-x-auto">
    <!-- 当前项目标签 -->
    <button
      @click="emit('switchToProject')"
      class="p-2 transition-all rounded-lg flex-shrink-0 hover-scale"
      :class="activeTab === 'project' ? 'bg-hoi4-accent text-hoi4-text' : 'text-hoi4-text-dim hover:text-hoi4-text hover:bg-hoi4-border/40'"
      title="当前项目"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"></path>
      </svg>
    </button>

    <!-- 依赖项标签 -->
    <button
      @click="emit('switchToSearch')"
      class="p-2 transition-all rounded-lg flex-shrink-0 hover-scale"
      :class="activeTab === 'search' ? 'bg-hoi4-accent text-hoi4-text' : 'text-hoi4-text-dim hover:text-hoi4-text hover:bg-hoi4-border/40'"
      title="搜索"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
      </svg>
    </button>

    <template v-if="enabledDependencies.length > 0">
      <div class="w-px h-6 bg-hoi4-border/40"></div>
      <button
        v-for="dep in enabledDependencies"
        :key="dep.id"
        @click="emit('switchToDependency', dep.id)"
        class="p-2 transition-all rounded-lg flex-shrink-0 relative group hover-scale"
        :class="activeTab === 'dependencies' && activeDependencyId === dep.id ? 'bg-hoi4-accent text-hoi4-text' : 'text-hoi4-text-dim hover:text-hoi4-text hover:bg-hoi4-border/40'"
        :title="dep.name"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path>
        </svg>
        <!-- 依赖项标记 -->
        <span class="absolute top-[2px] right-[2px] w-3 h-3 bg-hoi4-accent rounded-full text-[8px] flex items-center justify-center text-hoi4-text font-bold">
          {{ enabledDependencies.indexOf(dep) + 1 }}
        </span>
      </button>
    </template>

    <div class="w-px h-6 bg-hoi4-border/40"></div>
    <button
      @click="emit('switchToPlugins')"
      class="p-2 transition-all rounded-lg flex-shrink-0 hover-scale"
      :class="activeTab === 'plugins' ? 'bg-hoi4-accent text-hoi4-text' : 'text-hoi4-text-dim hover:text-hoi4-text hover:bg-hoi4-border/40'"
      title="插件面板"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v2h1a2 2 0 012 2v1h-2a2 2 0 100 4h2v1a2 2 0 01-2 2h-1v2a2 2 0 11-4 0v-2H9a2 2 0 01-2-2v-1h2a2 2 0 100-4H7V8a2 2 0 012-2h2V4z"></path>
      </svg>
    </button>

    <!-- 管理依赖项按钮 -->
    <div class="ml-auto flex items-center gap-1">
      <button
        @click="emit('manageDependencies')"
        class="p-2 transition-all rounded-lg flex-shrink-0 hover-scale"
        :class="'text-hoi4-text-dim hover:text-hoi4-text hover:bg-hoi4-border/40'"
        title="管理依赖项"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"></path>
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
/* 隐藏滚动条但保持滚动功能 */
.overflow-x-auto::-webkit-scrollbar {
  height: 4px;
}

.overflow-x-auto::-webkit-scrollbar-track {
  background: transparent;
}

.overflow-x-auto::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
}

.overflow-x-auto::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}

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
