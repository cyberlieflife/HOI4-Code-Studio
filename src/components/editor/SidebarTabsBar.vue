<script setup lang="ts">
import { computed, ref } from 'vue'
import type { BuiltinSidebarTab, SidebarMovableItem, SidebarSide, SidebarView } from '../../composables/useEditorUiState'
import type { Dependency } from '../../types/dependency'

const props = withDefaults(defineProps<{
  side: SidebarSide
  items: SidebarMovableItem[]
  activeView: SidebarView
  dependencies: Dependency[]
  showManageDependencies?: boolean
  showClose?: boolean
}>(), {
  showManageDependencies: false,
  showClose: false
})

const emit = defineEmits<{
  activateItem: [key: string]
  activatePlugins: []
  manageDependencies: []
  close: []
  openContextMenu: [event: MouseEvent, key: string, side: SidebarSide]
  reorder: [draggedKey: string, targetKey?: string, position?: 'before' | 'after' | 'end']
}>()

const draggedKey = ref('')
const dropTargetKey = ref('')
const dropPosition = ref<'before' | 'after' | 'end'>('before')

const dependencyNameMap = computed(() => {
  const out = new Map<string, string>()
  for (const dependency of props.dependencies) {
    out.set(dependency.id, dependency.name)
  }
  return out
})

function isItemActive(item: SidebarMovableItem) {
  if (props.activeView.type === 'builtin') {
    return item.kind === 'builtin' && item.builtinId === props.activeView.builtinId
  }
  if (props.activeView.type === 'dependency') {
    return item.kind === 'dependency' && item.dependencyId === props.activeView.dependencyId
  }
  return false
}

function isPluginsActive() {
  return props.activeView.type === 'plugins'
}

function getBuiltinTitle(builtinId?: BuiltinSidebarTab) {
  switch (builtinId) {
    case 'project':
      return '项目文件'
    case 'search':
      return '搜索'
    case 'info':
      return '项目信息'
    case 'game':
      return '游戏目录'
    case 'errors':
      return '错误列表'
    case 'ai':
      return 'AI'
    default:
      return ''
  }
}

function getBuiltinBadge(builtinId?: BuiltinSidebarTab) {
  return builtinId === 'ai' ? 'AI' : ''
}

function getDependencyIndex(item: SidebarMovableItem) {
  if (item.kind !== 'dependency' || !item.dependencyId) return 0
  return props.dependencies.findIndex(dep => dep.id === item.dependencyId) + 1
}

function getItemTitle(item: SidebarMovableItem) {
  if (item.kind === 'builtin') {
    return getBuiltinTitle(item.builtinId)
  }
  return dependencyNameMap.value.get(item.dependencyId || '') || '依赖项'
}

function handleDragStart(event: DragEvent, key: string) {
  draggedKey.value = key
  dropTargetKey.value = ''
  dropPosition.value = 'before'
  if (!event.dataTransfer) return
  event.dataTransfer.setData('text/plain', key)
  event.dataTransfer.effectAllowed = 'move'
}

function handleDragOver(event: DragEvent, key: string) {
  if (!draggedKey.value || draggedKey.value === key) return
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move'
  }
  const target = event.currentTarget as HTMLElement | null
  if (!target) return
  const rect = target.getBoundingClientRect()
  const centerX = rect.left + rect.width / 2
  dropTargetKey.value = key
  dropPosition.value = event.clientX >= centerX ? 'after' : 'before'
}

function handleContainerDragOver(event: DragEvent) {
  if (!draggedKey.value) return
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move'
  }
  dropTargetKey.value = ''
  dropPosition.value = 'end'
}

function handleDrop(key?: string) {
  if (!draggedKey.value) return
  emit('reorder', draggedKey.value, key, key ? dropPosition.value : 'end')
  draggedKey.value = ''
  dropTargetKey.value = ''
  dropPosition.value = 'before'
}

function clearDragState() {
  draggedKey.value = ''
  dropTargetKey.value = ''
  dropPosition.value = 'before'
}
</script>

<template>
  <div class="ui-island-header ui-separator-bottom p-1 flex items-center gap-1 overflow-x-auto">
    <div
      class="flex items-center gap-1"
      @dragover.prevent="handleContainerDragOver"
      @drop.prevent="handleDrop()"
    >
      <button
        v-for="item in items"
        :key="item.key"
        type="button"
        draggable="true"
        @dragstart="handleDragStart($event, item.key)"
        @dragend="clearDragState"
        @dragover.prevent="handleDragOver($event, item.key)"
        @drop.prevent="handleDrop(item.key)"
        @click="emit('activateItem', item.key)"
        @contextmenu.prevent="emit('openContextMenu', $event, item.key, side)"
        class="p-2 transition-all rounded-lg flex-shrink-0 relative hover-scale border border-transparent"
        :class="[
          isItemActive(item)
            ? 'bg-hoi4-accent text-hoi4-text'
            : 'text-hoi4-text-dim hover:text-hoi4-text hover:bg-hoi4-border/40',
          dropTargetKey === item.key && dropPosition === 'before' ? 'border-l-hoi4-accent' : '',
          dropTargetKey === item.key && dropPosition === 'after' ? 'border-r-hoi4-accent' : ''
        ]"
        :title="getItemTitle(item)"
      >
        <span
          v-if="item.kind === 'builtin' && getBuiltinBadge(item.builtinId)"
          class="text-xs font-bold"
        >
          {{ getBuiltinBadge(item.builtinId) }}
        </span>
        <svg
          v-else-if="item.kind === 'builtin' && item.builtinId === 'project'"
          class="w-5 h-5"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"></path>
        </svg>
        <svg
          v-else-if="item.kind === 'builtin' && item.builtinId === 'search'"
          class="w-5 h-5"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
        </svg>
        <svg
          v-else-if="item.kind === 'builtin' && item.builtinId === 'info'"
          class="w-5 h-5"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
        </svg>
        <svg
          v-else-if="item.kind === 'builtin' && item.builtinId === 'game'"
          class="w-5 h-5"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"></path>
        </svg>
        <svg
          v-else-if="item.kind === 'builtin' && item.builtinId === 'errors'"
          class="w-5 h-5"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"></path>
        </svg>
        <svg
          v-else
          class="w-5 h-5"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path>
        </svg>

        <span
          v-if="item.kind === 'dependency'"
          class="absolute top-[2px] right-[2px] w-3 h-3 bg-hoi4-accent rounded-full text-[8px] flex items-center justify-center text-hoi4-text font-bold"
        >
          {{ getDependencyIndex(item) }}
        </span>
      </button>
    </div>

    <div class="w-px h-6 bg-hoi4-border/40"></div>

    <button
      @click="emit('activatePlugins')"
      class="p-2 transition-all rounded-lg flex-shrink-0 hover-scale"
      :class="isPluginsActive() ? 'bg-hoi4-accent text-hoi4-text' : 'text-hoi4-text-dim hover:text-hoi4-text hover:bg-hoi4-border/40'"
      title="插件面板"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v2h1a2 2 0 012 2v1h-2a2 2 0 100 4h2v1a2 2 0 01-2 2h-1v2a2 2 0 11-4 0v-2H9a2 2 0 01-2-2v-1h2a2 2 0 100-4H7V8a2 2 0 012-2h2V4z"></path>
      </svg>
    </button>

    <div class="ml-auto flex items-center gap-1">
      <button
        v-if="showManageDependencies"
        @click="emit('manageDependencies')"
        class="p-2 transition-all rounded-lg flex-shrink-0 hover-scale text-hoi4-text-dim hover:text-hoi4-text hover:bg-hoi4-border/40"
        title="管理依赖项"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"></path>
        </svg>
      </button>

      <button
        v-if="showClose"
        @click="emit('close')"
        class="px-3 text-hoi4-text-dim hover:text-hoi4-text rounded-md hover:bg-hoi4-border/40 transition-colors"
        title="关闭侧边栏"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
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
