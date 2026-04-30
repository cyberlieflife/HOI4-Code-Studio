/**
 * HOI4 国策树文件解析器
 * 解析 focus_tree 结构，构建国策节点关系图谱
 * 支持 relative_position_id 相对定位
 */

export interface FocusNode {
  id: string
  icon?: string
  x: number                    // 网格X坐标
  y: number                    // 网格Y坐标
  cost?: number
  prerequisite?: string[][]    // 前置条件（二维数组：OR关系）
  mutually_exclusive?: string[] // 互斥国策
  relative_position_id?: string // 相对定位基准
  completion_reward?: Record<string, unknown>
  available?: Record<string, unknown>
  modifierText?: string
  completionRewardText?: string
  line: number                 // 源文件行号
  absoluteX?: number           // 计算后的绝对X坐标
  absoluteY?: number           // 计算后的绝对Y坐标
}

export interface FocusTree {
  id: string
  country?: string
  default?: boolean
  focuses: Map<string, FocusNode>
}

/**
 * 查找字符串中指定位置对应的行号
 */
function getLineNumber(content: string, position: number): number {
  const beforeContent = content.substring(0, position)
  return beforeContent.split('\n').length
}

/**
 * 查找块的结束位置
 */
function findBlockEnd(content: string, startPos: number): number {
  let depth = 1
  let i = startPos
  let inLineComment = false
  let inString = false

  while (i < content.length) {
    const char = content[i]

    if (!inString && char === '#') {
      inLineComment = true
      i++
      continue
    }

    if (inLineComment) {
      if (char === '\n') inLineComment = false
      i++
      continue
    }

    if (char === '"') {
      if (i > 0 && content[i - 1] !== '\\') {
        inString = !inString
      }
      i++
      continue
    }

    if (inString) {
      i++
      continue
    }

    if (char === '{') depth++
    else if (char === '}') {
      depth--
      if (depth === 0) return i
    }

    i++
  }

  return -1
}

/**
 * 提取字段值
 */
function extractField(blockContent: string, fieldName: string): string | null {
  const regex = new RegExp(`\\b${fieldName}\\s*=\\s*(["\']?)([^"\\n#}]+)\\1`)
  const match = blockContent.match(regex)
  return match ? match[2].trim() : null
}

/**
 * 提取数字字段
 */
function extractNumber(blockContent: string, fieldName: string): number | undefined {
  const value = extractField(blockContent, fieldName)
  return value !== null ? parseFloat(value) : undefined
}

function extractTopLevelField(blockContent: string, fieldName: string): string | null {
  // 仅在当前 focus 的顶层（花括号深度=1）匹配字段，避免误命中 completion_reward/modifier 等嵌套块
  // 注意：上层 parseFocusTreeFile 已经 removeLineComments，这里无需处理 # 行注释
  const nameRegex = new RegExp(`\\b${fieldName}\\b`)
  const valueRegex = new RegExp(`\\b${fieldName}\\s*=\\s*([+-]?(?:\\d+\\.\\d+|\\d+))`)

  let depth = 0
  let inString = false

  for (let i = 0; i < blockContent.length; i++) {
    const ch = blockContent[i]

    if (ch === '"') {
      if (i === 0 || blockContent[i - 1] !== '\\') {
        inString = !inString
      }
    }
    if (inString) continue

    if (ch === '{') {
      depth++
      continue
    }
    if (ch === '}') {
      depth = Math.max(0, depth - 1)
      continue
    }

    if (depth !== 1) continue

    // 快速过滤：不是字母/下划线起始就跳过
    if (!/[A-Za-z_]/.test(ch)) continue

    // 取当前行片段（减少正则扫描范围）
    const lineEnd = blockContent.indexOf('\n', i)
    const end = lineEnd === -1 ? blockContent.length : lineEnd
    const slice = blockContent.substring(i, end)

    if (!nameRegex.test(slice)) continue
    const m = slice.match(valueRegex)
    if (m) return m[1]
  }

  return null
}

function extractTopLevelNumber(blockContent: string, fieldName: string): number | undefined {
  const v = extractTopLevelField(blockContent, fieldName)
  if (v === null) return undefined
  const n = parseFloat(v)
  return Number.isNaN(n) ? undefined : n
}

function extractBlockText(blockContent: string, fieldName: string): string | undefined {
  const regex = new RegExp(`\\b${fieldName}\\s*=\\s*\\{`, 'g')
  const match = regex.exec(blockContent)
  if (!match) return undefined

  const blockStart = match.index + match[0].length
  const blockEnd = findBlockEnd(blockContent, blockStart)
  if (blockEnd === -1) return undefined

  return blockContent.substring(blockStart, blockEnd).trim()
}

/**
 * 提取 prerequisite（前置条件）
 */
function extractPrerequisites(blockContent: string): string[][] {
  const prerequisites: string[][] = []
  const prereqRegex = /prerequisite\s*=\s*\{/g
  let match: RegExpExecArray | null

  while ((match = prereqRegex.exec(blockContent)) !== null) {
    const blockStart = match.index + match[0].length
    const blockEnd = findBlockEnd(blockContent, blockStart)
    
    if (blockEnd === -1) continue

    const prereqContent = blockContent.substring(blockStart, blockEnd)
    
    // 提取 focus = xxx
    const focusMatches = prereqContent.matchAll(/focus\s*=\s*([a-zA-Z0-9_]+)/g)
    const focuses = Array.from(focusMatches, m => m[1])
    
    if (focuses.length > 0) {
      prerequisites.push(focuses)
    }
  }

  return prerequisites
}

/**
 * 提取互斥国策
 */
function extractMutuallyExclusive(blockContent: string): string[] {
  const exclusive: string[] = []
  const exclusiveRegex = /mutually_exclusive\s*=\s*\{/g
  let match: RegExpExecArray | null

  while ((match = exclusiveRegex.exec(blockContent)) !== null) {
    const blockStart = match.index + match[0].length
    const blockEnd = findBlockEnd(blockContent, blockStart)
    
    if (blockEnd === -1) continue

    const exclusiveContent = blockContent.substring(blockStart, blockEnd)
    
    // 提取 focus = xxx
    const focusMatches = exclusiveContent.matchAll(/focus\s*=\s*([a-zA-Z0-9_]+)/g)
    const focuses = Array.from(focusMatches, m => m[1])
    
    exclusive.push(...focuses)
  }

  return exclusive
}

/**
 * 计算绝对坐标（处理 relative_position_id）
 */
function calculateAbsolutePositions(focuses: Map<string, FocusNode>) {
  const calculated = new Set<string>()
  const visiting = new Set<string>()

  function calculate(focusId: string): { x: number; y: number } {
    const focus = focuses.get(focusId)
    if (!focus) return { x: 0, y: 0 }

    // 如果已经计算过，直接返回
    if (calculated.has(focusId)) {
      return { x: focus.absoluteX ?? focus.x, y: focus.absoluteY ?? focus.y }
    }

    // 检测循环依赖
    if (visiting.has(focusId)) {
      console.warn(`检测到循环依赖: ${focusId}`)
      focus.absoluteX = focus.x
      focus.absoluteY = focus.y
      calculated.add(focusId)
      return { x: focus.x, y: focus.y }
    }

    visiting.add(focusId)

    // 如果有相对定位基准，递归计算
    if (focus.relative_position_id) {
      const basePos = calculate(focus.relative_position_id)
      focus.absoluteX = basePos.x + focus.x
      focus.absoluteY = basePos.y + focus.y
    } else {
      // 没有相对定位，使用原始坐标
      focus.absoluteX = focus.x
      focus.absoluteY = focus.y
    }

    visiting.delete(focusId)
    calculated.add(focusId)

    return { x: focus.absoluteX, y: focus.absoluteY }
  }

  // 计算所有国策的绝对坐标
  focuses.forEach((_focus, focusId) => {
    if (!calculated.has(focusId)) {
      calculate(focusId)
    }
  })
}

/**
 * 移除行注释（# 及之后的内容）
 */
function removeLineComments(content: string): string {
  return content
    .split('\n')
    .map(line => line.split('#', 1)[0]) // 移除 # 及之后的所有内容
    .join('\n')
}

/**
 * 解析国策文件
 */
export function parseFocusTreeFile(content: string): FocusTree | null {
  // 首先移除所有行注释
  const contentWithoutComments = removeLineComments(content)
  
  const focusTreeRegex = /focus_tree\s*=\s*\{/
  const match = contentWithoutComments.match(focusTreeRegex)
  
  if (!match || match.index === undefined) return null

  const matchIndex = match.index
  const treeStart = matchIndex + match[0].length
  const treeEnd = findBlockEnd(contentWithoutComments, treeStart)
  
  if (treeEnd === -1) return null

  const treeContent = contentWithoutComments.substring(matchIndex, treeEnd + 1)
  const treeId = extractField(treeContent, 'id') || 'unknown'

  const focuses = new Map<string, FocusNode>()

  // 解析所有 focus
  const focusRegex = /\bfocus\s*=\s*\{/g
  let focusMatch: RegExpExecArray | null

  while ((focusMatch = focusRegex.exec(treeContent)) !== null) {
    const focusStart = focusMatch.index + focusMatch[0].length
    const focusEnd = findBlockEnd(treeContent, focusStart)
    
    if (focusEnd === -1) continue

    const focusContent = treeContent.substring(focusMatch.index, focusEnd + 1)
    const focusId = extractField(focusContent, 'id')
    
    if (!focusId) {
      focusRegex.lastIndex = focusEnd + 1
      continue
    }

    const node: FocusNode = {
      id: focusId,
      icon: extractField(focusContent, 'icon') || undefined,
      x: extractTopLevelNumber(focusContent, 'x') || 0,
      y: extractTopLevelNumber(focusContent, 'y') || 0,
      cost: extractNumber(focusContent, 'cost'),
      prerequisite: extractPrerequisites(focusContent),
      mutually_exclusive: extractMutuallyExclusive(focusContent),
      relative_position_id: extractField(focusContent, 'relative_position_id') || undefined,
      modifierText: extractBlockText(focusContent, 'modifier'),
      completionRewardText: extractBlockText(focusContent, 'completion_reward'),
      line: getLineNumber(contentWithoutComments, matchIndex + focusMatch.index)
    }

    focuses.set(focusId, node)
    focusRegex.lastIndex = focusEnd + 1
  }

  // 计算绝对坐标
  calculateAbsolutePositions(focuses)

  return {
    id: treeId,
    focuses
  }
}

/**
 * 搜索国策
 * @param focuses 国策Map
 * @param searchQuery 搜索关键词
 * @returns 匹配的国策ID列表
 */
export function searchFocuses(focuses: Map<string, FocusNode>, searchQuery: string): string[] {
  if (!searchQuery) return []
  
  const query = searchQuery.toLowerCase()
  const results: string[] = []

  focuses.forEach((_focus, focusId) => {
    if (focusId.toLowerCase().includes(query)) {
      results.push(focusId)
    }
  })

  return results
}
