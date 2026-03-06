/**
 * 版本管理工具
 */

// 缓存接口
interface VersionCache {
  version: string
  url: string
  releaseNotes?: string
  timestamp: number
}

interface ParsedVersion {
  major: number
  minor: number
  patch: number
  isDev: boolean
  devDate?: number
  revision: number
}

function parseVersion(version: string): ParsedVersion {
  const clean = version.replace(/^v/, '')
  const devMatch = clean.match(/^(\d+(?:\.\d+){0,2})(?:-dev(?:-(\d+))?)?(?:-(\d+))?$/)

  if (!devMatch) {
    return {
      major: 0,
      minor: 0,
      patch: 0,
      isDev: false,
      revision: 0
    }
  }

  const numbers = devMatch[1].split('.').map(Number)
  const [major, minor, patch] = [numbers[0] || 0, numbers[1] || 0, numbers[2] || 0]
  const devDate = devMatch[2] ? Number(devMatch[2]) : undefined
  const revision = devMatch[3] ? Number(devMatch[3]) : 0

  return {
    major,
    minor,
    patch,
    isDev: Boolean(devMatch[0]?.includes('-dev')),
    devDate,
    revision
  }
}

// 缓存时间：1小时（毫秒）
const CACHE_DURATION = 60 * 60 * 1000

// 缓存键
const CACHE_KEY = 'hoi4_version_check_cache'

// 从 localStorage 获取缓存
function getCache(): VersionCache | null {
  try {
    const cached = localStorage.getItem(CACHE_KEY)
    if (!cached) return null
    
    const data: VersionCache = JSON.parse(cached)
    const now = Date.now()
    
    // 检查缓存是否过期
    if (now - data.timestamp > CACHE_DURATION) {
      localStorage.removeItem(CACHE_KEY)
      return null
    }
    
    return data
  } catch (error) {
    console.error('[版本检查] 读取缓存失败:', error)
    return null
  }
}

// 保存到缓存
function setCache(version: string, url: string, releaseNotes?: string): void {
  try {
    const cache: VersionCache = {
      version,
      url,
      releaseNotes,
      timestamp: Date.now()
    }
    localStorage.setItem(CACHE_KEY, JSON.stringify(cache))
    console.log('[版本检查] 缓存已更新，有效期1小时')
  } catch (error) {
    console.error('[版本检查] 保存缓存失败:', error)
  }
}

/**
 * 解析版本字符串为标签格式
 * @param version 版本字符串 (例如: "v0.1.1-dev" 或 "v0.1.1")
 * @returns 标签格式 (例如: "Dev_0_1_1" 或 "v0_1_1")
 */
export function parseVersionToTag(version: string): string {
  // 移除开头的 'v'
  const cleanVersion = version.startsWith('v') ? version.slice(1) : version
  
  // 检查是否包含 -dev 后缀
  if (cleanVersion.endsWith('-dev')) {
    // 移除 -dev 并替换点为下划线
    const versionNumbers = cleanVersion.replace('-dev', '').replace(/\./g, '_')
    return `Dev_${versionNumbers}`
  } else {
    // 替换点为下划线并添加 v 前缀
    const versionNumbers = cleanVersion.replace(/\./g, '_')
    return `v${versionNumbers}`
  }
}

/**
 * 将标签格式转换回版本字符串
 * @param tag 标签格式 (例如: "Dev_0_1_1" 或 "v0_1_1")
 * @returns 版本字符串 (例如: "v0.1.1-dev" 或 "v0.1.1")
 */
export function parseTagToVersion(tag: string): string {
  // 兼容新的正式版与日期 dev 标签
  if (tag.startsWith('OV_')) {
    const body = tag.replace('OV_', '')
    const [versionPart, devPart] = body.split('_dev_')
    const normalizedVersion = versionPart.replace(/_/g, '.').replace(/\./g, '.')
    if (devPart) {
      return `v${normalizedVersion}-dev-${devPart}`
    }
    return `v${normalizedVersion}`
  }

  if (tag.startsWith('dev_')) {
    const devDate = tag.replace('dev_', '')
    return `v0.0.0-dev-${devDate}`
  }

  // 检查是否是 Dev 标签
  if (tag.startsWith('Dev_')) {
    // 移除 Dev_ 前缀，替换下划线为点，添加 v 前缀和 -dev 后缀
    const versionNumbers = tag.replace('Dev_', '').replace(/_/g, '.')
    return `v${versionNumbers}-dev`
  } else if (tag.startsWith('v')) {
    // 移除 v 前缀，替换下划线为点，重新添加 v 前缀
    const versionNumbers = tag.slice(1).replace(/_/g, '.')
    return `v${versionNumbers}`
  } else {
    // 如果格式不符合预期，直接返回原标签
    return tag
  }
}

/**
 * 比较两个版本号
 * @param version1 版本1 (例如: "0.1.1-dev")
 * @param version2 版本2 (例如: "0.2.0")
 * @returns 如果 version2 > version1 返回 true
 */
export function compareVersions(version1: string, version2: string): boolean {
  const parsed1 = parseVersion(version1)
  const parsed2 = parseVersion(version2)
  
  // 比较主版本、次版本、补丁版本
  const parts1 = [parsed1.major, parsed1.minor, parsed1.patch]
  const parts2 = [parsed2.major, parsed2.minor, parsed2.patch]

  for (let i = 0; i < parts1.length; i++) {
    const num1 = parts1[i]
    const num2 = parts2[i]

    if (num2 > num1) return true
    if (num2 < num1) return false
  }
  
  // 如果版本号相同，检查是否有 -dev 后缀
  // 正式版本 > dev 版本
  const isDev1 = parsed1.isDev
  const isDev2 = parsed2.isDev
  
  if (!isDev2 && isDev1) return true

  if (isDev1 && isDev2) {
    const devDate1 = parsed1.devDate ?? 0
    const devDate2 = parsed2.devDate ?? 0
    if (devDate2 > devDate1) return true
    if (devDate2 < devDate1) return false
    const rev1 = parsed1.revision ?? 0
    const rev2 = parsed2.revision ?? 0
    return rev2 > rev1
  }

  if (!isDev1 && !isDev2) {
    const rev1 = parsed1.revision ?? 0
    const rev2 = parsed2.revision ?? 0
    return rev2 > rev1
  }
  
  return false
}

/**
 * 从 GitHub 获取最新版本信息
 * @param currentVersion 当前版本
 * @param githubToken 可选的 GitHub Token
 * @param useCache 是否使用缓存，默认 true
 * @returns 如果有新版本返回版本信息，否则返回 null
 */
export async function checkForUpdates(
  currentVersion: string, 
  githubToken?: string,
  useCache: boolean = true
): Promise<{
  hasUpdate: boolean
  latestVersion?: string
  releaseUrl?: string
  releaseNotes?: string
  error?: string
}> {
  try {
    console.log('[版本检查] 开始检查更新...')
    console.log('[版本检查] 当前版本:', currentVersion)
    
    // 检查缓存
    if (useCache) {
      const cached = getCache()
      if (cached) {
        console.log('[版本检查] ✅ 使用缓存数据:', cached.version)
        const hasUpdate = compareVersions(currentVersion, cached.version)
        const cachedNotes = typeof cached.releaseNotes === 'string' ? cached.releaseNotes : ''
        if (!hasUpdate || cachedNotes.trim().length > 0) {
          return {
            hasUpdate,
            latestVersion: cached.version,
            releaseUrl: cached.url,
            releaseNotes: cached.releaseNotes
          }
        }
        console.log('[版本检查] 缓存缺少 releaseNotes，回退到网络请求补齐')
      }
      console.log('[版本检查] 缓存未命中或已过期，发起网络请求')
    }
    
    console.log('[版本检查] 请求 URL: https://api.github.com/repos/cyberlieflife/HOI4-Code-Studio/releases/latest')
    console.log('[版本检查] 使用 Token:', githubToken ? '✅ 是（5000次/小时）' : '❌ 否（60次/小时）')
    
    const headers: HeadersInit = {
      'Accept': 'application/vnd.github.v3+json'
    }
    
    // 如果有 Token，添加认证头
    if (githubToken && githubToken.trim()) {
      headers['Authorization'] = `token ${githubToken.trim()}`
    }
    
    const response = await fetch('https://api.github.com/repos/cyberlieflife/HOI4-Code-Studio/releases/latest', {
      headers
    })
    
    console.log('[版本检查] HTTP 状态码:', response.status)
    console.log('[版本检查] 响应状态:', response.statusText)
    console.log('[版本检查] 响应头:', Object.fromEntries(response.headers.entries()))
    
    if (!response.ok) {
      // 尝试获取错误响应内容
      let errorDetail = ''
      try {
        const errorData = await response.json()
        errorDetail = JSON.stringify(errorData, null, 2)
        console.error('[版本检查] 错误响应内容:', errorDetail)
      } catch (e) {
        const errorText = await response.text()
        errorDetail = errorText
        console.error('[版本检查] 错误响应文本:', errorText)
      }
      
      throw new Error(`GitHub API 请求失败: ${response.status} ${response.statusText}\n详情: ${errorDetail}`)
    }
    
    const data = await response.json()
    console.log('[版本检查] 成功获取版本信息:', data)
    
    const tagName = data.tag_name // 例如: "v0_2_0" 或 "Dev_0_1_1"
    const releaseUrl = data.html_url
    const releaseNotes = typeof data.body === 'string' ? data.body : ''
    
    // 将标签格式转换为版本格式
    const latestVersion = parseTagToVersion(tagName)
    console.log('[版本检查] 最新版本:', latestVersion)
    
    // 保存到缓存
    if (useCache) {
      setCache(latestVersion, releaseUrl, releaseNotes)
    }
    
    // 比较版本
    const hasUpdate = compareVersions(currentVersion, latestVersion)
    console.log('[版本检查] 是否有更新:', hasUpdate)
    
    return {
      hasUpdate,
      latestVersion,
      releaseUrl,
      releaseNotes
    }
  } catch (error) {
    console.error('[版本检查] ❌ 检查更新失败:')
    console.error('[版本检查] 错误类型:', error?.constructor?.name)
    console.error('[版本检查] 错误消息:', error instanceof Error ? error.message : String(error))
    console.error('[版本检查] 完整错误:', error)
    
    return {
      hasUpdate: false,
      error: error instanceof Error ? error.message : '未知错误'
    }
  }
}
