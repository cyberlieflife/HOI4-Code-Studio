/**
 * 统一错误/通知服务
 * 替代直接调用 alert()，提供非阻塞式通知
 */

import { ref, readonly } from 'vue'

export type NotificationType = 'error' | 'warning' | 'info' | 'success'

export interface Notification {
  id: number
  type: NotificationType
  message: string
  timeout: number
}

const notifications = ref<Notification[]>([])
let nextId = 0

/** 获取通知列表（只读） */
export const useNotifications = () => readonly(notifications)

/** 添加通知 */
function notify(type: NotificationType, message: string, timeout = 5000) {
  const id = nextId++
  const notification: Notification = { id, type, message, timeout }
  notifications.value.push(notification)

  if (timeout > 0) {
    setTimeout(() => dismiss(id), timeout)
  }
}

/** 移除通知 */
function dismiss(id: number) {
  const idx = notifications.value.findIndex(n => n.id === id)
  if (idx !== -1) {
    notifications.value.splice(idx, 1)
  }
}

/** 清空所有通知 */
function clearAll() {
  notifications.value = []
}

/** 便捷方法 */
export const toast = {
  error: (msg: string, timeout?: number) => notify('error', msg, timeout),
  warning: (msg: string, timeout?: number) => notify('warning', msg, timeout),
  info: (msg: string, timeout?: number) => notify('info', msg, timeout),
  success: (msg: string, timeout?: number) => notify('success', msg, timeout),
  dismiss,
  clearAll
}
