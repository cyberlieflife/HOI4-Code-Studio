import { invoke } from '@tauri-apps/api/core'
import { openUrl as tauriOpenUrl } from '@tauri-apps/plugin-opener'

// ==================== 类型定义 ====================

export interface CreateProjectResult {
  success: boolean
  message: string
  project_path?: string
}

/**
 * 项目数据接口
 */
export interface ProjectData {
  name: string
  version: string
  path: string
  [key: string]: unknown
}

export interface OpenProjectResult {
  success: boolean
  message: string
  project_data?: ProjectData
}

export interface RecentProject {
  name: string
  path: string
  last_opened: string
}

export type TerminalShell = 'cmd' | 'powershell'

export interface TerminalSessionInfo {
  sessionId: string
  shell: TerminalShell
  cwd: string
}

export interface ProjectStats {
  path: string
  fileCount: number
  totalSize: number
  version?: string
}

export interface RecentProjectStatsResult {
  success: boolean
  stats: ProjectStats[]
}

export interface RecentProjectsResult {
  success: boolean
  projects: RecentProject[]
}

export interface FileDialogResult {
  success: boolean
  path?: string
}

export interface JsonResult<T = unknown> {
  success: boolean
  message: string
  data?: T
}

export interface JsonValidationResult {
  valid: boolean
  errors: string[]
}

export interface ThemeColors {
  bg: string
  bgSecondary: string
  fg: string
  comment: string
  border: string
  selection: string
  accent: string
  success: string
  warning: string
  error: string
  keyword: string
}

export interface Theme {
  id: string
  name: string
  colors: ThemeColors
}

export interface PluginShortcut {
  id: string
  keys?: string[]
  description?: string
  action?: string
}

export interface PluginSnippet {
  id: string
  title: string
  description?: string
  content: string
  pathIncludes?: string[]
}

export interface PluginIconSetFolder {
  closed: string
  open: string
}

export interface PluginIconSetIcons {
  folder: PluginIconSetFolder
  files: Record<string, string>
}

export interface PluginIconSet {
  id: string
  name: string
  description?: string
  type: 'emoji' | 'svg'
  icons: PluginIconSetIcons
}

export interface PluginInstallHooks {
  themes?: Theme[]
  settings?: Record<string, unknown>
  shortcuts?: PluginShortcut[]
  snippets?: PluginSnippet[]
  iconSets?: PluginIconSet[]
  editorSettings?: Record<string, unknown>
}

export interface PluginPermissions {
  commands: string[]
}

export interface PluginPanelContribution {
  id: string
  title: string
}

export interface PluginToolbarOpenTarget {
  side: string
  panel: string
}

export interface PluginToolbarContribution {
  id: string
  title: string
  open?: PluginToolbarOpenTarget
}

export interface PluginContributes {
  left_sidebar: PluginPanelContribution[]
  right_sidebar: PluginPanelContribution[]
  toolbar: PluginToolbarContribution[]
}

export interface PluginAbout {
  id: string
  name: string
  version: string
  description?: string
  author?: string
  main?: string
  contributes: PluginContributes
  permissions: PluginPermissions
  install?: PluginInstallHooks
}

export interface InstalledPlugin {
  about: PluginAbout
  install_path: string
  entry_file_path: string
}

export interface PluginValidateResult {
  ok: boolean
  errors: string[]
  warnings: string[]
  about?: PluginAbout
  entry_file_path?: string
}

/**
 * 构建插件安装钩子（提供空数组默认值，便于 JS/TS 调用）
 */
export function buildPluginInstallHooks(hooks: PluginInstallHooks): PluginInstallHooks {
  return {
    themes: hooks.themes ?? [],
    settings: hooks.settings,
    shortcuts: hooks.shortcuts ?? [],
    snippets: hooks.snippets ?? [],
    iconSets: hooks.iconSets ?? [],
    editorSettings: hooks.editorSettings
  }
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function mergeJsonObject(target: Record<string, unknown>, src: Record<string, unknown>) {
  Object.entries(src).forEach(([key, value]) => {
    const existing = target[key]
    if (isPlainObject(existing) && isPlainObject(value)) {
      mergeJsonObject(existing, value)
    } else {
      target[key] = value
    }
  })
}

function mergeById<T extends { id: string }>(existing: T[], incoming: T[]): T[] {
  const out: T[] = []
  existing.forEach(item => {
    if (item.id?.trim()) out.push(item)
  })
  incoming.forEach(item => {
    if (!item.id?.trim()) return
    const index = out.findIndex(x => x.id === item.id)
    if (index >= 0) out[index] = item
    else out.push(item)
  })
  return out
}

/**
 * JS/TS 侧应用 install hooks（合并并写入 settings.json）
 */
export async function applyPluginInstallHooks(hooks: PluginInstallHooks): Promise<JsonResult> {
  const install = buildPluginInstallHooks(hooks)
  const result = await loadSettings()
  if (!result.success || !result.data) {
    return result
  }

  const settings = (result.data as Record<string, unknown>) || {}

  const shortcuts: PluginShortcut[] = install.shortcuts ?? []
  const snippets: PluginSnippet[] = install.snippets ?? []
  const iconSets: PluginIconSet[] = install.iconSets ?? []

  if (install.settings && isPlainObject(install.settings)) {
    mergeJsonObject(settings, install.settings)
  }

  if (shortcuts.length > 0) {
    const existing = Array.isArray(settings.shortcuts) ? (settings.shortcuts as PluginShortcut[]) : []
    settings.shortcuts = mergeById(existing, shortcuts)
  }

  if (snippets.length > 0) {
    const existing = Array.isArray(settings.snippets) ? (settings.snippets as PluginSnippet[]) : []
    settings.snippets = mergeById(existing, snippets)
  }

  if (iconSets.length > 0) {
    const existing = Array.isArray(settings.iconSets) ? (settings.iconSets as PluginIconSet[]) : []
    settings.iconSets = mergeById(existing, iconSets)
  }

  if (install.editorSettings && isPlainObject(install.editorSettings)) {
    if (isPlainObject(settings.editorSettings)) {
      mergeJsonObject(settings.editorSettings, install.editorSettings)
    } else {
      settings.editorSettings = install.editorSettings
    }
  }

  return await saveSettings(settings)
}

/**
 * 检查插件是否已执行过 install hooks（基于 settings.pluginInstallFlags）
 */
export async function hasPluginInstalled(pluginId: string): Promise<boolean> {
  const result = await loadSettings()
  if (!result.success || !result.data) {
    return false
  }
  const settings = result.data as Record<string, unknown>
  const flags = (settings.pluginInstallFlags as Record<string, boolean>) || {}
  return flags[pluginId] === true
}

/**
 * 标记插件已执行 install hooks（基于 settings.pluginInstallFlags）
 */
export async function markPluginInstalled(pluginId: string, installed: boolean = true): Promise<JsonResult> {
  const result = await loadSettings()
  if (!result.success || !result.data) {
    return result
  }
  const settings = (result.data as Record<string, unknown>) || {}
  const flags = (settings.pluginInstallFlags as Record<string, boolean>) || {}
  flags[pluginId] = installed
  settings.pluginInstallFlags = flags
  return await saveSettings(settings)
}

/**
 * 仅执行一次 install hooks（若已标记则跳过）
 */
export async function applyPluginInstallHooksOnce(pluginId: string, hooks: PluginInstallHooks): Promise<JsonResult> {
  const alreadyInstalled = await hasPluginInstalled(pluginId)
  if (alreadyInstalled) {
    return { success: true, message: 'already installed', data: null }
  }
  const result = await applyPluginInstallHooks(hooks)
  if (result.success) {
    await markPluginInstalled(pluginId, true)
  }
  return result
}

// ==================== 项目管理 ====================

/**
 * 创建新项目
 */
export async function createNewProject(
  projectName: string,
  version: string,
  projectPath: string,
  replacePath: string[]
): Promise<CreateProjectResult> {
  return await invoke('create_new_project', {
    projectName,
    version,
    projectPath,
    replacePath
  })
}

/**
 * 打开现有项目
 */
export async function openProject(projectPath: string): Promise<OpenProjectResult> {
  return await invoke('open_project', { projectPath })
}

/**
 * 初始化项目
 */
export async function initializeProject(projectPath: string): Promise<OpenProjectResult> {
  return await invoke('initialize_project', { projectPath })
}

/**
 * 获取最近打开的项目列表
 */
export async function getRecentProjects(): Promise<RecentProjectsResult> {
  return await invoke('get_recent_projects')
}

export async function getRecentProjectStats(paths: string[]): Promise<RecentProjectStatsResult> {
  return await invoke('get_recent_project_stats_cached', { paths })
}

/**
 * 打开文件选择对话框
 */
export async function openFileDialog(mode: 'directory' | 'file'): Promise<FileDialogResult> {
  return await invoke('open_file_dialog', { mode })
}

/**
 * 读取目录内容
 */
/**
 * 目录条目接口
 */
export interface DirectoryEntry {
  name: string
  path: string
  is_directory: boolean
  size?: number
}

export interface DirectoryResult {
  success: boolean
  message: string
  entries?: DirectoryEntry[]
}

export async function readDirectory(dirPath: string): Promise<DirectoryResult> {
  return await invoke('read_directory', { dirPath })
}

/**
 * 创建文件
 */
export interface FileOperationResult {
  success: boolean
  message: string
}

export async function createFile(filePath: string, content: string, useBom: boolean = false): Promise<FileOperationResult> {
  return await invoke('create_file', { filePath, content, useBom })
}

/**
 * 创建文件夹
 */
export async function createFolder(folderPath: string): Promise<FileOperationResult> {
  return await invoke('create_folder', { folderPath })
}

export async function copyPaths(sourcePaths: string[], targetDir: string): Promise<FileOperationResult> {
  return await invoke('copy_paths', { sourcePaths, targetDir })
}

export async function movePaths(sourcePaths: string[], targetDir: string): Promise<FileOperationResult> {
  return await invoke('move_paths', { sourcePaths, targetDir })
}

/**
 * 重命名文件或文件夹
 */
export async function renamePath(oldPath: string, newPath: string): Promise<FileOperationResult> {
  return await invoke('rename_path', { oldPath, newPath })
}

/**
 * 删除文件或文件夹
 */
export async function deletePath(targetPath: string): Promise<FileOperationResult> {
  return await invoke('delete_path', { targetPath })
}

/**
 * 打开文件夹
 */
export async function openFolder(path: string): Promise<FileOperationResult> {
  return await invoke('open_folder', { path })
}

/**
 * 读取文件内容
 */
export interface FileContentResult {
  success: boolean
  message: string
  content: string
  encoding?: string
  is_binary?: boolean
  is_image?: boolean
}

export async function readFileContent(filePath: string): Promise<FileContentResult> {
  return await invoke('read_file_content', { filePath })
}

/**
 * 写入文件内容
 */
export async function writeFileContent(filePath: string, content: string): Promise<FileOperationResult> {
  return await invoke('write_file_content', { filePath, content })
}

/**
 * 获取 Modifier 列表内容
 */
export async function getModifierList(): Promise<JsonResult<string>> {
  return await invoke('get_modifier_list')
}

export async function installPlugin(sourcePath: string): Promise<InstalledPlugin> {
  return await invoke('install_plugin', { sourcePath })
}

export async function uninstallPlugin(pluginId: string): Promise<void> {
  await invoke('uninstall_plugin', { pluginId })
}

export async function listInstalledPlugins(): Promise<InstalledPlugin[]> {
  return await invoke('list_installed_plugins')
}

export async function validatePluginPackage(sourcePath: string): Promise<PluginValidateResult> {
  return await invoke('validate_plugin_package', { sourcePath })
}

export async function listThemes(): Promise<Theme[]> {
  return await invoke('list_themes')
}

export async function upsertTheme(theme: Theme): Promise<Theme[]> {
  return await invoke('upsert_theme', { theme })
}

export async function deleteTheme(themeId: string): Promise<Theme[]> {
  return await invoke('delete_theme', { themeId })
}

// ==================== 设置管理 ====================

/**
 * 加载设置
 */
export async function loadSettings(): Promise<JsonResult> {
  return await invoke('load_settings')
}

/**
 * 保存设置
 */
export interface Settings {
  gameDirectory?: string
  autoSave?: boolean
  disableErrorHandling?: boolean
  theme?: string
  iconSet?: string
  checkForUpdates?: boolean
  lastProjectPath?: string
  [key: string]: unknown
}

export async function saveSettings(settings: Settings): Promise<JsonResult> {
  return await invoke('save_settings', { settings })
}

export async function loadSettingsSnapshot(): Promise<Settings> {
  const result = await loadSettings()
  if (!result.success || !result.data || typeof result.data !== 'object') {
    return {}
  }
  return result.data as Settings
}

/**
 * 获取默认缓存目录
 */
export async function getDefaultCacheDirectory(): Promise<JsonResult> {
  return await invoke('get_default_cache_directory')
}

/**
 * 迁移缓存目录
 */
export async function migrateCacheDirectory(newCacheDir: string): Promise<JsonResult> {
  return await invoke('migrate_cache_directory', { newCacheDir })
}

/**
 * 验证游戏目录
 */
export async function validateGameDirectory(path: string): Promise<{ valid: boolean; message: string }> {
  return await invoke('validate_game_directory', { path })
}

// ==================== JSON 操作 ====================

/**
 * 解析 JSON 字符串
 */
export async function parseJson(jsonStr: string): Promise<JsonResult> {
  return await invoke('parse_json', { jsonStr })
}

/**
 * 序列化 JSON 对象
 */
export async function stringifyJson(value: unknown, pretty: boolean = true): Promise<JsonResult<string>> {
  return await invoke('stringify_json', { value, pretty })
}

/**
 * 验证 JSON 格式
 */
export async function validateJson(jsonStr: string): Promise<JsonValidationResult> {
  return await invoke('validate_json', { jsonStr })
}

/**
 * 合并 JSON 对象
 */
export async function mergeJson(base: unknown, overlay: unknown, deep: boolean = false): Promise<JsonResult> {
  return await invoke('merge_json', { base, overlay, deep })
}

/**
 * 获取 JSON 路径值
 */
export async function getJsonPath(value: unknown, path: string): Promise<JsonResult> {
  return await invoke('get_json_path', { value, path })
}

/**
 * 设置 JSON 路径值
 */
export async function setJsonPath(value: unknown, path: string, newValue: unknown): Promise<JsonResult> {
  return await invoke('set_json_path', { value, path, newValue })
}

/**
 * 读取 JSON 文件
 */
export async function readJsonFile(filePath: string): Promise<JsonResult> {
  return await invoke('read_json_file', { filePath })
}

/**
 * 写入 JSON 文件
 */
export async function writeJsonFile(filePath: string, value: unknown, pretty: boolean = true): Promise<JsonResult> {
  return await invoke('write_json_file', { filePath, value, pretty })
}

// ==================== 其他 ====================

/**
 * 退出应用程序
 */
export async function exitApplication(): Promise<void> {
  await invoke('exit_application')
}

/**
 * 打开设置页面
 */
export async function openSettings(): Promise<FileOperationResult> {
  return await invoke('open_settings')
}

// ==================== 搜索功能 ====================

export interface SearchResult {
  file_name: string
  file_path: string
  line: number
  content: string
  match_start: number
  match_end: number
}

export interface SearchResponse {
  success: boolean
  message: string
  results: SearchResult[]
}

export async function searchFiles(
  directory: string,
  query: string,
  case_sensitive: boolean,
  use_regex: boolean,
  include_all_files: boolean = false
): Promise<SearchResponse> {
  return await invoke('search_files', {
    directoryPath: directory,
    query,
    caseSensitive: case_sensitive,
    useRegex: use_regex,
    includeAllFiles: include_all_files
  })
}

// ==================== 文件树构建 ====================

export interface FileNode {
  name: string
  path: string
  is_directory: boolean
  children?: FileNode[]
  size?: number
  expanded: boolean
}

export interface FileTreeResult {
  success: boolean
  message: string
  tree?: FileNode[]
}

export async function buildDirectoryTree(
  path: string,
  maxDepth: number = 0
): Promise<FileTreeResult> {
  return await invoke('build_directory_tree', { path, maxDepth })
}

export async function buildDirectoryTreeFast(
  path: string,
  maxDepth: number = 0
): Promise<FileTreeResult> {
  return await invoke('build_directory_tree_fast', { path, maxDepth })
}

// ==================== Focus Localizations ====================

export interface FocusLocalizationLoadResponse {
  success: boolean
  message: string
  map?: Record<string, string>
}

export async function loadFocusLocalizations(roots: string[]): Promise<FocusLocalizationLoadResponse> {
  return invoke<FocusLocalizationLoadResponse>('load_focus_localizations', { roots })
}

// ==================== Country Tags ====================

export interface TagEntry {
  code: string
  name?: string
  source: 'project' | 'game' | 'dependency'
}

export interface TagLoadResponse {
  success: boolean
  message: string
  tags?: TagEntry[]
}

export async function loadCountryTags(
  projectRoot?: string,
  gameRoot?: string,
  dependencyRoots?: string[]
): Promise<TagLoadResponse> {
  return invoke<TagLoadResponse>('load_country_tags', { projectRoot, gameRoot, dependencyRoots })
}

export interface TagValidationError {
  line: number
  message: string
}

export interface TagValidationResponse {
  success: boolean
  message: string
  errors: TagValidationError[]
}

export async function validateTags(
  content: string,
  projectRoot?: string,
  gameRoot?: string,
  dependencyRoots?: string[]
): Promise<TagValidationResponse> {
  return invoke<TagValidationResponse>('validate_tags', { content, projectRoot, gameRoot, dependencyRoots })
}

// ==================== idea ====================

export interface IdeaEntry {
  id: string
  source: 'project' | 'game' | 'dependency'
}

export interface IdeaLoadResponse {
  success: boolean
  message: string
  ideas?: IdeaEntry[]
}

export async function loadIdeas(
  projectRoot?: string,
  gameRoot?: string,
  dependencyRoots?: string[]
): Promise<IdeaLoadResponse> {
  return invoke<IdeaLoadResponse>('load_ideas', { projectRoot, gameRoot, dependencyRoots })
}

export async function resetIdeaCache(): Promise<boolean> {
  return await invoke('reset_idea_cache')
}
// ==================== 括号匹配 ====================

export enum BracketType {
  Round = 'Round',
  Square = 'Square',
  Curly = 'Curly'
}

export interface BracketInfo {
  bracket_type: BracketType
  start: number
  end: number
  depth: number
  matched: boolean
}

export interface BracketMatchResult {
  success: boolean
  message: string
  brackets: BracketInfo[]
  unmatched: number[]
}

export async function matchBrackets(content: string): Promise<BracketMatchResult> {
  return await invoke('match_brackets', { content })
}

export async function findBracketPair(
  content: string,
  cursorPos: number
): Promise<number | null> {
  return await invoke('find_bracket_pair', { content, cursorPos })
}

export async function getBracketDepths(content: string): Promise<number[]> {
  return await invoke('get_bracket_depths', { content })
}

// ==================== 外部链接 ====================

/**
 * 在默认浏览器中打开URL
 */
export async function openUrl(url: string): Promise<void> {
  await tauriOpenUrl(url)
}

// ==================== 游戏启动 ====================

/**
 * 启动游戏结果
 */
export interface LaunchGameResult {
  success: boolean
  message: string
}

/**
 * 启动游戏
 */
export async function launchGame(): Promise<LaunchGameResult> {
  return await invoke('launch_game')
}

// ==================== 依赖项管理 ====================

/**
 * 依赖项类型
 */
export type DependencyType = 'hoics' | 'hoi4mod'

/**
 * 依赖项接口
 */
export interface Dependency {
  id: string
  name: string
  path: string
  type: DependencyType
  addedAt: string
  enabled: boolean
}

/**
 * 依赖项验证结果
 */
export interface DependencyValidation {
  valid: boolean
  message: string
  name?: string
  type?: DependencyType
}

/**
 * 依赖项加载结果
 */
export interface DependencyLoadResult {
  success: boolean
  message: string
  dependencies?: Dependency[]
}

/**
 * 依赖项保存结果
 */
export interface DependencySaveResult {
  success: boolean
  message: string
}

/**
 * 依赖项索引结果
 */
export interface DependencyIndexResult {
  success: boolean
  message: string
  ideaCount?: number
  tagCount?: number
}

/**
 * 加载项目的依赖项列表
 */
export async function loadDependencies(projectPath: string): Promise<DependencyLoadResult> {
  return await invoke('load_dependencies', { projectPath })
}

/**
 * 保存项目的依赖项列表
 */
export async function saveDependencies(
  projectPath: string,
  dependencies: Dependency[]
): Promise<DependencySaveResult> {
  return await invoke('save_dependencies', { projectPath, dependencies })
}

/**
 * 验证依赖项路径
 */
export async function validateDependencyPath(path: string): Promise<DependencyValidation> {
  return await invoke('validate_dependency_path', { path })
}

/**
 * 索引依赖项的 Idea 和 Tag 数据
 */
export async function indexDependency(dependencyPath: string): Promise<DependencyIndexResult> {
  return await invoke('index_dependency', { dependencyPath })
}

// ==================== 项目打包 ====================

/**
 * 打包选项
 */
export interface PackageOptions {
  projectPath: string
  outputName: string
  excludeDependencies: boolean
}

/**
 * 打包结果
 */
export interface PackageResult {
  success: boolean
  message: string
  outputPath?: string
  fileSize?: number
}

/**
 * 打包项目
 */
export async function packProject(options: PackageOptions): Promise<PackageResult> {
  return await invoke('pack_project', {
    opts: {
      projectPath: options.projectPath,
      outputName: options.outputName,
      excludeDependencies: options.excludeDependencies
    }
  })
}

// ==================== 图片读取 ====================

/**
 * 图片读取结果
 */
export interface ImageReadResult {
  success: boolean
  message?: string
  base64?: string
  mimeType?: string
  mime_type?: string
}

function normalizeImageReadResult(result: any): ImageReadResult {
  if (!result || typeof result !== 'object') return { success: false, message: 'invalid response' }
  return {
    success: !!result.success,
    message: result.message,
    base64: result.base64,
    mimeType: result.mimeType ?? result.mime_type,
    mime_type: result.mime_type ?? result.mimeType
  }
}

/**
 * 读取图片文件为 base64
 */
export async function readImageAsBase64(filePath: string): Promise<ImageReadResult> {
  const resp = await invoke('read_image_as_base64', { filePath })
  return normalizeImageReadResult(resp)
}

// ==================== GUI 引擎 ====================

export interface GuiNode {
  node_type: 'container_window' | 'icon' | 'button' | 'instant_text_box' | 'grid_box' | 'window' | 'window_type'
  properties: {
    name?: string
    position?: { x: number; y: number }
    size?: { width: number; height: number }
    orientation?: string
    origo?: string
    sprite_type?: string
    quad_texture_sprite?: string
    background?: string
    font?: string
    text?: string
    format?: string
    vertical_alignment?: string
    max_width?: number
    max_height?: number
    scale?: number
    frame?: number
    clipping?: boolean
    fixedsize?: boolean
    slotsize?: { width: number; height: number }
    add_horizontal?: boolean
    max_slots_horizontal?: number
    max_slots_vertical?: number
  }
  children: GuiNode[]
}

export interface GuiParseResult {
  success: boolean
  path: string
  windows: GuiNode[]
}

export interface GfxParseResult {
  success: boolean
  sprites: Record<string, {
    texturefile: string
    noOfFrames: number
    borderSize?: { x: number; y: number }
  }>
}

export interface GfxSpritePreviewItem {
  name: string
  texturefile?: string | null
  noOfFrames: number
  borderSize?: any
  sourceLine: number
  resolvedPath?: string | null
  cachedPngPath?: string | null
  error?: string | null
}

/**
 * 解析 GUI 文件
 */
export async function parseGuiFile(path: string): Promise<GuiParseResult> {
  return await invoke('parse_gui_file', { path })
}

/**
 * 解析 GUI 内容
 */
export async function parseGuiContent(content: string): Promise<GuiParseResult> {
  return await invoke('parse_gui_content', { content })
}

/**
 * 解析 GFX 文件
 */
export async function parseGfxFile(path: string): Promise<GfxParseResult> {
  return await invoke('parse_gfx_file', { path })
}

export async function parseGfxPreview(params: {
  filePath: string
  contentOverride?: string
  projectPath?: string
  gameDirectory?: string
  dependencyRoots?: string[]
}): Promise<GfxSpritePreviewItem[]> {
  return await invoke('parse_gfx_preview', {
    filePath: params.filePath,
    contentOverride: params.contentOverride,
    projectPath: params.projectPath,
    gameDirectory: params.gameDirectory,
    dependencyRoots: params.dependencyRoots
  })
}

/**
 * 解析 GUI 资源
 */
export async function resolveGuiResource(
  name: string,
  projectPath: string,
  gameDirectory: string,
  dependencyRoots: string[]
): Promise<any> {
  return await invoke('resolve_gui_resource', { name, projectPath, gameDirectory, dependencyRoots })
}

/**
 * 根据国策 icon 名称加载图标（会在项目、依赖和游戏目录中查找 gfx/interface/goals/*.gfx）
 */
export async function loadFocusIcon(
  iconName: string,
  projectRoot?: string,
  gameRoot?: string
): Promise<ImageReadResult> {
  const resp = await invoke('load_focus_icon', { iconName, projectRoot, gameRoot })
  return normalizeImageReadResult(resp)
}

// ==================== 图标缓存 ====================

/**
 * 读取图标缓存
 */
export async function readIconCache(iconName: string): Promise<ImageReadResult> {
  const resp = await invoke('read_icon_cache', { iconName })
  return normalizeImageReadResult(resp)
}

/**
 * 写入图标缓存
 */
export async function writeIconCache(
  iconName: string,
  base64: string,
  mimeType: string
): Promise<ImageReadResult> {
  const resp = await invoke('write_icon_cache', { iconName, base64, mimeType, mime_type: mimeType })
  return normalizeImageReadResult(resp)
}

/**
 * 清理图标缓存
 */
export async function clearIconCache(): Promise<ImageReadResult> {
  const resp = await invoke('clear_icon_cache')
  return normalizeImageReadResult(resp)
}

// ==================== 地图引擎 ====================

export interface RGBColor {
  r: number
  g: number
  b: number
  a: number
}

export interface ProvinceDefinition {
  id: number
  r: number
  g: number
  b: number
  type: string
  coastal: boolean
  terrain: string
  continent: number
  name?: string
}

export interface DefaultMap {
  definitions: string
  provinces: string
  adjacencies: string
  continent: string
  rivers: string
  terrain_definition?: string
}

export interface StateDefinition {
  id: number
  name: string
  provinces: number[]
  owner: string
  cores: string[]
  claims: string[]
}

export interface ProvinceEdge {
  from_id: number
  to_id: number
  points: [number, number][]
}

export interface ProvinceInstance {
  definition: ProvinceDefinition
  bounding_box?: {
    min_x: number
    min_y: number
    max_x: number
    max_y: number
  }
  pixels_count: number
}

export interface ProvinceMapData {
  width: number
  height: number
  province_ids: number[]
  instances: ProvinceInstance[]
  edges: ProvinceEdge[]
}

export interface MapLoadResult<T> {
  success: boolean
  message: string
  data?: T
}

export async function loadMapDefinitions(path: string): Promise<MapLoadResult<ProvinceDefinition[]>> {
  return await invoke('load_map_definitions', { path })
}

export async function loadDefaultMap(path: string): Promise<MapLoadResult<DefaultMap>> {
  return await invoke('load_default_map', { path })
}

export async function loadProvincesBmp(
  path: string,
  definitions: ProvinceDefinition[]
): Promise<MapLoadResult<ProvinceMapData>> {
  return await invoke('load_provinces_bmp', { path, definitions })
}

export async function getProvinceMapBinary(
  path: string,
  definitions: ProvinceDefinition[]
): Promise<Uint8Array> {
  const res = await invoke<number[]>('get_province_map_binary', { path, definitions })
  return new Uint8Array(res)
}

export async function generateColoredMap(
  provinceIds: number[] | Uint32Array,
  colorMap: Record<number, RGBColor>,
  defaultColor: RGBColor,
  width: number,
  height: number,
  downsample?: number
): Promise<Uint8Array> {
  // 注意：Tauri invoke 自动序列化，大数组建议分块或使用二进制
  const res = await invoke<number[]>('generate_colored_map', {
    provinceIds: Array.from(provinceIds),
    colorMap,
    defaultColor,
    width,
    height,
    downsample
  })
  return new Uint8Array(res)
}

export async function getDefinitionColorMap(
  definitions: ProvinceDefinition[]
): Promise<Record<number, RGBColor>> {
  return await invoke('get_definition_color_map', { definitions })
}

export async function loadAllStates(statesDir: string): Promise<StateDefinition[]> {
  return await invoke('load_all_states', { statesDir })
}

export async function loadCountryColors(path: string): Promise<Record<string, RGBColor>> {
  return await invoke('load_country_colors', { path })
}

export async function getProvinceOwnerColorMap(
  states: StateDefinition[],
  countryColors: Record<string, RGBColor>
): Promise<Record<number, RGBColor>> {
  return await invoke('get_province_owner_color_map', { states, countryColors })
}

export async function initializeMapContext(
  mapPath: string,
  definitionsPath: string,
  statesPath: string,
  countryColorsPath: string
): Promise<MapInitializationData> {
  return await invoke('initialize_map_context', {
    mapPath,
    definitionsPath,
    statesPath,
    countryColorsPath
  })
}

export async function initializeMapContextWithFallback(
  projectRoot: string,
  gameDirectory?: string,
  dependencyRoots?: string[]
): Promise<MapInitializationData> {
  return await invoke('initialize_map_context_with_fallback', {
    projectRoot,
    gameDirectory,
    dependencyRoots
  })
}

export async function getMapTileDirect(
  x: number,
  y: number,
  zoom: number,
  mode: string
): Promise<Uint8Array> {
  const res = await invoke<number[]>('get_map_tile_direct', { x, y, zoom, mode })
  return new Uint8Array(res)
}

export interface MapMetadata {
  width: number
  height: number
  province_count: number
}

export interface MapInitializationData {
  metadata: MapMetadata
  definitions: ProvinceDefinition[]
  states: StateDefinition[]
  defaultMap?: DefaultMap
}

export async function getMapMetadata(): Promise<MapMetadata> {
  return await invoke('get_map_metadata')
}

export async function getMapPreview(
  targetWidth: number,
  targetHeight: number,
  mode: string
): Promise<Uint8Array> {
  const res = await invoke<number[]>('get_map_preview', {
    targetWidth,
    targetHeight,
    mode
  })
  return new Uint8Array(res)
}

export async function getProvinceAtPoint(x: number, y: number): Promise<number | null> {
  return await invoke('get_province_at_point', { x, y })
}

export async function getProvinceOutline(provinceId: number): Promise<Uint32Array> {
  const res = await invoke<number[]>('get_province_outline', { provinceId })
  return new Uint32Array(new Uint8Array(res).buffer)
}

/**
 * 获取地区的轮廓点集
 */
export async function getStateOutline(stateId: number): Promise<Uint32Array> {
  const res = await invoke<number[]>('get_state_outline', { stateId })
  return new Uint32Array(new Uint8Array(res).buffer)
}

/**
 * 加载省份中文本地化名字
 */
export async function loadProvinceLocalizations(roots: string[]): Promise<Record<string, string>> {
  return await invoke('load_province_localizations', { roots })
}

// ==================== MIO 预览 ====================

export interface MioPreviewData {
  source_file: string
  mios: MioDto[]
}

export interface MioDto {
  id: string
  traits: MioTraitDto[]
  warnings: string[]
}

export interface MioTraitDto {
  id: string
  name?: string
  icon?: string
  x: number
  y: number
  relative_position_id?: string
  any_parent: string[]
  all_parents: string[]
  parent?: { traits: string[]; num_needed: number }
  mutually_exclusive: string[]
  effects_present: string[]
  file: string
  start: number
  end: number
  line: number
}

export async function parseMioPreview(params: {
  filePath: string
  contentOverride?: string
  projectPath?: string
  gameDirectory?: string
  dependencyRoots?: string[]
}): Promise<MioPreviewData> {
  return await invoke('parse_mio_preview', {
    filePath: params.filePath,
    contentOverride: params.contentOverride,
    projectPath: params.projectPath,
    gameDirectory: params.gameDirectory,
    dependencyRoots: params.dependencyRoots
  })
}

export async function startTerminalSession(
  shell: TerminalShell,
  cwd?: string
): Promise<TerminalSessionInfo> {
  return await invoke('start_terminal_session', { shell, cwd })
}

export async function writeTerminalInput(
  sessionId: string,
  input: string
): Promise<boolean> {
  return await invoke('write_terminal_input', { sessionId, input })
}

export async function stopTerminalSession(sessionId: string): Promise<boolean> {
  return await invoke('stop_terminal_session', { sessionId })
}
