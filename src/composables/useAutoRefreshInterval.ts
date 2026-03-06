import { ref } from 'vue'

export function useAutoRefreshInterval(task: () => void | Promise<void>, intervalMs: number) {
  const intervalId = ref<number | null>(null)
  const enabled = ref(true)

  function start() {
    stop()

    if (!enabled.value) {
      return
    }

    intervalId.value = window.setInterval(() => {
      void task()
    }, intervalMs)
  }

  function stop() {
    if (intervalId.value === null) {
      return
    }

    clearInterval(intervalId.value)
    intervalId.value = null
  }

  return {
    enabled,
    start,
    stop
  }
}
