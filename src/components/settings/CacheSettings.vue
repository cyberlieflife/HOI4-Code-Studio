<template>
  <div class="settings-form-group space-y-6">
    <!-- 缓存目录 -->
    <div>
      <h4 class="text-sm font-medium text-hoi4-text mb-2">缓存存储目录</h4>
      <p class="text-xs text-hoi4-text/50 mb-3">
        设置缓存文件的存储位置。更改后保存设置时，旧目录中的缓存内容将自动迁移到新目录。
      </p>
      <div class="flex space-x-2">
        <input
          :value="cacheDirectory"
          type="text"
          readonly
          :placeholder="defaultCacheDirectory || '加载中...'"
          class="input-field flex-1"
        />
        <button
          type="button"
          @click="selectCacheDirectory"
          class="btn-primary px-6"
        >
          浏览
        </button>
        <button
          v-if="cacheDirectory"
          type="button"
          @click="resetCacheDirectory"
          class="btn-secondary px-4"
          title="恢复默认缓存目录"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
          </svg>
        </button>
      </div>
      <p v-if="defaultCacheDirectory" class="text-xs text-hoi4-text/40 mt-2">
        默认目录: {{ defaultCacheDirectory }}
      </p>
    </div>

    <!-- 缓存信息 -->
    <div class="border-t border-hoi4-border pt-4">
      <h4 class="text-sm font-medium text-hoi4-text mb-2">缓存目录结构</h4>
      <p class="text-xs text-hoi4-text/50 mb-2">缓存目录下包含以下子目录：</p>
      <div class="bg-hoi4-gray/50 rounded-lg p-3 font-mono text-xs text-hoi4-text/70 space-y-1">
        <div>temp/</div>
        <div class="pl-4">focus-icon-cache/ <span class="text-hoi4-text/40">— 焦点图标缓存</span></div>
        <div class="pl-4">gfx-preview-cache/ <span class="text-hoi4-text/40">— GFX 预览缓存</span></div>
        <div>dds-conversion-cache/ <span class="text-hoi4-text/40">— DDS 转换缓存</span></div>
      </div>
    </div>

    <!-- 迁移状态 -->
    <div v-if="isMigrating" class="flex items-center space-x-2 text-sm text-hoi4-accent">
      <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
      </svg>
      <span>正在迁移缓存文件...</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { openFileDialog, getDefaultCacheDirectory } from '../../api/tauri'

interface Props {
  cacheDirectory: string
}

interface Emits {
  (e: 'update:cacheDirectory', value: string): void
  (e: 'status-message', message: string): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const cacheDirectory = ref(props.cacheDirectory)
const defaultCacheDirectory = ref('')
const isMigrating = ref(false)

// 加载默认缓存目录
onMounted(async () => {
  try {
    const result = await getDefaultCacheDirectory()
    if (result.success && result.data) {
      defaultCacheDirectory.value = result.data as string
    }
  } catch {
    // 忽略错误
  }
})

// 选择缓存目录
async function selectCacheDirectory() {
  const result = await openFileDialog('directory')
  if (result.success && result.path) {
    cacheDirectory.value = result.path
    emit('update:cacheDirectory', result.path)
  }
}

// 恢复默认缓存目录
function resetCacheDirectory() {
  cacheDirectory.value = ''
  emit('update:cacheDirectory', '')
  emit('status-message', '已恢复默认缓存目录')
}

// 监听外部变化
watch(() => props.cacheDirectory, (newValue) => {
  cacheDirectory.value = newValue
})

// 暴露迁移状态
defineExpose({ isMigrating })
</script>
