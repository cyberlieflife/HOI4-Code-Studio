import { ref, type Ref } from 'vue'
import { readFileContent, readJsonFile, writeJsonFile } from '../api/tauri'
import { logger } from '../utils/logger'
import { toast } from '../utils/notification'

type ConfirmType = 'info' | 'warning' | 'danger'

export interface ProjectInfo {
  name: string
  version: string
  created_at: string
}

export function useProjectBootstrap(
  projectPath: Ref<string>,
  showConfirmDialog: (message: string, title: string, type?: ConfirmType) => Promise<boolean>
) {
  const projectInfo = ref<ProjectInfo | null>(null)

  async function loadProjectInfo() {
    if (!projectPath.value) return

    try {
      const projectJsonPath = `${projectPath.value}/project.json`
      const result = await readJsonFile(projectJsonPath)
      if (result.success && result.data) {
        projectInfo.value = result.data as ProjectInfo
        return
      }

      const shouldInitialize = await showConfirmDialog(
        '检测到此文件夹不是 HOI4 Code Studio 项目，是否要将其初始化为项目？',
        '初始化项目',
        'info'
      )

      if (!shouldInitialize) {
        return
      }

      try {
        const descriptorPath = `${projectPath.value}/descriptor.mod`
        const descriptorResult = await readFileContent(descriptorPath)
        if (!descriptorResult.success) {
          toast.error(`无法读取 descriptor.mod 文件: ${descriptorResult.message}`)
          return
        }

        const content = descriptorResult.content ?? ''
        const nameMatch = content.match(/^name\s*=\s*"([^"]+)"/m)
        const modName = nameMatch ? nameMatch[1] : 'Unknown Mod'
        const projectData: ProjectInfo = {
          name: modName,
          version: '1.0.0',
          created_at: new Date().toISOString()
        }

        const writeResult = await writeJsonFile(projectJsonPath, projectData)
        if (!writeResult.success) {
          toast.error(`项目初始化失败: ${writeResult.message}`)
          return
        }

        projectInfo.value = projectData
        toast.success(`项目初始化成功，项目名称: ${modName}`)
      } catch (error) {
        logger.error('项目初始化失败:', error)
        toast.error(`项目初始化失败: ${error}`)
      }
    } catch (error) {
      logger.error('加载项目信息失败:', error)
    }
  }

  return {
    projectInfo,
    loadProjectInfo
  }
}
