import type { FileNode } from '../composables/useFileManager'

export function collectExpandedPaths(nodes: FileNode[]): Set<string> {
  const expandedPaths = new Set<string>()

  function traverse(node: FileNode) {
    if (node.isDirectory && node.expanded) {
      expandedPaths.add(node.path)
      if (node.children) {
        node.children.forEach(traverse)
      }
    }
  }

  nodes.forEach(traverse)
  return expandedPaths
}

export function restoreExpandedState(nodes: FileNode[], expandedPaths: Set<string>): void {
  function traverse(node: FileNode) {
    if (node.isDirectory && expandedPaths.has(node.path)) {
      node.expanded = true
      if (node.children) {
        node.children.forEach(traverse)
      }
    }
  }

  nodes.forEach(traverse)
}

export function mergeExpandedChildren(oldNodes: FileNode[], newNodes: FileNode[], expandedPaths: Set<string>): void {
  const oldByPath = new Map<string, FileNode>()

  const indexOld = (nodes: FileNode[]) => {
    nodes.forEach((node) => {
      oldByPath.set(node.path, node)
      if (node.children) {
        indexOld(node.children)
      }
    })
  }

  indexOld(oldNodes)

  const merge = (node: FileNode) => {
    if (!node.isDirectory) return

    if (expandedPaths.has(node.path)) {
      const old = oldByPath.get(node.path)
      if ((!node.children || node.children.length === 0) && old?.children && old.children.length > 0) {
        node.children = old.children
      }
    }

    if (node.children) {
      node.children.forEach(merge)
    }
  }

  newNodes.forEach(merge)
}
