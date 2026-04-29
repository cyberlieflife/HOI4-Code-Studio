/**
 * 文件工具函数
 * 纯函数，无状态依赖
 */

import type { FileNode } from '../composables/useFileManager'

export const INITIAL_FILE_TREE_DEPTH = 1
export const DIRECTORY_EXPAND_LOAD_DEPTH = 1

/**
 * 从路径中提取文件名
 * @param p 文件路径
 * @returns 文件名
 */
export function basename(p: string): string {
  return p.replace(/\\/g, '/').split('/').pop() || p
}

/**
 * 转义正则表达式特殊字符
 * @param input 输入字符串
 * @returns 转义后的字符串
 */
export function escapeRegExp(input: string): string {
  return input.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

/**
 * 检查是否为图片文件
 * @param filePath 文件路径
 * @returns 是否为图片文件
 */
export function isImageFile(filePath: string): boolean {
  const ext = filePath.split('.').pop()?.toLowerCase()
  return ['png', 'jpg', 'jpeg', 'tga', 'bmp', 'gif', 'webp', 'dds'].includes(ext || '')
}

/**
 * 检查目标路径是否在基础路径下
 * @param target 目标路径
 * @param base 基础路径
 * @returns 是否在基础路径下
 */
export function isPathUnder(target: string, base: string): boolean {
  const normalize = (p: string) => p.replace(/\\/g, '/').toLowerCase().replace(/\/+$/g, '')
  const t = normalize(target)
  const b = normalize(base)
  return t === b || t.startsWith(b + '/')
}

interface RustFileNode {
  name: string
  path: string
  is_directory: boolean
  children?: RustFileNode[]
  expanded?: boolean
}

/**
 * 转换 Rust 返回的文件节点格式为前端格式
 * @param node Rust 返回的文件节点
 * @returns 前端格式的文件节点
 */
export function convertRustFileNode(node: RustFileNode): FileNode {
  return {
    name: node.name,
    path: node.path,
    isDirectory: node.is_directory,
    children: node.children?.map(convertRustFileNode),
    expanded: node.expanded || false
  }
}
