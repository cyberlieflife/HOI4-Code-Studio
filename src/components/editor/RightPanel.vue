<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import ProjectInfo from './ProjectInfo.vue'
import GameDirectory from './GameDirectory.vue'
import ErrorList from './ErrorList.vue'
import AIPanelConstruction from './AIPanelConstruction.vue'
import PluginIframeHost from '../plugins/PluginIframeHost.vue'
import type { FileNode } from '../../composables/useFileManager'
import type { PluginPanelRef } from '../../composables/usePluginManager'

const props = withDefaults(defineProps<{
  projectInfo: any
  gameDirectory: string
  gameFileTree: FileNode[]
  isLoadingGameTree: boolean
  txtErrors: Array<{line: number, msg: string, type: string}>
  width: number
  pluginPanels: PluginPanelRef[]
  activePluginPanelUid?: string
  activeTab?: 'info' | 'game' | 'errors' | 'ai' | 'plugins'
}>(), {
  activeTab: 'info'
})

const emit = defineEmits<{
  close: []
  resize: [event: MouseEvent]
  jumpToError: [error: {line: number, msg: string, type: string}]
  toggleGameFolder: [node: FileNode]
  openFile: [node: FileNode]
  'update:activeTab': [value: 'info' | 'game' | 'errors' | 'ai' | 'plugins']
  'update:activePluginPanelUid': [value: string]
}>()

const localActiveTab = ref<'info' | 'game' | 'errors' | 'ai' | 'plugins'>(props.activeTab)
const localActivePluginPanelUid = ref<string>(props.activePluginPanelUid || '')

const activePluginPanel = computed(() => {
  const panels = props.pluginPanels || []
  if (panels.length === 0) return null
  const byUid = panels.find(p => p.uid === localActivePluginPanelUid.value)
  return byUid || panels[0]
})

watch(() => props.activeTab, (newTab) => {
  if (newTab) {
    localActiveTab.value = newTab
  }
})

watch(() => props.activePluginPanelUid, (uid) => {
  if (uid !== undefined) {
    localActivePluginPanelUid.value = uid
  }
})

watch(() => props.pluginPanels, (panels) => {
  if (!panels || panels.length === 0) return
  if (!localActivePluginPanelUid.value) {
    localActivePluginPanelUid.value = panels[0].uid
    emit('update:activePluginPanelUid', panels[0].uid)
    return
  }
  if (!panels.some(p => p.uid === localActivePluginPanelUid.value)) {
    localActivePluginPanelUid.value = panels[0].uid
    emit('update:activePluginPanelUid', panels[0].uid)
  }
})

watch(localActiveTab, (newTab) => {
  emit('update:activeTab', newTab)
})

watch(localActivePluginPanelUid, (uid) => {
  if (uid) {
    emit('update:activePluginPanelUid', uid)
  }
})
</script>

<template>
  <div
    class="ui-island flex-shrink-0 overflow-hidden flex flex-col rounded-xl my-2 mr-2"
    :style="{ width: width + 'px' }"
  >
    <div class="ui-island-header ui-separator-bottom flex items-center justify-between rounded-t-xl">
      <div class="flex gap-1 p-1">
        <button
          @click="localActiveTab = 'info'"
          class="p-2 transition-all rounded-lg hover-scale"
          :class="localActiveTab === 'info' ? 'bg-hoi4-accent text-hoi4-text' : 'text-hoi4-text-dim hover:text-hoi4-text hover:bg-hoi4-border/40'"
          title="项目信息"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
          </svg>
        </button>
        <button
          @click="localActiveTab = 'game'"
          class="p-2 transition-all rounded-lg hover-scale"
          :class="localActiveTab === 'game' ? 'bg-hoi4-accent text-hoi4-text' : 'text-hoi4-text-dim hover:text-hoi4-text hover:bg-hoi4-border/40'"
          title="游戏目录"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"></path>
          </svg>
        </button>
        <button
          @click="localActiveTab = 'errors'"
          class="p-2 transition-all rounded-lg hover-scale"
          :class="localActiveTab === 'errors' ? 'bg-hoi4-accent text-hoi4-text' : 'text-hoi4-text-dim hover:text-hoi4-text hover:bg-hoi4-border/40'"
          title="错误列表"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"></path>
          </svg>
        </button>
        <button
          @click="localActiveTab = 'ai'"
          class="px-3 py-2 transition-all rounded-lg hover-scale text-sm font-bold"
          :class="localActiveTab === 'ai' ? 'bg-hoi4-accent text-hoi4-text' : 'text-hoi4-text-dim hover:text-hoi4-text hover:bg-hoi4-border/40'"
          title="AI"
        >
          AI
        </button>
        <button
          @click="localActiveTab = 'plugins'"
          class="p-2 transition-all rounded-lg hover-scale"
          :class="localActiveTab === 'plugins' ? 'bg-hoi4-accent text-hoi4-text' : 'text-hoi4-text-dim hover:text-hoi4-text hover:bg-hoi4-border/40'"
          title="插件"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v2h1a2 2 0 012 2v1h-2a2 2 0 100 4h2v1a2 2 0 01-2 2h-1v2a2 2 0 11-4 0v-2H9a2 2 0 01-2-2v-1h2a2 2 0 100-4H7V8a2 2 0 012-2h2V4z"></path>
          </svg>
        </button>
      </div>
      <button
        @click="emit('close')"
        class="px-3 text-hoi4-text-dim hover:text-hoi4-text rounded-md hover:bg-hoi4-border/40 transition-colors"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
        </svg>
      </button>
    </div>

    <div class="flex-1 overflow-hidden">
      <Transition name="sidebar-fade-slide" mode="out-in">
        <ProjectInfo
          v-if="localActiveTab === 'info'"
          :key="'info'"
          :project-info="projectInfo"
        />

        <GameDirectory
          v-else-if="localActiveTab === 'game'"
          :key="'game'"
          :game-directory="gameDirectory"
          :game-file-tree="gameFileTree"
          :is-loading="isLoadingGameTree"
          @toggle-folder="emit('toggleGameFolder', $event)"
          @open-file="emit('openFile', $event)"
        />

        <ErrorList
          v-else-if="localActiveTab === 'errors'"
          :key="'errors'"
          :errors="txtErrors"
          @jump-to-error="emit('jumpToError', $event)"
        />

        <AIPanelConstruction
          v-else-if="localActiveTab === 'ai'"
          :key="'ai'"
        />

        <div
          v-else-if="localActiveTab === 'plugins'"
          :key="'plugins'"
          class="h-full overflow-hidden flex flex-col"
        >
          <div class="p-2 ui-separator-bottom flex items-center gap-2 overflow-x-auto">
            <button
              v-for="p in pluginPanels"
              :key="p.uid"
              class="px-2 py-1 rounded text-xs flex-shrink-0"
              :class="p.uid === localActivePluginPanelUid ? 'bg-hoi4-accent text-hoi4-text' : 'bg-hoi4-border/40 text-hoi4-text-dim hover:text-hoi4-text hover:bg-hoi4-border/60'"
              @click="localActivePluginPanelUid = p.uid"
              :title="p.title"
            >
              {{ p.title }}
            </button>
          </div>
          <div class="flex-1 overflow-hidden">
            <div v-if="!activePluginPanel" class="p-3 text-hoi4-text-dim text-sm">暂无插件面板</div>
            <PluginIframeHost
              v-else
              :entry-file-path="activePluginPanel.entryFilePath"
              :plugin-id="activePluginPanel.pluginId"
              :plugin-name="activePluginPanel.pluginName"
              :side="activePluginPanel.side"
              :panel-id="activePluginPanel.panelId"
              :panel-title="activePluginPanel.title"
              :allowed-commands="activePluginPanel.allowedCommands"
            />
          </div>
        </div>
      </Transition>
    </div>
  </div>
</template>

<style scoped>
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
