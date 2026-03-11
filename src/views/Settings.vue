<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { loadSettings, saveSettings, openUrl } from '../api/tauri'
import { useTheme } from '../composables/useTheme'
import { useFileTreeIcons, iconSets } from '../composables/useFileTreeIcons'
import { getDefaultMenuItem } from '../data/settingsMenu'
import MarkdownIt from 'markdown-it'

// 导入子组件
import SettingsSidebar from '../components/settings/SettingsSidebar.vue'
import SettingsCard from '../components/settings/SettingsCard.vue'
import GameDirectorySettings from '../components/settings/GameDirectorySettings.vue'
import GameLaunchSettings from '../components/settings/GameLaunchSettings.vue'
import EditorFontSettings from '../components/settings/EditorFontSettings.vue'
import EditorSaveSettings from '../components/settings/EditorSaveSettings.vue'
import ThemeSettings from '../components/settings/ThemeSettings.vue'
import PluginSettings from '../components/settings/PluginSettings.vue'
import IconSettings from '../components/settings/IconSettings.vue'
import UpdateSettings from '../components/settings/UpdateSettings.vue'
import VersionInfoSettings from '../components/settings/VersionInfoSettings.vue'
import AISettings from '../components/settings/AISettings.vue'

// 主题系统
const { currentThemeId } = useTheme()
const { themes: allThemes } = useTheme()

// 图标系统
const { currentIconSetId } = useFileTreeIcons()

const router = useRouter()

// 当前选中的菜单项
const activeMenuItem = ref(getDefaultMenuItem())

// 处理菜单点击
function handleMenuItemClick(itemId: string) {
  activeMenuItem.value = itemId
}

// 设置数据
const gameDirectory = ref('')
const checkForUpdatesOnStartup = ref(true)

const autoSave = ref(true)
const disableErrorHandling = ref(false)
const enableRGBColorDisplay = ref(true)

// 地图预览设置
const mapPerformanceMode = ref(true)
const mapSamplingRate = ref(50)

// AI 设置
const openaiApiKey = ref('')
const openaiBaseUrl = ref('https://api.openai.com')
const openaiModel = ref('gpt-4o-mini')
const aiRenderMarkdown = ref(false)
const aiRequestReasoning = ref(false)
const aiRule = ref('')
const aiAgentMode = ref<'plan' | 'code' | 'ask'>('plan')

// 编辑器字体设置
import { useEditorFont } from '../composables/useEditorFont'
const { 
  fontConfig, 
  setFontConfig 
} = useEditorFont()

// 游戏启动设置
const useSteamVersion = ref(true)
const usePirateVersion = ref(false)
const pirateExecutable = ref<'dowser' | 'hoi4'>('dowser')
const launchWithDebug = ref(false)

// 状态
const showStatus = ref(false)
const statusMessage = ref('')
const isSaving = ref(false)
// 确认提示
const showConfirmDialog = ref(false)
const pendingDisableErrorHandling = ref(false)

// 版本信息
const CURRENT_VERSION = 'v0.3.5-dev'
const currentVersion = ref(CURRENT_VERSION)
const githubVersion = ref('检查中...')
const isCheckingUpdate = ref(false)
const isInitializing = ref(true)

// 更新对话框
const showUpdateDialog = ref(false)
const updateInfo = ref<{ version: string; url: string; releaseNotes?: string } | null>(null)

const md = new MarkdownIt({
  html: false,
  linkify: true,
  breaks: true
})

function updateNotesHtml() {
  const notes = updateInfo.value?.releaseNotes
  if (!notes || !notes.trim()) return ''
  return md.render(notes)
}

function handleShowUpdateDialog(info: { version: string; url: string; releaseNotes?: string }) {
  updateInfo.value = info
  showUpdateDialog.value = true
}

// 显示状态消息
function displayStatus(message: string, duration: number = 3000) {
  statusMessage.value = message
  showStatus.value = true
  
  setTimeout(() => {
    showStatus.value = false
  }, duration)
}

// 加载设置
async function loadUserSettings() {
  const result = await loadSettings()
  if (result.success && result.data) {
    const data = result.data as any
    gameDirectory.value = data.gameDirectory || ''
    checkForUpdatesOnStartup.value = data.checkForUpdates !== false
    useSteamVersion.value = data.useSteamVersion !== false
    usePirateVersion.value = data.usePirateVersion || false
    pirateExecutable.value = data.pirateExecutable || 'dowser'
    launchWithDebug.value = data.launchWithDebug || false
    autoSave.value = data.autoSave !== false
    disableErrorHandling.value = data.disableErrorHandling || false
    enableRGBColorDisplay.value = data.enableRGBColorDisplay !== false

    // 加载地图设置
    mapPerformanceMode.value = data.mapPerformanceMode !== false
    mapSamplingRate.value = data.mapSamplingRate || (data.mapDownsample ? Math.round(100 / data.mapDownsample) : 50)

    // 加载 AI 设置
    openaiApiKey.value = data.openaiApiKey || ''
    openaiBaseUrl.value = data.openaiBaseUrl || 'https://api.openai.com'
    openaiModel.value = data.openaiModel || 'gpt-4o-mini'
    aiRenderMarkdown.value = data.aiRenderMarkdown || false
    aiRequestReasoning.value = data.aiRequestReasoning || false
    aiRule.value = data.aiRule || data.aiSystemPrompt || ''
    aiAgentMode.value = (data.aiAgentMode as 'plan' | 'code' | 'ask') || 'plan'

    // 加载编辑器字体设置
    if (data.editorFont) {
      setFontConfig(data.editorFont)
    } else {
      // 兼容旧的字体设置格式
      setFontConfig({
        family: data.editorFontFamily || 'Consolas',
        size: data.editorFontSize || 14,
        weight: data.editorFontWeight || '400'
      })
    }
    // 加载主题设置
    if (data.theme && allThemes.value.some(t => t.id === data.theme)) {
      currentThemeId.value = data.theme
    }
    
    // 加载图标设置
    if (data.iconSet && iconSets.some(set => set.id === data.iconSet)) {
      currentIconSetId.value = data.iconSet
    }
  }
}

// 保存设置（静默保存，仅在失败时提示）
async function handleSave() {
  isSaving.value = true
  
  const settings = {
    gameDirectory: gameDirectory.value,
    checkForUpdates: checkForUpdatesOnStartup.value,
    useSteamVersion: useSteamVersion.value,
    usePirateVersion: usePirateVersion.value,
    pirateExecutable: pirateExecutable.value,
    launchWithDebug: launchWithDebug.value,
    autoSave: autoSave.value,
    disableErrorHandling: disableErrorHandling.value,
    enableRGBColorDisplay: enableRGBColorDisplay.value,
    theme: currentThemeId.value,
    iconSet: currentIconSetId.value,
    editorFont: fontConfig.value,

    // AI 设置
    openaiApiKey: openaiApiKey.value,
    openaiBaseUrl: openaiBaseUrl.value,
    openaiModel: openaiModel.value,
    aiRenderMarkdown: aiRenderMarkdown.value,
    aiRequestReasoning: aiRequestReasoning.value,
    aiRule: aiRule.value,
    aiAgentMode: aiAgentMode.value,

    // 地图设置
    mapPerformanceMode: mapPerformanceMode.value,
    mapSamplingRate: mapSamplingRate.value
  }
  
  const result = await saveSettings(settings)
  
  // 只在保存失败时显示提示
  if (!result.success) {
    displayStatus(`保存失败: ${result.message}`, 3000)
  }
  
  isSaving.value = false
}



// 打开更新页面
async function openUpdatePage() {
  if (updateInfo.value?.url) {
    await openUrl(updateInfo.value.url)
    showUpdateDialog.value = false
  }
}

// 关闭更新对话框
function closeUpdateDialog() {
  showUpdateDialog.value = false
}

// 返回主界面（自动保存）
async function goBack() {
  // 自动保存设置
  await handleSave()
  router.push('/')
}

// 监听自动保存开关变化，立即保存
watch(autoSave, async () => {
  await handleSave()
})

// 监听错误处理开关变化，需要确认
watch(disableErrorHandling, async (newValue, oldValue) => {
  // 忽略初始加载时的变化（从undefined到具体值）
  if (oldValue === undefined) {
    return
  }
  
  // 如果正在初始化，不触发任何操作
  if (isInitializing.value) {
    return
  }
  
  // 如果当前正在处理确认对话框，不重复触发
  if (pendingDisableErrorHandling.value) {
    return
  }
  
  if (newValue && !oldValue) {
    // 用户尝试启用（关闭错误处理），显示确认提示
    pendingDisableErrorHandling.value = true
    showConfirmDialog.value = true
  } else {
    // 直接保存
    await handleSave()
  }
})

// 确认禁用错误处理
async function confirmDisableErrorHandling() {
  disableErrorHandling.value = true
  pendingDisableErrorHandling.value = false
  showConfirmDialog.value = false
  await handleSave()
}

// 取消禁用错误处理
function cancelDisableErrorHandling() {
  disableErrorHandling.value = false
  pendingDisableErrorHandling.value = false
  showConfirmDialog.value = false
}

onMounted(async () => {
  await loadUserSettings()
  // 初始化完成后，标记为false
  isInitializing.value = false
})
</script>

<template>
  <div class="settings-container">
    <!-- 左侧菜单 -->
    <div class="settings-sidebar">
      <SettingsSidebar 
        :active-item="activeMenuItem"
        @item-click="handleMenuItemClick"
      />
    </div>

    <!-- 右侧内容区 -->
    <div class="settings-main">
      <!-- 顶部栏 -->
      <div class="settings-header">
        <button
          @click="goBack"
          class="btn-secondary flex items-center space-x-2"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18"></path>
          </svg>
          <span>返回</span>
        </button>
        <h1 class="ml-6 font-bold text-hoi4-text text-2xl">
          设置
        </h1>
      </div>

      <!-- 设置内容 -->
      <div class="settings-content">
        <div class="settings-content-inner">
          <!-- 游戏目录设置 -->
          <SettingsCard 
            v-if="activeMenuItem === 'game-directory'"
            title="HOI4 游戏目录"
            description="选择 Hearts of Iron IV 游戏安装目录"
          >
            <GameDirectorySettings 
              v-model="gameDirectory"
              @status-message="displayStatus"
            />
          </SettingsCard>

          <!-- 游戏启动设置 -->
          <SettingsCard 
            v-if="activeMenuItem === 'game-launch'"
            title="游戏启动设置"
            description="配置游戏启动方式和参数"
          >
            <GameLaunchSettings 
              :use-steam-version="useSteamVersion"
              :use-pirate-version="usePirateVersion"
              :pirate-executable="pirateExecutable"
              :launch-with-debug="launchWithDebug"
              @update:useSteamVersion="useSteamVersion = $event"
              @update:usePirateVersion="usePirateVersion = $event"
              @update:pirateExecutable="pirateExecutable = $event"
              @update:launchWithDebug="launchWithDebug = $event"
              @save="handleSave"
            />
          </SettingsCard>

          <!-- 编辑器字体设置 -->
          <SettingsCard 
            v-if="activeMenuItem === 'editor-font'"
            title="编辑器字体设置"
            description="自定义编辑器字体样式"
          >
            <EditorFontSettings 
              @save="handleSave"
            />
          </SettingsCard>

          <!-- AI 设置 -->
          <SettingsCard 
            v-if="activeMenuItem === 'ai-settings'"
            title="AI 设置"
            description="配置 OpenAI 兼容接口与渲染选项"
          >
            <AISettings
              :openai-api-key="openaiApiKey"
              :openai-base-url="openaiBaseUrl"
              :openai-model="openaiModel"
              :ai-render-markdown="aiRenderMarkdown"
              :ai-request-reasoning="aiRequestReasoning"
              :ai-rule="aiRule"
              :ai-agent-mode="aiAgentMode"
              @update:openaiApiKey="openaiApiKey = $event"
              @update:openaiBaseUrl="openaiBaseUrl = $event"
              @update:openaiModel="openaiModel = $event"
              @update:aiRenderMarkdown="aiRenderMarkdown = $event"
              @update:aiRequestReasoning="aiRequestReasoning = $event"
              @update:aiRule="aiRule = $event"
              @update:aiAgentMode="aiAgentMode = $event"
              @save="handleSave"
            />
          </SettingsCard>

          <!-- 地图设置 -->
          <SettingsCard 
            v-if="activeMenuItem === 'map-settings'"
            title="地图预览设置"
            description="配置地图渲染性能和缩放比例"
          >
            <div class="space-y-6">
              <!-- 性能模式 -->
              <div class="flex items-center justify-between">
                <div>
                  <h4 class="text-sm font-medium text-hoi4-text">性能模式</h4>
                  <p class="text-xs text-hoi4-text/50">开启后将以较低分辨率渲染地图，显著提升加载速度和交互流畅度。</p>
                </div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" v-model="mapPerformanceMode" @change="handleSave" class="sr-only peer">
                  <div class="w-11 h-6 bg-hoi4-gray rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-hoi4-accent"></div>
                </label>
              </div>

              <!-- 缩放比例 -->
              <div>
                <div class="flex items-center justify-between mb-2">
                  <h4 class="text-sm font-medium text-hoi4-text">性能模式采样率</h4>
                  <span class="text-xs font-mono text-hoi4-accent">{{ mapSamplingRate }}%</span>
                </div>
                <input 
                  type="range" 
                  v-model.number="mapSamplingRate" 
                  @change="handleSave"
                  min="10" 
                  max="100" 
                  step="5"
                  class="w-full h-1.5 bg-hoi4-gray rounded-lg appearance-none cursor-pointer accent-hoi4-accent"
                >
                <div class="flex justify-between mt-1 px-1 text-[10px] text-hoi4-text/30">
                  <span>10%</span>
                  <span>25%</span>
                  <span>50%</span>
                  <span>75%</span>
                  <span>100%</span>
                </div>
                <p class="text-xs text-hoi4-text/50 mt-2">调整性能模式下的采样率。采样率越低，地图加载和操作越流畅，但图像细节越模糊。</p>
              </div>
            </div>
          </SettingsCard>

          <!-- 保存设置 -->
          <SettingsCard 
            v-if="activeMenuItem === 'editor-save'"
            title="保存设置"
            description="配置自动保存和错误处理"
          >
            <EditorSaveSettings 
              :autoSave="autoSave"
              :disableErrorHandling="disableErrorHandling"
              :enableRGBColorDisplay="enableRGBColorDisplay"
              @update:autoSave="autoSave = $event"
              @update:disableErrorHandling="disableErrorHandling = $event"
              @update:enableRGBColorDisplay="enableRGBColorDisplay = $event"
              @save="handleSave"
              @confirm-disable-error-handling="showConfirmDialog = true"
            />
          </SettingsCard>

          <!-- 主题设置 -->
          <SettingsCard 
            v-if="activeMenuItem === 'theme'"
            title="界面主题"
            description="选择应用界面主题"
          >
            <ThemeSettings />
          </SettingsCard>

          <SettingsCard
            v-if="activeMenuItem === 'plugins'"
            title="插件"
            description="管理已安装的插件"
          >
            <PluginSettings />
          </SettingsCard>

          <!-- 图标设置 -->
          <SettingsCard 
            v-if="activeMenuItem === 'icons'"
            title="文件树图标"
            description="选择文件树图标样式"
          >
            <IconSettings />
          </SettingsCard>

          <!-- 更新设置 -->
          <SettingsCard 
            v-if="activeMenuItem === 'update-settings'"
            title="更新设置"
            description="配置应用更新选项"
          >
            <UpdateSettings 
              :check-for-updates-on-startup="checkForUpdatesOnStartup"
              @update:checkForUpdatesOnStartup="checkForUpdatesOnStartup = $event"
              @save="handleSave"
            />
          </SettingsCard>

          <!-- 版本信息 -->
          <SettingsCard 
            v-if="activeMenuItem === 'version-info'"
            title="版本信息"
            description="查看当前版本和检查更新"
          >
            <VersionInfoSettings 
              :current-version="currentVersion"
              :github-version="githubVersion"
              :is-checking-update="isCheckingUpdate"
              @update:githubVersion="githubVersion = $event"
              @update:isCheckingUpdate="isCheckingUpdate = $event"
              @status-message="displayStatus"
              @show-update-dialog="handleShowUpdateDialog"
            />
          </SettingsCard>
        </div>
      </div>
    </div>
                

    <!-- 状态提示 -->
    <div v-if="showStatus" class="fixed bottom-[2vh] right-[2vw] z-50">
      <div class="bg-hoi4-gray border-2 border-hoi4-border rounded-lg shadow-lg p-4">
        <p class="text-hoi4-text">{{ statusMessage }}</p>
      </div>
    </div>

    <!-- 更新提示对话框 -->
    <div 
      v-if="showUpdateDialog"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      @click.self="closeUpdateDialog"
    >
      <div class="card max-w-md mx-4">
        <div class="space-y-4">
          <div class="flex items-start space-x-3">
            <svg class="w-8 h-8 text-hoi4-accent flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path>
            </svg>
            <div class="flex-1">
              <h3 class="text-xl font-bold text-hoi4-text mb-2">发现新版本</h3>
              <p class="text-hoi4-comment mb-1">当前版本: {{ currentVersion }}</p>
              <p class="text-hoi4-accent font-semibold">最新版本: {{ updateInfo?.version }}</p>
            </div>
          </div>
          
          <p class="text-hoi4-text">
            新版本已发布，建议更新以获得最新功能和修复。
          </p>

          <div v-if="updateInfo?.releaseNotes" class="rounded-lg border border-hoi4-border bg-hoi4-gray/70 p-3 max-h-56 overflow-auto">
            <div class="ai-markdown text-sm text-hoi4-text" v-html="updateNotesHtml()"></div>
          </div>
          <div v-else class="rounded-lg border border-hoi4-border bg-hoi4-gray/70 p-3">
            <div class="text-xs text-hoi4-comment">暂无更新内容</div>
          </div>
          
          <div class="flex space-x-3 pt-2">
            <button
              @click="openUpdatePage"
              class="btn-primary flex-1"
            >
              查看更新
            </button>
            <button
              @click="closeUpdateDialog"
              class="btn-secondary flex-1"
            >
              稍后提醒
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 确认提示对话框 -->
    <div 
      v-if="showConfirmDialog"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      @click.self="cancelDisableErrorHandling"
    >
      <div class="card max-w-md mx-4">
        <div class="space-y-4">
          <div class="flex items-start space-x-3">
            <svg class="w-8 h-8 text-hoi4-warning flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"></path>
            </svg>
            <div class="flex-1">
              <h3 class="text-xl font-bold text-hoi4-text mb-2">确认禁用错误处理</h3>
            </div>
          </div>
          
          <p class="text-hoi4-text">
            禁用错误处理功能将关闭所有错误检查和提示，可能导致代码质量问题。确定要继续吗？
          </p>
          
          <div class="flex space-x-3 pt-2">
            <button
              @click="confirmDisableErrorHandling"
              class="btn-primary flex-1"
            >
              确定
            </button>
            <button
              @click="cancelDisableErrorHandling"
              class="btn-secondary flex-1"
            >
              取消
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.ai-markdown :deep(h1) {
  font-size: 1.25rem;
  font-weight: 700;
  margin: 0.5rem 0 0.25rem;
}

.ai-markdown :deep(h2) {
  font-size: 1.15rem;
  font-weight: 700;
  margin: 0.5rem 0 0.25rem;
}

.ai-markdown :deep(h3) {
  font-size: 1.05rem;
  font-weight: 700;
  margin: 0.5rem 0 0.25rem;
}

.ai-markdown :deep(p) {
  margin: 0.25rem 0;
}

.ai-markdown :deep(blockquote) {
  margin: 0.35rem 0;
  padding: 0.35rem 0.6rem;
  border-left: 3px solid rgba(255, 255, 255, 0.18);
  background: rgba(255, 255, 255, 0.04);
  border-radius: 0.5rem;
}

.ai-markdown :deep(a) {
  color: rgba(140, 200, 255, 0.95);
  text-decoration: underline;
}
</style>
