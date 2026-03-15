import { describe, expect, it } from 'vitest'
import { highlightPowerShellTerminalOutput } from '../../../src/utils/powerShellTerminalHighlight'

describe('highlightPowerShellTerminalOutput', () => {
  it('高亮 PowerShell 列表字段', () => {
    const result = highlightPowerShellTerminalOutput('Name : WindowsPowerShell')

    expect(result).toContain('<span class="ps-field-name">Name</span>')
    expect(result).toContain('<span class="ps-field-separator"> : </span>')
    expect(result).toContain('WindowsPowerShell')
  })

  it('高亮 PowerShell 表格表头', () => {
    const result = highlightPowerShellTerminalOutput('Name Length\n---- ------\nfile 1024')

    expect(result).toContain('<span class="ps-table-header">Name</span>')
    expect(result).toContain('<span class="ps-table-header">Length</span>')
    expect(result).toContain('---- ------')
    expect(result).toContain('file 1024')
  })

  it('保留普通输出，不误高亮', () => {
    const result = highlightPowerShellTerminalOutput('PS D:\\HOI4-Code-Studio> Get-ChildItem')

    expect(result).toBe('PS D:\\HOI4-Code-Studio&gt; Get-ChildItem')
  })

  it('转义危险 HTML 字符', () => {
    const result = highlightPowerShellTerminalOutput('Name : <script>alert(1)</script>')

    expect(result).toContain('&lt;script&gt;alert(1)&lt;/script&gt;')
    expect(result).not.toContain('<script>')
  })

  it('兼容 CRLF 输出', () => {
    const result = highlightPowerShellTerminalOutput('Path : C:\\Temp\r\nMode : d----')

    expect(result).toContain('<span class="ps-field-name">Path</span>')
    expect(result).toContain('<span class="ps-field-name">Mode</span>')
  })
})
