/**
 * 编辑器模板插入功能
 * 提供 Idea、Tag 初始态定义、权力平衡等模板的插入
 */

export interface EditorMethods {
  insertText?: (text: string) => void
  getSelectedText?: () => string
  cutSelection?: () => string
  selectAll?: () => void
}

export interface OpenFileLike {
  node: {
    path: string
  }
}

export interface PaneLike {
  activeFileIndex: number
  openFiles: OpenFileLike[]
}

/**
 * 检查文件路径是否在指定目录下
 */
function isInDirectory(filePath: string, directory: string): boolean {
  const normalizedPath = filePath.replace(/\\/g, '/')
  return normalizedPath.includes(directory)
}

/**
 * Idea 模板内容
 */
const IDEA_TEMPLATE = `ideas = {
	country = {
		idea_name = {
			picture = your_image
			allowed = {
				always = yes
			}
			allowed_civil_war = {
				always = yes
			}
			modifier = {
			}
		}
	}
}`

/**
 * Tag 初始态定义模板内容
 */
const TAG_TEMPLATE = `capital = your_tag_owner_provinces

set_research_slots = your_research_slots

set_oob = army_file

set_stability = your_stability_value
set_war_support = your_war_support_value

set_politics = {
	ruling_party = your_ruling_party
	elections_allowed = no
}

set_popularities = {
	democratic = democratic_value
	communism = communism_value
	neutrality = neutrality_value
	fascism = fascism_value
}

add_ideas = {
	idea1
	idea2	
}

recruit_character = char1
recruit_character = char2

set_technology = {
}`

/**
 * 权力平衡模板内容
 */
const BOP_TEMPLATE = `bop_name = {

	initial_value = #默认值

	left_side = #左侧名称
	right_side = #右侧名称

	decision_category = #决议组
	
	# 中间范围
	range = {

		id = 

		min = 

		max = 

		modifier = {
		}
	}
	
	#右侧
	side = {

		id = #右侧名称

		icon = 
		
		# 阈值1
		range = {

			id = 

			min = 

			max = 

			modifier = {
			}
		}
		
		# 阈值2
		range = {
			...
		}
	}
	
	#左侧同理
}`

/**
 * 插入 Idea 模板
 * @param pane 当前窗格
 * @param editorMethods 编辑器方法
 * @returns 是否成功插入
 */
export function insertIdeaTemplate(pane: PaneLike, editorMethods: EditorMethods): boolean {
  if (pane.activeFileIndex === -1) return false
  
  const currentFile = pane.openFiles[pane.activeFileIndex]
  if (!currentFile) return false
  
  const filePath = currentFile.node.path
  
  // 检查文件是否在 common/ideas/ 目录下
  if (!isInDirectory(filePath, 'common/ideas/')) {
    alert('错误：只能在 common/ideas/ 目录下的文件中插入 Idea 模板')
    return false
  }
  
  editorMethods.insertText?.(IDEA_TEMPLATE)
  return true
}

/**
 * 插入 Tag 初始态定义模板
 * @param pane 当前窗格
 * @param editorMethods 编辑器方法
 * @returns 是否成功插入
 */
export function insertTagTemplate(pane: PaneLike, editorMethods: EditorMethods): boolean {
  if (pane.activeFileIndex === -1) return false
  
  const currentFile = pane.openFiles[pane.activeFileIndex]
  if (!currentFile) return false
  
  const filePath = currentFile.node.path
  
  // 检查文件是否在 history/countries/ 目录下
  if (!isInDirectory(filePath, 'history/countries/')) {
    alert('错误：只能在 history/countries/ 目录下的文件中插入 Tag 初始态定义模板')
    return false
  }
  
  editorMethods.insertText?.(TAG_TEMPLATE)
  return true
}

/**
 * 插入权力平衡模板
 * @param pane 当前窗格
 * @param editorMethods 编辑器方法
 * @returns 是否成功插入
 */
export function insertBopTemplate(pane: PaneLike, editorMethods: EditorMethods): boolean {
  if (pane.activeFileIndex === -1) return false
  
  const currentFile = pane.openFiles[pane.activeFileIndex]
  if (!currentFile) return false
  
  const filePath = currentFile.node.path
  
  // 检查文件是否在 common/bop/ 目录下
  if (!isInDirectory(filePath, 'common/bop/')) {
    alert('错误：只能在 common/bop/ 目录下的文件中插入权力平衡模板')
    return false
  }
  
  editorMethods.insertText?.(BOP_TEMPLATE)
  return true
}

/**
 * 根据动作类型插入对应模板
 * @param action 动作类型
 * @param pane 当前窗格
 * @param editorMethods 编辑器方法
 */
export function handleInsertTemplate(
  action: 'insertIdeaTemplate' | 'insertTagTemplate' | 'insertBopTemplate',
  pane: PaneLike,
  editorMethods: EditorMethods
): void {
  switch (action) {
    case 'insertIdeaTemplate':
      insertIdeaTemplate(pane, editorMethods)
      break
    case 'insertTagTemplate':
      insertTagTemplate(pane, editorMethods)
      break
    case 'insertBopTemplate':
      insertBopTemplate(pane, editorMethods)
      break
  }
}
