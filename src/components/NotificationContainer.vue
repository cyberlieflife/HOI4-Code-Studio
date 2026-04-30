<script setup lang="ts">
import { useNotifications, toast } from '../utils/notification'

const notifications = useNotifications()

function typeClass(type: string) {
  switch (type) {
    case 'error': return 'notification-error'
    case 'warning': return 'notification-warning'
    case 'info': return 'notification-info'
    case 'success': return 'notification-success'
    default: return 'notification-info'
  }
}
</script>

<template>
  <div class="notification-container">
    <TransitionGroup name="notification">
      <div
        v-for="n in notifications"
        :key="n.id"
        :class="['notification', typeClass(n.type)]"
        @click="toast.dismiss(n.id)"
      >
        {{ n.message }}
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.notification-container {
  position: fixed;
  top: 16px;
  right: 16px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: 400px;
}

.notification {
  padding: 10px 16px;
  border-radius: 6px;
  color: #fff;
  font-size: 13px;
  line-height: 1.5;
  cursor: pointer;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  word-break: break-word;
}

.notification-error { background: rgba(220, 53, 69, 0.95); }
.notification-warning { background: rgba(255, 193, 7, 0.95); color: #000; }
.notification-info { background: rgba(13, 110, 253, 0.95); }
.notification-success { background: rgba(25, 135, 84, 0.95); }

.notification-enter-active,
.notification-leave-active {
  transition: all 0.3s ease;
}

.notification-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.notification-leave-to {
  opacity: 0;
  transform: translateX(100%);
}
</style>
