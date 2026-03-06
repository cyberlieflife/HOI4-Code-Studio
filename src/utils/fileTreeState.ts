export interface TreeStateNode {
  path: string
  isDirectory: boolean
  expanded?: boolean
  children?: TreeStateNode[]
}

export function collectExpandedPaths<T extends TreeStateNode>(nodes: T[]): Set<string> {
  const expandedPaths = new Set<string>()

  function traverse(node: T) {
    if (node.isDirectory && node.expanded) {
      expandedPaths.add(node.path)
      if (node.children) {
        node.children.forEach((child) => traverse(child as T))
      }
    }
  }

  nodes.forEach(traverse)
  return expandedPaths
}

export function restoreExpandedState<T extends TreeStateNode>(nodes: T[], expandedPaths: Set<string>): void {
  function traverse(node: T) {
    if (node.isDirectory && expandedPaths.has(node.path)) {
      node.expanded = true
      if (node.children) {
        node.children.forEach((child) => traverse(child as T))
      }
    }
  }

  nodes.forEach(traverse)
}

export function mergeExpandedChildren<T extends TreeStateNode>(
  oldNodes: T[],
  newNodes: T[],
  expandedPaths: Set<string>
): void {
  const oldByPath = new Map<string, T>()

  const indexOld = (nodes: T[]) => {
    nodes.forEach((node) => {
      oldByPath.set(node.path, node)
      if (node.children) {
        indexOld(node.children as T[])
      }
    })
  }

  indexOld(oldNodes)

  const merge = (node: T) => {
    if (!node.isDirectory) return

    if (expandedPaths.has(node.path)) {
      const old = oldByPath.get(node.path)
      if ((!node.children || node.children.length === 0) && old?.children && old.children.length > 0) {
        node.children = old.children as T[]
      }
    }

    if (node.children) {
      node.children.forEach((child: TreeStateNode) => merge(child as T))
    }
  }

  newNodes.forEach(merge)
}
