<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
import { getFileIcon } from '../composables/useFileTreeIcons'

interface FileNode {
  name: string
  path: string
  isDirectory: boolean
  children?: FileNode[]
  expanded?: boolean
}

const props = defineProps<{
  node: FileNode
  level: number
  selectedPaths?: string[]
}>()

const emit = defineEmits<{
  select: [event: MouseEvent, node: FileNode]
  toggle: [node: FileNode]
  openFile: [node: FileNode]
  contextmenu: [event: MouseEvent, node: FileNode]
}>()

function handleClick(event: MouseEvent) {
  emit('select', event, props.node)

  if (event.ctrlKey || event.metaKey || event.shiftKey) {
    return
  }

  if (props.node.isDirectory) {
    emit('toggle', props.node)
  } else {
    emit('openFile', props.node)
  }
}

function getFileIconDisplay(node: FileNode) {
  return getFileIcon(node.name, node.isDirectory, node.expanded)
}

function isSelected(path: string) {
  return props.selectedPaths?.includes(path) ?? false
}
</script>

<template>
  <div>
    <div
      class="flex items-center px-2 py-1 rounded cursor-pointer text-sm file-tree-node transition-colors"
      :class="[isSelected(node.path) ? 'bg-hoi4-selected text-white' : 'hover:bg-hoi4-accent/50']"
      :style="{ paddingLeft: (level * 16 + 8) + 'px' }"
      @click="handleClick"
      @contextmenu.stop.prevent="(e) => emit('contextmenu', e, props.node)"
    >
      <!-- 图标显示 -->
      <span class="mr-2 flex items-center justify-center icon-container">
        <img
          v-if="getFileIconDisplay(node).type === 'svg'"
          :src="getFileIconDisplay(node).content"
          :alt="node.name"
          class="icon-svg"
          loading="lazy"
        />
        <span v-else class="icon-emoji">{{ getFileIconDisplay(node).content }}</span>
      </span>
      <span class="text-hoi4-text truncate">{{ node.name }}</span>
    </div>
    
    <div v-if="node.isDirectory && node.expanded && node.children">
      <FileTreeNode
        v-for="child in node.children"
        :key="child.path"
        :node="child"
        :level="level + 1"
        :selected-paths="selectedPaths"
        @select="(e, n) => emit('select', e, n)"
        @toggle="(n) => emit('toggle', n)"
        @open-file="(n) => emit('openFile', n)"
        @contextmenu="(e, n) => emit('contextmenu', e, n)"
      />
    </div>
  </div>
</template>

<style scoped>
.icon-container {
  width: 1.25rem;
  height: 1.25rem;
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-svg {
  width: 100%;
  height: 100%;
  object-fit: contain;
  flex-shrink: 0;
}

.icon-emoji {
  font-size: 1rem;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* 确保图标在不同主题下都有良好的对比度 */
.file-tree-node:hover .icon-svg {
  filter: brightness(1.1);
}

.file-tree-node[class*="bg-hoi4-selected"] .icon-svg {
  filter: brightness(1.2);
}
</style>
