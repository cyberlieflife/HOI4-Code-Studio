import { logger } from './logger'

const isDev = import.meta.env.DEV

function canMeasure() {
  return isDev && typeof performance !== 'undefined'
}

export async function measureMapAsync<T>(label: string, task: () => Promise<T>): Promise<T> {
  if (!canMeasure()) {
    return await task()
  }

  const start = performance.now()
  try {
    return await task()
  } finally {
    logger.info(`[map] ${label}: ${(performance.now() - start).toFixed(2)}ms`)
  }
}

export function measureMapSync<T>(label: string, task: () => T): T {
  if (!canMeasure()) {
    return task()
  }

  const start = performance.now()
  try {
    return task()
  } finally {
    logger.info(`[map] ${label}: ${(performance.now() - start).toFixed(2)}ms`)
  }
}

export function logMapEvent(label: string, payload?: Record<string, unknown>) {
  if (!isDev) return
  if (payload) {
    logger.info(`[map] ${label}`, payload)
    return
  }
  logger.info(`[map] ${label}`)
}
