<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'

// HOI4 Code Studio - 主应用组件

// 禁用浏览器默认右键菜单
function handleContextMenu(event: MouseEvent) {
  event.preventDefault()
  return false
}

onMounted(() => {
  // 延迟加载主题系统，避免把主题模块放进启动首包
  setTimeout(async () => {
    try {
      const { useTheme } = await import('./composables/useTheme')
      const { loadThemeFromSettings } = useTheme()
      await loadThemeFromSettings()
    } catch (error) {
      console.error('加载主题设置失败:', error)
    }
  }, 100)

  // 添加全局右键菜单禁用
  document.addEventListener('contextmenu', handleContextMenu)
})

onUnmounted(() => {
  // 清理事件监听
  document.removeEventListener('contextmenu', handleContextMenu)
})
</script>

<template>
  <div id="app" class="h-screen w-screen overflow-hidden">
    <Transition name="page-transition" mode="out-in">
      <router-view />
    </Transition>
  </div>
</template>

<style>
/* 页面过渡动画 */
.page-transition-enter-active,
.page-transition-leave-active {
  transition: all 0.3s ease;
}

.page-transition-enter-from {
  opacity: 0;
  transform: translateY(20px);
}

.page-transition-leave-to {
  opacity: 0;
  transform: translateY(-20px);
}
</style>
