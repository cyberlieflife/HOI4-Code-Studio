function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

function renderTokenizedLine(line: string, className: string): string {
  const parts: string[] = []
  let lastIndex = 0

  for (const match of line.matchAll(/\S+/g)) {
    const token = match[0]
    const index = match.index ?? 0

    if (index > lastIndex) {
      parts.push(escapeHtml(line.slice(lastIndex, index)))
    }

    parts.push(`<span class="${className}">${escapeHtml(token)}</span>`)
    lastIndex = index + token.length
  }

  if (lastIndex < line.length) {
    parts.push(escapeHtml(line.slice(lastIndex)))
  }

  return parts.join('')
}

function highlightFieldLine(line: string): string {
  const match = line.match(/^(\s*)([\w.-]+(?:\s+[\w.-]+)*)(\s+:\s*|\s*:\s+)(.*)$/)
  if (!match) {
    return escapeHtml(line)
  }

  const [, indent, fieldName, separator, value] = match

  return [
    escapeHtml(indent),
    `<span class="ps-field-name">${escapeHtml(fieldName)}</span>`,
    `<span class="ps-field-separator">${escapeHtml(separator)}</span>`,
    escapeHtml(value)
  ].join('')
}

function isTableSeparatorLine(line: string): boolean {
  const trimmed = line.trim()
  return trimmed.length > 0 && /^[-\s]+$/.test(line) && trimmed.includes('-')
}

function isTableHeaderLine(line: string, nextLine?: string): boolean {
  if (!nextLine || !line.trim() || !isTableSeparatorLine(nextLine)) {
    return false
  }

  const headerTokens = [...line.matchAll(/\S+/g)]
  const separatorTokens = [...nextLine.matchAll(/-+/g)]

  if (headerTokens.length === 0 || headerTokens.length !== separatorTokens.length) {
    return false
  }

  return headerTokens.every((token, index) => {
    const headerStart = token.index ?? 0
    const separatorStart = separatorTokens[index]?.index ?? -100
    return Math.abs(headerStart - separatorStart) <= 2
  })
}

export function highlightPowerShellTerminalOutput(text: string): string {
  const normalizedText = text.replace(/\r\n/g, '\n')
  const lines = normalizedText.split('\n')

  return lines.map((line: string, index: number) => {
    if (isTableHeaderLine(line, lines[index + 1])) {
      return renderTokenizedLine(line, 'ps-table-header')
    }

    return highlightFieldLine(line)
  }).join('\n')
}
