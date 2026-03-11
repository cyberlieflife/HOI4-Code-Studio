import { logger } from './logger'

const isDev = import.meta.env.DEV
const START_MARK = 'startup:app-start'

type StartupMark =
  | 'startup:app-start'
  | 'startup:app-mounted'
  | 'startup:router-ready'
  | 'startup:home-mounted'
  | 'startup:home-recent-projects-loaded'
  | 'startup:home-ready'
  | 'startup:editor-mounted'
  | 'startup:editor-theme-loaded'
  | 'startup:editor-icons-loaded'
  | 'startup:editor-settings-loaded'
  | 'startup:editor-project-info-loaded'
  | 'startup:editor-file-tree-loaded'
  | 'startup:editor-game-directory-loaded'
  | 'startup:editor-dependencies-loaded'
  | 'startup:editor-tags-loaded'
  | 'startup:editor-ideas-loaded'
  | 'startup:editor-ready'

function canMeasure() {
  return isDev && typeof window !== 'undefined' && typeof performance !== 'undefined'
}

export function markStartup(mark: StartupMark) {
  if (!canMeasure()) return
  performance.mark(mark)
}

export function measureStartup(name: string, startMark: StartupMark, endMark: StartupMark) {
  if (!canMeasure()) return

  try {
    const measure = performance.measure(name, startMark, endMark)
    logger.info(`[startup] ${name}: ${measure.duration.toFixed(2)}ms`)
  } catch (error) {
    logger.warn(`[startup] 无法生成性能记录: ${name}`, error)
  }
}

export function markStartupStep(mark: StartupMark, label: string) {
  markStartup(mark)
  measureStartup(label, START_MARK, mark)
}
