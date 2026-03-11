import { ref, computed } from 'vue'
import { loadSettings, saveSettings, listThemes, upsertTheme, deleteTheme } from '../api/tauri'

/**
 * 主题定义接口
 */
export interface Theme {
  id: string
  name: string
  colors: {
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
}

/**
 * 预定义主题列表
 */
export const themes: Theme[] = [
  {
    id: 'onedark',
    name: 'One Dark',
    colors: {
      bg: '#282c34',
      bgSecondary: '#21252b',
      fg: '#abb2bf',
      comment: '#5c6370',
      border: '#181a1f',
      selection: '#3e4451',
      accent: '#61afef',
      success: '#98c379',
      warning: '#e5c07b',
      error: '#e06c75',
      keyword: '#c678dd'
    }
  },
  {
    id: 'one-light',
    name: 'One Light',
    colors: {
      bg: '#fafafa',
      bgSecondary: '#f0f0f0',
      fg: '#383a42',
      comment: '#a0a1a7',
      border: '#d3d4d5',
      selection: '#e5e5e6',
      accent: '#4078f2',
      success: '#50a14f',
      warning: '#c18401',
      error: '#e45649',
      keyword: '#a626a4'
    }
  },
  {
    id: 'vscode',
    name: 'VS Code Dark',
    colors: {
      bg: '#1e1e1e',
      bgSecondary: '#252526',
      fg: '#d4d4d4',
      comment: '#6a9955',
      border: '#3c3c3c',
      selection: '#264f78',
      accent: '#569cd6',
      success: '#4ec9b0',
      warning: '#dcdcaa',
      error: '#f14c4c',
      keyword: '#c586c0'
    }
  },
  {
    id: 'github',
    name: 'GitHub Dark',
    colors: {
      bg: '#0d1117',
      bgSecondary: '#161b22',
      fg: '#c9d1d9',
      comment: '#8b949e',
      border: '#30363d',
      selection: '#388bfd26',
      accent: '#58a6ff',
      success: '#3fb950',
      warning: '#d29922',
      error: '#f85149',
      keyword: '#ff7b72'
    }
  },
  {
    id: 'catppuccin-latte',
    name: 'Catppuccin Latte',
    colors: {
      bg: '#eff1f5',
      bgSecondary: '#e6e9ef',
      fg: '#4c4f69',
      comment: '#8c8fa1',
      border: '#bcc0cc',
      selection: '#acb0be',
      accent: '#1e66f5',
      success: '#40a02b',
      warning: '#df8e1d',
      error: '#d20f39',
      keyword: '#8839ef'
    }
  },
  {
    id: 'catppuccin-frappe',
    name: 'Catppuccin Frappé',
    colors: {
      bg: '#303446',
      bgSecondary: '#292c3c',
      fg: '#c6d0f5',
      comment: '#838ba7',
      border: '#51576d',
      selection: '#626880',
      accent: '#8caaee',
      success: '#a6d189',
      warning: '#e5c890',
      error: '#e78284',
      keyword: '#ca9ee6'
    }
  },
  {
    id: 'catppuccin-macchiato',
    name: 'Catppuccin Macchiato',
    colors: {
      bg: '#24273a',
      bgSecondary: '#1e2030',
      fg: '#cad3f5',
      comment: '#8087a2',
      border: '#494d64',
      selection: '#5b6078',
      accent: '#8aadf4',
      success: '#a6da95',
      warning: '#eed49f',
      error: '#ed8796',
      keyword: '#c6a0f6'
    }
  },
  {
    id: 'catppuccin-mocha',
    name: 'Catppuccin Mocha',
    colors: {
      bg: '#1e1e2e',
      bgSecondary: '#181825',
      fg: '#cdd6f4',
      comment: '#6c7086',
      border: '#313244',
      selection: '#45475a',
      accent: '#89b4fa',
      success: '#a6e3a1',
      warning: '#f9e2af',
      error: '#f38ba8',
      keyword: '#cba6f7'
    }
  },
  {
    id: 'dracula',
    name: 'Dracula',
    colors: {
      bg: '#282a36',
      bgSecondary: '#21222c',
      fg: '#f8f8f2',
      comment: '#6272a4',
      border: '#44475a',
      selection: '#44475a',
      accent: '#8be9fd',
      success: '#50fa7b',
      warning: '#f1fa8c',
      error: '#ff5555',
      keyword: '#ff79c6'
    }
  },
  {
    id: 'monokai',
    name: 'Monokai',
    colors: {
      bg: '#272822',
      bgSecondary: '#1e1f1c',
      fg: '#f8f8f2',
      comment: '#75715e',
      border: '#3e3d32',
      selection: '#49483e',
      accent: '#66d9ef',
      success: '#a6e22e',
      warning: '#e6db74',
      error: '#f92672',
      keyword: '#ae81ff'
    }
  },
  {
    id: 'github-light',
    name: 'GitHub Light',
    colors: {
      bg: '#ffffff',
      bgSecondary: '#f6f8fa',
      fg: '#24292e',
      comment: '#6a737d',
      border: '#e1e4e8',
      selection: '#0366d626',
      accent: '#0366d6',
      success: '#28a745',
      warning: '#d73a49',
      error: '#cb2431',
      keyword: '#d73a49'
    }
  },
  {
    id: 'solarized-light',
    name: 'Solarized Light',
    colors: {
      bg: '#fdf6e3',
      bgSecondary: '#eee8d5',
      fg: '#657b83',
      comment: '#93a1a1',
      border: '#d8d0c0',
      selection: '#eee8d5',
      accent: '#268bd2',
      success: '#859900',
      warning: '#b58900',
      error: '#dc322f',
      keyword: '#859900'
    }
  },
  {
    id: 'solarized-dark',
    name: 'Solarized Dark',
    colors: {
      bg: '#002b36',
      bgSecondary: '#073642',
      fg: '#839496',
      comment: '#586e75',
      border: '#002b36',
      selection: '#073642',
      accent: '#268bd2',
      success: '#859900',
      warning: '#b58900',
      error: '#dc322f',
      keyword: '#859900'
    }
  },
  {
    id: 'nord',
    name: 'Nord',
    colors: {
      bg: '#2e3440',
      bgSecondary: '#3b4252',
      fg: '#eceff4',
      comment: '#4c566a',
      border: '#434c5e',
      selection: '#4c566a',
      accent: '#81a1c1',
      success: '#a3be8c',
      warning: '#ebcb8b',
      error: '#bf616a',
      keyword: '#b48ead'
    }
  },
  {
    id: 'gruvbox-dark',
    name: 'Gruvbox Dark',
    colors: {
      bg: '#282828',
      bgSecondary: '#3c3836',
      fg: '#ebdbb2',
      comment: '#928374',
      border: '#504945',
      selection: '#665c54',
      accent: '#83a598',
      success: '#b8bb26',
      warning: '#fabd2f',
      error: '#fb4934',
      keyword: '#8ec07c'
    }
  },
  {
    id: 'material-dark',
    name: 'Material Dark',
    colors: {
      bg: '#263238',
      bgSecondary: '#2e3c43',
      fg: '#eeffff',
      comment: '#546e7a',
      border: '#37474f',
      selection: '#37474f',
      accent: '#82aaff',
      success: '#c3e88d',
      warning: '#ffcb6b',
      error: '#f07178',
      keyword: '#c792ea'
    }
  },
  {
    id: 'tokyo-night',
    name: 'Tokyo Night',
    colors: {
      bg: '#1a1b26',
      bgSecondary: '#16161e',
      fg: '#a9b1d6',
      comment: '#565f89',
      border: '#414868',
      selection: '#33467c',
      accent: '#7aa2f7',
      success: '#9ece6a',
      warning: '#e0af68',
      error: '#f7768e',
      keyword: '#bb9af7'
    }
  },
  {
    id: 'gruvbox-light',
    name: 'Gruvbox Light',
    colors: {
      bg: '#fbf1c7',
      bgSecondary: '#f9f5d7',
      fg: '#3c3836',
      comment: '#928374',
      border: '#d5c4a1',
      selection: '#ebdbb2',
      accent: '#076678',
      success: '#79740e',
      warning: '#b57614',
      error: '#9d0006',
      keyword: '#8f3f71'
    }
  },
  {
    id: 'palenight',
    name: 'Palenight',
    colors: {
      bg: '#292d3e',
      bgSecondary: '#1b1e2b',
      fg: '#a6accd',
      comment: '#676e95',
      border: '#4a4f6a',
      selection: '#444267',
      accent: '#82aaff',
      success: '#c3e88d',
      warning: '#ffcb6b',
      error: '#f07178',
      keyword: '#c792ea'
    }
  },
  {
    id: 'shades-of-purple',
    name: 'Shades of Purple',
    colors: {
      bg: '#1e1e3f',
      bgSecondary: '#2d2b55',
      fg: '#a599e9',
      comment: '#b362ff',
      border: '#4e4e8a',
      selection: '#4d21fc',
      accent: '#ff628c',
      success: '#a5ff90',
      warning: '#fffb80',
      error: '#ff5454',
      keyword: '#ff9d00'
    }
  },
  {
    id: 'cobalt2',
    name: 'Cobalt2',
    colors: {
      bg: '#193549',
      bgSecondary: '#15232d',
      fg: '#ffffff',
      comment: '#0088ff',
      border: '#1f4662',
      selection: '#255f7e',
      accent: '#ffc600',
      success: '#9dff00',
      warning: '#ff7b00',
      error: '#ff2c6d',
      keyword: '#ff9d00'
    }
  },
  {
    id: 'night-owl',
    name: 'Night Owl',
    colors: {
      bg: '#011627',
      bgSecondary: '#0b2942',
      fg: '#d6deeb',
      comment: '#5f7e97',
      border: '#1d3b53',
      selection: '#1d3b53',
      accent: '#82aaff',
      success: '#22da6e',
      warning: '#ecc48d',
      error: '#ef5350',
      keyword: '#c792ea'
    }
  },
  {
    id: 'panda',
    name: 'Panda',
    colors: {
      bg: '#1a1a1a',
      bgSecondary: '#2d2d2d',
      fg: '#e6e6e6',
      comment: '#676b79',
      border: '#373737',
      selection: '#3f3f3f',
      accent: '#ff2c6d',
      success: '#19f9d8',
      warning: '#ffb86c',
      error: '#ff2c6d',
      keyword: '#ff75b5'
    }
  },
  {
    id: 'vibrant-ink',
    name: 'Vibrant Ink',
    colors: {
      bg: '#0f0f0f',
      bgSecondary: '#1c1c1c',
      fg: '#ffffff',
      comment: '#9933cc',
      border: '#333333',
      selection: '#2d2d2d',
      accent: '#66ccff',
      success: '#99ff33',
      warning: '#ffcc00',
      error: '#ff3333',
      keyword: '#ff6600'
    }
  },
  {
    id: 'arc',
    name: 'Arc',
    colors: {
      bg: '#1e2127',
      bgSecondary: '#2a2d34',
      fg: '#9da5b4',
      comment: '#5c6370',
      border: '#3a3f4a',
      selection: '#3e4451',
      accent: '#61afef',
      success: '#98c379',
      warning: '#e5c07b',
      error: '#e06c75',
      keyword: '#c678dd'
    }
  },
  {
    id: 'mayukai',
    name: 'Mayukai',
    colors: {
      bg: '#1f1f1f',
      bgSecondary: '#2b2b2b',
      fg: '#f8f8f2',
      comment: '#75715e',
      border: '#3e3e3e',
      selection: '#414339',
      accent: '#66d9ef',
      success: '#a6e22e',
      warning: '#e6db74',
      error: '#f92672',
      keyword: '#ae81ff'
    }
  },
  {
    id: 'material-light',
    name: 'Material Light',
    colors: {
      bg: '#fafafa',
      bgSecondary: '#ffffff',
      fg: '#37474f',
      comment: '#90a4ae',
      border: '#e0e0e0',
      selection: '#e0e0e0',
      accent: '#2962ff',
      success: '#00c853',
      warning: '#ffab00',
      error: '#ff3d00',
      keyword: '#aa00ff'
    }
  },
  {
    id: 'atom-dark',
    name: 'Atom Dark',
    colors: {
      bg: '#1d1f23',
      bgSecondary: '#282c34',
      fg: '#abb2bf',
      comment: '#5c6370',
      border: '#181a1f',
      selection: '#3e4451',
      accent: '#528bff',
      success: '#98c379',
      warning: '#e5c07b',
      error: '#e06c75',
      keyword: '#c678dd'
    }
  },
  {
    id: 'atom-light',
    name: 'Atom Light',
    colors: {
      bg: '#ffffff',
      bgSecondary: '#f9f9f9',
      fg: '#383a42',
      comment: '#a0a1a7',
      border: '#e5e5e5',
      selection: '#e5e5e6',
      accent: '#4078f2',
      success: '#50a14f',
      warning: '#c18401',
      error: '#e45649',
      keyword: '#a626a4'
    }
  },
  {
    id: 'tomorrow-light',
    name: 'Tomorrow Light',
    colors: {
      bg: '#ffffff',
      bgSecondary: '#efefef',
      fg: '#4d4d4c',
      comment: '#8e908c',
      border: '#d6d6d6',
      selection: '#d6d6d6',
      accent: '#4271ae',
      success: '#718c00',
      warning: '#eab700',
      error: '#c82829',
      keyword: '#8959a8'
    }
  },
  {
    id: 'tomorrow-night',
    name: 'Tomorrow Night',
    colors: {
      bg: '#1d1f21',
      bgSecondary: '#282a2e',
      fg: '#c5c8c6',
      comment: '#969896',
      border: '#373b41',
      selection: '#373b41',
      accent: '#81a2be',
      success: '#b5bd68',
      warning: '#f0c674',
      error: '#cc6666',
      keyword: '#b294bb'
    }
  },
  {
    id: 'base16',
    name: 'Base16',
    colors: {
      bg: '#181818',
      bgSecondary: '#202020',
      fg: '#d8d8d8',
      comment: '#585858',
      border: '#383838',
      selection: '#303030',
      accent: '#7cafc2',
      success: '#a1b56c',
      warning: '#f7ca88',
      error: '#ab4642',
      keyword: '#ba8baf'
    }
  },
  {
    id: 'cyan',
    name: 'Cyan',
    colors: {
      bg: '#1b1b1b',
      bgSecondary: '#2d2d2d',
      fg: '#e0e0e0',
      comment: '#00ffff',
      border: '#008b8b',
      selection: '#2d4b4b',
      accent: '#00ffff',
      success: '#00ff00',
      warning: '#ffff00',
      error: '#ff6b6b',
      keyword: '#ff1493'
    }
  },
  {
    id: 'neon',
    name: 'Neon',
    colors: {
      bg: '#0a0a0a',
      bgSecondary: '#1a1a1a',
      fg: '#ffffff',
      comment: '#00ff00',
      border: '#333333',
      selection: '#1a3d1a',
      accent: '#00ffff',
      success: '#00ff00',
      warning: '#ffff00',
      error: '#ff00ff',
      keyword: '#ff00ff'
    }
  },
  {
    id: 'mint',
    name: 'Mint',
    colors: {
      bg: '#1e1e1e',
      bgSecondary: '#2d2d2d',
      fg: '#e0e0e0',
      comment: '#7fbbb3',
      border: '#3a3a3a',
      selection: '#2d4b46',
      accent: '#7fbbb3',
      success: '#a7c080',
      warning: '#dbbc7f',
      error: '#e67e80',
      keyword: '#d699b6'
    }
  },
  {
    id: 'rose-pine',
    name: 'Rose Pine',
    colors: {
      bg: '#191724',
      bgSecondary: '#1f1d2e',
      fg: '#e0def4',
      comment: '#6e6a86',
      border: '#2a2837',
      selection: '#3a3849',
      accent: '#c4a7e7',
      success: '#9ccfd8',
      warning: '#f6c177',
      error: '#eb6f92',
      keyword: '#31748f'
    }
  },
  {
    id: 'hearts-allies-dark',
    name: 'Hearts of Iron - Allies Dark',
    colors: {
      bg: '#2a1f1f',
      bgSecondary: '#1f1717',
      fg: '#e0d0d0',
      comment: '#8b6b6b',
      border: '#4a3636',
      selection: '#3d2a2a',
      accent: '#d48b8b',
      success: '#8b9d6b',
      warning: '#b8a56b',
      error: '#c47b7b',
      keyword: '#b58b9b'
    }
  },
  {
    id: 'hearts-allies-light',
    name: 'Hearts of Iron - Allies Light',
    colors: {
      bg: '#f8f0f0',
      bgSecondary: '#f5e8e8',
      fg: '#4a3535',
      comment: '#a58b8b',
      border: '#d4c0c0',
      selection: '#e8d0d0',
      accent: '#c47b7b',
      success: '#6b8b5b',
      warning: '#a58b4b',
      error: '#b54b4b',
      keyword: '#9b6b8b'
    }
  },
  {
    id: 'hearts-america-dark',
    name: 'Hearts of Iron - America Dark',
    colors: {
      bg: '#1f242a',
      bgSecondary: '#171a1f',
      fg: '#d0d8e0',
      comment: '#6b7b8b',
      border: '#36424a',
      selection: '#2a343d',
      accent: '#8bb4d4',
      success: '#8bd48b',
      warning: '#d4b48b',
      error: '#d48b8b',
      keyword: '#9bb4c4'
    }
  },
  {
    id: 'hearts-america-light',
    name: 'Hearts of Iron - America Light',
    colors: {
      bg: '#f0f4f8',
      bgSecondary: '#e8eef5',
      fg: '#353a4a',
      comment: '#8b9bb4',
      border: '#c0c8d4',
      selection: '#d0d4e8',
      accent: '#6b9bc4',
      success: '#6b9b6b',
      warning: '#b49b6b',
      error: '#b46b6b',
      keyword: '#8b9bb4'
    }
  },
  {
    id: 'hearts-axis-dark',
    name: 'Hearts of Iron - Axis Dark',
    colors: {
      bg: '#2a2520',
      bgSecondary: '#1f1a16',
      fg: '#e0dcd0',
      comment: '#8b7f6b',
      border: '#4a4236',
      selection: '#3d352a',
      accent: '#b8a07b',
      success: '#7b9b6b',
      warning: '#b8a56b',
      error: '#b87b7b',
      keyword: '#a58b7b'
    }
  },
  {
    id: 'hearts-axis-light',
    name: 'Hearts of Iron - Axis Light',
    colors: {
      bg: '#f8f4f0',
      bgSecondary: '#f5efe8',
      fg: '#4a4235',
      comment: '#a59b8b',
      border: '#d4c8b8',
      selection: '#e8dfd0',
      accent: '#a58b6b',
      success: '#6b8b5b',
      warning: '#a58b4b',
      error: '#b54b4b',
      keyword: '#9b7b6b'
    }
  },
  {
    id: 'hearts-comintern-dark',
    name: 'Hearts of Iron - Comintern Dark',
    colors: {
      bg: '#2a1f1f',
      bgSecondary: '#1f1616',
      fg: '#e0d0d0',
      comment: '#8b6b6b',
      border: '#4a3636',
      selection: '#3d2a2a',
      accent: '#d46b6b',
      success: '#7b9b6b',
      warning: '#b8a56b',
      error: '#b84b4b',
      keyword: '#b58b8b'
    }
  },
  {
    id: 'hearts-comintern-light',
    name: 'Hearts of Iron - Comintern Light',
    colors: {
      bg: '#f8f0f0',
      bgSecondary: '#f5e8e8',
      fg: '#4a3535',
      comment: '#a58b8b',
      border: '#d4c0c0',
      selection: '#e8d0d0',
      accent: '#b54b4b',
      success: '#6b8b5b',
      warning: '#a58b4b',
      error: '#b52b2b',
      keyword: '#9b6b6b'
    }
  },
  {
    id: 'hearts-china-dark',
    name: 'Hearts of Iron - China Dark',
    colors: {
      bg: '#1f202a',
      bgSecondary: '#161720',
      fg: '#d0d4e0',
      comment: '#6b708b',
      border: '#363a4a',
      selection: '#2a2d3d',
      accent: '#6b8bc4',
      success: '#7bb48b',
      warning: '#b4a56b',
      error: '#b47b7b',
      keyword: '#8b9bc4'
    }
  },
  {
    id: 'hearts-china-light',
    name: 'Hearts of Iron - China Light',
    colors: {
      bg: '#f0f2f8',
      bgSecondary: '#e8ecf5',
      fg: '#353a4a',
      comment: '#8b9bb4',
      border: '#c0c4d4',
      selection: '#d0d4e8',
      accent: '#4b6bc4',
      success: '#6b9b7b',
      warning: '#a58b4b',
      error: '#b54b4b',
      keyword: '#6b7bc4'
    }
  },
  {
    id: 'oceanic-next',
    name: 'Oceanic Next',
    colors: {
      bg: '#1b2b34',
      bgSecondary: '#343d46',
      fg: '#c0c5ce',
      comment: '#65737e',
      border: '#343d46',
      selection: '#4f5b66',
      accent: '#6699cc',
      success: '#99c794',
      warning: '#fac863',
      error: '#ec5f67',
      keyword: '#c594c5'
    }
  },
  // HOI4 国家主题
  {
    id: 'hearts-uk-dark',
    name: 'Hearts of Iron - United Kingdom Dark',
    colors: {
      bg: '#1a1a2e',
      bgSecondary: '#16213e',
      fg: '#e8e8e8',
      comment: '#8b8b8b',
      border: '#3a3a5a',
      selection: '#2a2a4a',
      accent: '#c41e3a', // 英国红
      success: '#4169e1', // 英国蓝
      warning: '#ffd700', // 金色
      error: '#ff6b6b',
      keyword: '#4169e1'
    }
  },
  {
    id: 'hearts-uk-light',
    name: 'Hearts of Iron - United Kingdom Light',
    colors: {
      bg: '#f8f8ff',
      bgSecondary: '#f0f0f8',
      fg: '#2a2a3e',
      comment: '#8b8b9b',
      border: '#d0d0e0',
      selection: '#e0e0f0',
      accent: '#c41e3a', // 英国红
      success: '#4169e1', // 英国蓝
      warning: '#b8860b', // 暗金色
      error: '#dc143c',
      keyword: '#4169e1'
    }
  },
  {
    id: 'hearts-france-dark',
    name: 'Hearts of Iron - France Dark',
    colors: {
      bg: '#1a1a2e',
      bgSecondary: '#16213e',
      fg: '#e8e8e8',
      comment: '#8b8b8b',
      border: '#3a3a5a',
      selection: '#2a2a4a',
      accent: '#002395', // 法国蓝
      success: '#ffffff', // 白色
      warning: '#ed2939', // 法国红
      error: '#ff6b6b',
      keyword: '#002395'
    }
  },
  {
    id: 'hearts-france-light',
    name: 'Hearts of Iron - France Light',
    colors: {
      bg: '#f8f8ff',
      bgSecondary: '#f0f0f8',
      fg: '#2a2a3e',
      comment: '#8b8b9b',
      border: '#d0d0e0',
      selection: '#e0e0f0',
      accent: '#002395', // 法国蓝
      success: '#4a4a4a', // 深灰色代表白色
      warning: '#ed2939', // 法国红
      error: '#dc143c',
      keyword: '#002395'
    }
  },
  {
    id: 'hearts-germany-dark',
    name: 'Hearts of Iron - Germany Dark',
    colors: {
      bg: '#1a1a1a',
      bgSecondary: '#2a2a2a',
      fg: '#e8e8e8',
      comment: '#8b8b8b',
      border: '#3a3a3a',
      selection: '#2a2a2a',
      accent: '#000000', // 德国黑
      success: '#dd0000', // 德国红
      warning: '#ffce00', // 德国金
      error: '#ff6b6b',
      keyword: '#dd0000'
    }
  },
  {
    id: 'hearts-germany-light',
    name: 'Hearts of Iron - Germany Light',
    colors: {
      bg: '#f8f8f8',
      bgSecondary: '#e8e8e8',
      fg: '#2a2a2a',
      comment: '#8b8b8b',
      border: '#d0d0d0',
      selection: '#e0e0e0',
      accent: '#2a2a2a', // 深灰色代表黑色
      success: '#dd0000', // 德国红
      warning: '#ffce00', // 德国金
      error: '#dc143c',
      keyword: '#dd0000'
    }
  },
  {
    id: 'hearts-italy-dark',
    name: 'Hearts of Iron - Italy Dark',
    colors: {
      bg: '#1a1a1a',
      bgSecondary: '#2a2a2a',
      fg: '#e8e8e8',
      comment: '#8b8b8b',
      border: '#3a3a3a',
      selection: '#2a2a2a',
      accent: '#009246', // 意大利绿
      success: '#ffffff', // 白色
      warning: '#ce2b37', // 意大利红
      error: '#ff6b6b',
      keyword: '#009246'
    }
  },
  {
    id: 'hearts-italy-light',
    name: 'Hearts of Iron - Italy Light',
    colors: {
      bg: '#f8f8f8',
      bgSecondary: '#e8e8e8',
      fg: '#2a2a2a',
      comment: '#8b8b8b',
      border: '#d0d0d0',
      selection: '#e0e0e0',
      accent: '#009246', // 意大利绿
      success: '#4a4a4a', // 深灰色代表白色
      warning: '#ce2b37', // 意大利红
      error: '#dc143c',
      keyword: '#009246'
    }
  },
  {
    id: 'hearts-japan-dark',
    name: 'Hearts of Iron - Japan Dark',
    colors: {
      bg: '#1a1a1a',
      bgSecondary: '#2a2a2a',
      fg: '#e8e8e8',
      comment: '#8b8b8b',
      border: '#3a3a3a',
      selection: '#2a2a2a',
      accent: '#bc002d', // 日本红
      success: '#ffffff', // 白色
      warning: '#e8e8e8', // 浅灰色
      error: '#ff6b6b',
      keyword: '#bc002d'
    }
  },
  {
    id: 'hearts-japan-light',
    name: 'Hearts of Iron - Japan Light',
    colors: {
      bg: '#f8f8f8',
      bgSecondary: '#e8e8e8',
      fg: '#2a2a2a',
      comment: '#8b8b8b',
      border: '#d0d0d0',
      selection: '#e0e0e0',
      accent: '#bc002d', // 日本红
      success: '#4a4a4a', // 深灰色代表白色
      warning: '#e8e8e8', // 浅灰色
      error: '#dc143c',
      keyword: '#bc002d'
    }
  },
  // 流行编辑器主题
  {
    id: 'jetbrains-darcula',
    name: 'JetBrains Darcula',
    colors: {
      bg: '#2b2b2b',
      bgSecondary: '#313335',
      fg: '#bbbbbb',
      comment: '#808080',
      border: '#3c3f41',
      selection: '#214283',
      accent: '#6897bb',
      success: '#629755',
      warning: '#bbb529',
      error: '#cc7832',
      keyword: '#cc7832'
    }
  },
  {
    id: 'jetbrains-intellij-light',
    name: 'JetBrains IntelliJ Light',
    colors: {
      bg: '#ffffff',
      bgSecondary: '#f7f7f7',
      fg: '#000000',
      comment: '#808080',
      border: '#d7d7d7',
      selection: '#acc8d6',
      accent: '#6a8759',
      success: '#6a8759',
      warning: '#a7a733',
      error: '#cc7832',
      keyword: '#cc7832'
    }
  },
  {
    id: 'doom-one',
    name: 'Doom One',
    colors: {
      bg: '#282c34',
      bgSecondary: '#21252b',
      fg: '#bbc2cf',
      comment: '#5b6268',
      border: '#3f444a',
      selection: '#3e4451',
      accent: '#51afef',
      success: '#98be65',
      warning: '#ecbe7b',
      error: '#ff6c6b',
      keyword: '#c678dd'
    }
  },
  // 无障碍主题
  {
    id: 'high-contrast',
    name: 'High Contrast',
    colors: {
      bg: '#000000',
      bgSecondary: '#1a1a1a',
      fg: '#ffffff',
      comment: '#cccccc',
      border: '#ffffff',
      selection: '#333333',
      accent: '#00ffff',
      success: '#00ff00',
      warning: '#ffff00',
      error: '#ff0000',
      keyword: '#ff00ff'
    }
  },
  {
    id: 'colorblind-friendly',
    name: 'Colorblind Friendly',
    colors: {
      bg: '#1e1e1e',
      bgSecondary: '#252526',
      fg: '#d4d4d4',
      comment: '#6d6d6d',
      border: '#3c3c3c',
      selection: '#264f78',
      accent: '#569cd6', // 蓝色
      success: '#4ec9b0', // 青色
      warning: '#dcdcaa', // 黄色
      error: '#f44747', // 红色
      keyword: '#c586c0' // 紫色
    }
  }
]

// 当前主题ID
const currentThemeId = ref('onedark')

const customThemes = ref<Theme[]>([])

// 主题面板可见性
const themePanelVisible = ref(false)

function withAlpha(hexColor: string, alpha: number) {
  const color = hexColor.trim()
  if (!color.startsWith('#') || (color.length !== 7)) return hexColor
  const r = Number.parseInt(color.slice(1, 3), 16)
  const g = Number.parseInt(color.slice(3, 5), 16)
  const b = Number.parseInt(color.slice(5, 7), 16)
  if (Number.isNaN(r) || Number.isNaN(g) || Number.isNaN(b)) return hexColor
  const a = Math.max(0, Math.min(1, alpha))
  return `rgba(${r}, ${g}, ${b}, ${a})`
}

/**
 * 获取当前主题
 */
const mergedThemes = computed<Theme[]>(() => {
  const map = new Map<string, Theme>()
  for (const t of themes) {
    map.set(t.id, t)
  }
  for (const t of customThemes.value) {
    map.set(t.id, t)
  }
  return Array.from(map.values())
})

const currentTheme = computed(() => {
  return mergedThemes.value.find(t => t.id === currentThemeId.value) || mergedThemes.value[0] || themes[0]
})

async function refreshCustomThemes() {
  try {
    customThemes.value = await listThemes()
  } catch (_e) {
    customThemes.value = []
  }
}

async function upsertCustomTheme(theme: Theme) {
  const res = await upsertTheme(theme)
  customThemes.value = res
}

async function deleteCustomTheme(themeId: string) {
  const res = await deleteTheme(themeId)
  customThemes.value = res
}

/**
 * 应用主题到 CSS 变量
 */
function applyTheme(theme: Theme) {
  const root = document.documentElement
  const colors = theme.colors
  const uiBorder = withAlpha(colors.border, 0.25)
  
  // 基础主题变量
  root.style.setProperty('--theme-bg', colors.bg)
  root.style.setProperty('--theme-bg-secondary', colors.bgSecondary)
  root.style.setProperty('--theme-fg', colors.fg)
  root.style.setProperty('--theme-comment', colors.comment)
  root.style.setProperty('--theme-border', uiBorder)
  root.style.setProperty('--theme-selection', colors.selection)
  root.style.setProperty('--theme-accent', colors.accent)
  root.style.setProperty('--theme-success', colors.success)
  root.style.setProperty('--theme-warning', colors.warning)
  root.style.setProperty('--theme-error', colors.error)
  root.style.setProperty('--theme-keyword', colors.keyword)
  
  // HOI4 兼容变量
  root.style.setProperty('--hoi4-dark', colors.bg)
  root.style.setProperty('--hoi4-gray', colors.bgSecondary)
  root.style.setProperty('--hoi4-border', uiBorder)
  root.style.setProperty('--hoi4-accent', colors.accent)
  root.style.setProperty('--hoi4-text', colors.fg)
  root.style.setProperty('--hoi4-comment', colors.comment)
  
  // 设置页面专用变量
  root.style.setProperty('--settings-sidebar-bg', colors.bgSecondary)
  root.style.setProperty('--settings-sidebar-border', uiBorder)
  root.style.setProperty('--settings-sidebar-hover', colors.selection)
  root.style.setProperty('--settings-sidebar-active', colors.accent)
  root.style.setProperty('--settings-content-bg', colors.bg)
  root.style.setProperty('--settings-card-bg', colors.bgSecondary)
  root.style.setProperty('--settings-card-border', uiBorder)
  root.style.setProperty('--settings-card-hover', colors.selection)
  
  // 高级主题变量（用于更复杂的UI效果）
  root.style.setProperty('--theme-island-bg', colors.bgSecondary + 'dd')
  root.style.setProperty('--theme-island-shadow', '0 0 0 rgba(0, 0, 0, 0)')
  root.style.setProperty('--theme-island-border', `1px solid ${uiBorder}`)
  root.style.setProperty('--theme-fg-glass', colors.fg + '33')
  root.style.setProperty('--theme-island-hover-shadow', '0 0 0 rgba(0, 0, 0, 0)')
  root.style.setProperty('--theme-accent-island-bg', colors.accent + '14')
  root.style.setProperty('--theme-accent-glow', colors.accent + '2a')
  root.style.setProperty('--theme-section-bg', colors.bgSecondary + 'ee')
  root.style.setProperty('--theme-bg-hover', colors.selection + '33')

  root.style.setProperty('--theme-surface-1', colors.bg + 'f2')
  root.style.setProperty('--theme-surface-2', colors.bgSecondary + 'cc')
  root.style.setProperty('--theme-surface-3', colors.bgSecondary + 'ff')
}

/**
 * 设置主题
 */
async function setTheme(themeId: string, saveToSettings = true) {
  const theme = mergedThemes.value.find(t => t.id === themeId)
  if (!theme) return
  
  currentThemeId.value = themeId
  applyTheme(theme)
  
  // 保存到设置
  if (saveToSettings) {
    try {
      const result = await loadSettings()
      if (result.success && result.data) {
        const settings = result.data as Record<string, unknown>
        settings.theme = themeId
        await saveSettings(settings)
      }
    } catch (error) {
      console.error('保存主题设置失败:', error)
    }
  }
}

/**
 * 从设置加载主题
 */
async function loadThemeFromSettings(settings?: Record<string, unknown>) {
  try {
    await refreshCustomThemes()
    const loadedSettings = settings ?? await (async () => {
      const result = await loadSettings()
      if (!result.success || !result.data) return null
      return result.data as Record<string, unknown>
    })()

    if (loadedSettings) {
      const savedTheme = loadedSettings.theme as string
      if (savedTheme && mergedThemes.value.some(t => t.id === savedTheme)) {
        currentThemeId.value = savedTheme
        applyTheme(currentTheme.value)
      } else {
        // 默认主题
        applyTheme(mergedThemes.value[0] || themes[0])
      }
    } else {
      applyTheme(mergedThemes.value[0] || themes[0])
    }
  } catch (error) {
    console.error('加载主题设置失败:', error)
    applyTheme(mergedThemes.value[0] || themes[0])
  }
}

/**
 * 切换主题面板可见性
 */
function toggleThemePanel() {
  themePanelVisible.value = !themePanelVisible.value
}

/**
 * 关闭主题面板
 */
function closeThemePanel() {
  themePanelVisible.value = false
}

/**
 * 主题系统 Composable
 */
export function useTheme() {
  return {
    themes: mergedThemes,
    customThemes,
    currentThemeId,
    currentTheme,
    themePanelVisible,
    setTheme,
    loadThemeFromSettings,
    refreshCustomThemes,
    upsertCustomTheme,
    deleteCustomTheme,
    toggleThemePanel,
    closeThemePanel,
    applyTheme
  }
}
