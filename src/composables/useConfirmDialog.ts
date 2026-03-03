/**
 * 确认对话框 Composable
 * 管理确认对话框的显示状态和交互逻辑
 */

import { ref } from 'vue'

export type ConfirmDialogType = 'warning' | 'danger' | 'info'

// 全局状态（单例模式）
const confirmDialogVisible = ref(false)
const confirmDialogTitle = ref('')
const confirmDialogMessage = ref('')
const confirmDialogType = ref<ConfirmDialogType>('warning')
let confirmDialogResolve: ((value: boolean) => void) | null = null

/**
 * 确认对话框 Composable
 * 使用单例模式，确保全局只有一个确认对话框实例
 */
export function useConfirmDialog() {
  /**
   * 显示确认对话框
   * @param message 确认消息
   * @param title 对话框标题
   * @param type 对话框类型
   * @returns Promise<boolean> 用户是否确认
   */
  function showConfirmDialog(
    message: string,
    title = '⚠️ 确认操作',
    type: ConfirmDialogType = 'warning'
  ): Promise<boolean> {
    return new Promise((resolve) => {
      confirmDialogMessage.value = message
      confirmDialogTitle.value = title
      confirmDialogType.value = type
      confirmDialogVisible.value = true
      confirmDialogResolve = resolve
    })
  }

  /**
   * 处理确认按钮点击
   */
  function handleConfirmDialogConfirm() {
    confirmDialogVisible.value = false
    if (confirmDialogResolve) {
      confirmDialogResolve(true)
      confirmDialogResolve = null
    }
  }

  /**
   * 处理取消按钮点击
   */
  function handleConfirmDialogCancel() {
    confirmDialogVisible.value = false
    if (confirmDialogResolve) {
      confirmDialogResolve(false)
      confirmDialogResolve = null
    }
  }

  return {
    // 状态
    confirmDialogVisible,
    confirmDialogTitle,
    confirmDialogMessage,
    confirmDialogType,
    // 方法
    showConfirmDialog,
    handleConfirmDialogConfirm,
    handleConfirmDialogCancel
  }
}
