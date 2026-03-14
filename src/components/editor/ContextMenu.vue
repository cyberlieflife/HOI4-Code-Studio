<script setup lang="ts">
import { ref, computed } from 'vue'
import { useTheme } from '../../composables/useTheme'

const props = defineProps<{
  visible: boolean
  x: number
  y: number
  menuType: 'file' | 'tree' | 'pane' | 'sidebar' | 'editor'
  canSplit?: boolean
  currentFilePath?: string
  treeNodePath?: string
  treeNodeIsDirectory?: boolean
  hasTreeClipboard?: boolean
  treeSupportsFileOperations?: boolean
  projectRoot?: string
  availablePanes?: Array<{id: string, name: string}>
  sidebarCurrentSide?: 'left' | 'right'
}>()

// 获取当前主题。
const { currentTheme } = useTheme()

const emit = defineEmits<{
  action: [actionName: string, payload?: any]
  close: []
}>()

const templateMenuVisible = ref(false)
const moveMenuVisible = ref(false)

// 判断当前文件是否位于 common/ideas 目录。
const isInCommonIdeas = computed(() => {
  if (!props.currentFilePath) return false
  const normalizedPath = props.currentFilePath.replace(/\\/g, '/')
  return normalizedPath.includes('common/ideas/')
})

// 判断当前文件是否位于 history/countries 目录。
const isInHistoryCountries = computed(() => {
  if (!props.currentFilePath) return false
  const normalizedPath = props.currentFilePath.replace(/\\/g, '/')
  return normalizedPath.includes('history/countries/')
})

// 判断当前文件是否位于 common/bop 目录。
const isInCommonBop = computed(() => {
  if (!props.currentFilePath) return false
  const normalizedPath = props.currentFilePath.replace(/\\/g, '/')
  return normalizedPath.includes('common/bop/')
})

// 判断是否存在可用模板。
const hasAnyTemplateAvailable = computed(() => {
  return isInCommonIdeas.value || isInHistoryCountries.value || isInCommonBop.value
})

// 判断二级菜单是否需要显示在左侧，避免超出屏幕。
const showSubmenuOnLeft = computed(() => {
  const submenuWidth = 180
  const padding = 20
  return props.x + 200 + submenuWidth > window.innerWidth - padding
})

function normalizePath(path?: string) {
  return (path || '').replace(/\\/g, '/').replace(/\/+$/, '')
}

const isProjectMapDirectory = computed(() => {
  if (!props.treeNodeIsDirectory || !props.treeNodePath || !props.projectRoot) {
    return false
  }

  return normalizePath(props.treeNodePath) === `${normalizePath(props.projectRoot)}/map`
})

const canOperateTreeFiles = computed(() => props.treeSupportsFileOperations !== false)

function handleAction(action: string, payload?: any) {
  emit('action', action, payload)
}

function showTemplateMenu() {
  templateMenuVisible.value = true
}

function hideTemplateMenu() {
  templateMenuVisible.value = false
}

function showMoveMenu() {
  moveMenuVisible.value = true
}

function hideMoveMenu() {
  moveMenuVisible.value = false
}
</script>

<template>
  <!-- 文件标签页右键菜单 -->
  <div
    v-if="visible && menuType === 'file'"
    class="fixed border rounded-xl shadow-2xl z-50 backdrop-blur-sm"
    :style="{
      left: x + 'px',
      top: y + 'px',
      backgroundColor: currentTheme.colors.bgSecondary,
      borderColor: currentTheme.colors.border,
      color: currentTheme.colors.fg
    }"
    @click.stop
  >
    <button
      @click="handleAction('closeAll')"
      class="w-full px-4 py-2 text-left text-sm whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg }"
    >
      关闭全部
    </button>
    <button
      @click="handleAction('closeOthers')"
      class="w-full px-4 py-2 text-left text-sm border-t whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg, borderColor: currentTheme.colors.border }"
    >
      关闭其他
    </button>
  </div>

  <!-- 文件树右键菜单：可操作目录 -->
  <div
    v-if="visible && menuType === 'tree' && canOperateTreeFiles"
    class="fixed border rounded-xl shadow-2xl z-50 backdrop-blur-sm"
    :style="{
      left: x + 'px',
      top: y + 'px',
      backgroundColor: currentTheme.colors.bgSecondary,
      borderColor: currentTheme.colors.border,
      color: currentTheme.colors.fg
    }"
    @click.stop
  >
    <button
      @click="handleAction('createFile')"
      class="w-full px-4 py-2 text-left text-sm whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg }"
    >
      新建文件
    </button>
    <button
      @click="handleAction('createFolder')"
      class="w-full px-4 py-2 text-left text-sm border-t whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg, borderColor: currentTheme.colors.border }"
    >
      新建文件夹
    </button>
    <div class="h-px w-full my-1" :style="{ backgroundColor: currentTheme.colors.border }"></div>
    <button
      @click="handleAction('rename')"
      class="w-full px-4 py-2 text-left text-sm whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg }"
    >
      重命名
    </button>
    <button
      @click="handleAction('delete')"
      class="w-full px-4 py-2 text-left text-sm border-t whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg, borderColor: currentTheme.colors.border }"
    >
      删除
    </button>
    <button
      v-if="treeNodePath"
      @click="handleAction('copy')"
      class="w-full px-4 py-2 text-left text-sm border-t whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg, borderColor: currentTheme.colors.border }"
    >
      复制
    </button>
    <button
      v-if="treeNodePath"
      @click="handleAction('cut')"
      class="w-full px-4 py-2 text-left text-sm border-t whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg, borderColor: currentTheme.colors.border }"
    >
      剪切
    </button>
    <button
      v-if="hasTreeClipboard"
      @click="handleAction('paste')"
      class="w-full px-4 py-2 text-left text-sm border-t whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg, borderColor: currentTheme.colors.border }"
    >
      粘贴
    </button>
    <button
      @click="handleAction('copyPath')"
      class="w-full px-4 py-2 text-left text-sm whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg }"
    >
      复制路径
    </button>
    <button
      v-if="isProjectMapDirectory"
      @click="handleAction('previewMap')"
      class="w-full px-4 py-2 text-left text-sm border-t whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg, borderColor: currentTheme.colors.border }"
    >
      地图预览
    </button>
    <button
      @click="handleAction('showInExplorer')"
      class="w-full px-4 py-2 text-left text-sm whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg }"
    >
      在资源管理器中显示
    </button>
  </div>

  <!-- 文件树右键菜单：只读目录 -->
  <div
    v-if="visible && menuType === 'tree' && !canOperateTreeFiles"
    class="fixed border rounded-xl shadow-2xl z-50 backdrop-blur-sm"
    :style="{
      left: x + 'px',
      top: y + 'px',
      backgroundColor: currentTheme.colors.bgSecondary,
      borderColor: currentTheme.colors.border,
      color: currentTheme.colors.fg
    }"
    @click.stop
  >
    <button
      @click="handleAction('copyPath')"
      class="w-full px-4 py-2 text-left text-sm whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg }"
    >
      复制路径
    </button>
    <button
      @click="handleAction('showInExplorer')"
      class="w-full px-4 py-2 text-left text-sm border-t whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg, borderColor: currentTheme.colors.border }"
    >
      在资源管理器中显示
    </button>
  </div>

  <!-- 侧边栏标签右键菜单 -->
  <div
    v-if="visible && menuType === 'sidebar'"
    class="fixed border rounded-xl shadow-2xl z-50 backdrop-blur-sm"
    :style="{
      left: x + 'px',
      top: y + 'px',
      backgroundColor: currentTheme.colors.bgSecondary,
      borderColor: currentTheme.colors.border,
      color: currentTheme.colors.fg
    }"
    @click.stop
  >
    <button
      v-if="sidebarCurrentSide !== 'left'"
      @click="handleAction('moveSidebarItem', 'left')"
      class="w-full px-4 py-2 text-left text-sm whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg }"
    >
      移动到左侧边栏
    </button>
    <button
      v-if="sidebarCurrentSide !== 'right'"
      @click="handleAction('moveSidebarItem', 'right')"
      class="w-full px-4 py-2 text-left text-sm whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg }"
    >
      移动到右侧边栏
    </button>
  </div>

  <!-- 编辑器窗格右键菜单 -->
  <div
    v-if="visible && menuType === 'pane'"
    class="fixed border rounded-xl shadow-2xl z-50 backdrop-blur-sm"
    :style="{
      left: x + 'px',
      top: y + 'px',
      backgroundColor: currentTheme.colors.bgSecondary,
      borderColor: currentTheme.colors.border,
      color: currentTheme.colors.fg
    }"
    @click.stop
  >
    <button
      v-if="canSplit"
      @click="handleAction('splitRight')"
      class="w-full px-4 py-2 text-left text-sm whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg }"
    >
      向右分屏
    </button>
    <div
      v-if="availablePanes && availablePanes.length > 0"
      class="relative"
      @mouseenter="showMoveMenu"
      @mouseleave="hideMoveMenu"
    >
      <button
        class="w-full px-4 py-2 text-left text-sm whitespace-nowrap transition-colors context-menu-item flex items-center justify-between"
        :class="{ 'border-t': canSplit }"
        :style="{ color: currentTheme.colors.fg, borderColor: currentTheme.colors.border }"
      >
        <span>移动到</span>
        <span>▶</span>
      </button>
      <div
        v-if="moveMenuVisible"
        class="absolute top-0 border rounded-xl shadow-2xl backdrop-blur-sm"
        :class="showSubmenuOnLeft ? 'right-full mr-1' : 'left-full ml-1'"
        :style="{
          backgroundColor: currentTheme.colors.bgSecondary,
          borderColor: currentTheme.colors.border,
          minWidth: '150px',
          color: currentTheme.colors.fg
        }"
      >
        <button
          v-for="(pane, index) in availablePanes"
          :key="pane.id"
          @click="handleAction('moveToPane', pane.id)"
          class="w-full px-4 py-2 text-left text-sm whitespace-nowrap transition-colors context-menu-item"
          :class="{ 'border-t': index > 0 }"
          :style="{ color: currentTheme.colors.fg, borderColor: currentTheme.colors.border }"
        >
          {{ pane.name }}
        </button>
      </div>
    </div>
    <button
      @click="handleAction('closeAll')"
      class="w-full px-4 py-2 text-left text-sm border-t whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg, borderColor: currentTheme.colors.border }"
    >
      关闭全部
    </button>
    <button
      @click="handleAction('closeOthers')"
      class="w-full px-4 py-2 text-left text-sm border-t whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg, borderColor: currentTheme.colors.border }"
    >
      关闭其他
    </button>
  </div>

  <!-- 编辑器内容右键菜单 -->
  <div
    v-if="visible && menuType === 'editor'"
    class="fixed border rounded-xl shadow-2xl z-50 backdrop-blur-sm"
    :style="{
      left: x + 'px',
      top: y + 'px',
      backgroundColor: currentTheme.colors.bgSecondary,
      borderColor: currentTheme.colors.border,
      color: currentTheme.colors.fg
    }"
    @click.stop
  >
    <button
      @click="handleAction('selectAll')"
      class="w-full px-4 py-2 text-left text-sm whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg }"
    >
      全选
    </button>
    <div class="h-px w-full my-1" :style="{ backgroundColor: currentTheme.colors.border }"></div>
    <button
      @click="handleAction('copy')"
      class="w-full px-4 py-2 text-left text-sm whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg }"
    >
      复制
    </button>
    <button
      @click="handleAction('cut')"
      class="w-full px-4 py-2 text-left text-sm border-t whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg, borderColor: currentTheme.colors.border }"
    >
      剪切
    </button>
    <button
      @click="handleAction('paste')"
      class="w-full px-4 py-2 text-left text-sm border-t whitespace-nowrap transition-colors context-menu-item"
      :style="{ color: currentTheme.colors.fg, borderColor: currentTheme.colors.border }"
    >
      粘贴
    </button>
    <div
      v-if="hasAnyTemplateAvailable"
      class="h-px w-full my-1"
      :style="{ backgroundColor: currentTheme.colors.border }"
    ></div>
    <div
      v-if="hasAnyTemplateAvailable"
      class="relative"
      @mouseenter="showTemplateMenu"
      @mouseleave="hideTemplateMenu"
    >
      <button
        class="w-full px-4 py-2 text-left text-sm whitespace-nowrap transition-colors context-menu-item flex items-center justify-between"
        :style="{ color: currentTheme.colors.fg }"
      >
        <span>插入模板</span>
        <span>▶</span>
      </button>
      <div
        v-if="templateMenuVisible"
        class="absolute top-0 border rounded-xl shadow-2xl backdrop-blur-sm"
        :class="showSubmenuOnLeft ? 'right-full mr-1' : 'left-full ml-1'"
        :style="{
          backgroundColor: currentTheme.colors.bgSecondary,
          borderColor: currentTheme.colors.border,
          minWidth: '200px',
          color: currentTheme.colors.fg
        }"
      >
        <button
          v-if="isInCommonIdeas"
          @click="handleAction('insertIdeaTemplate')"
          class="w-full px-4 py-2 text-left text-sm whitespace-nowrap transition-colors context-menu-item"
          :style="{ color: currentTheme.colors.fg }"
        >
          插入 Idea 模板
        </button>
        <button
          v-if="isInHistoryCountries"
          @click="handleAction('insertTagTemplate')"
          class="w-full px-4 py-2 text-left text-sm whitespace-nowrap transition-colors context-menu-item"
          :class="{ 'border-t': isInCommonIdeas }"
          :style="{ color: currentTheme.colors.fg, borderColor: currentTheme.colors.border }"
        >
          插入 Tag 初始定义模板
        </button>
        <button
          v-if="isInCommonBop"
          @click="handleAction('insertBopTemplate')"
          class="w-full px-4 py-2 text-left text-sm whitespace-nowrap transition-colors context-menu-item"
          :class="{ 'border-t': isInCommonIdeas || isInHistoryCountries }"
          :style="{ color: currentTheme.colors.fg, borderColor: currentTheme.colors.border }"
        >
          插入权力平衡模板
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.context-menu-item {
  background-color: transparent;
}

.context-menu-item:hover {
  background-color: var(--theme-selection);
}
</style>
