import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import VersionInfoSettings from '../../../components/settings/VersionInfoSettings.vue'

// Mock the checkForUpdates function
vi.mock('../../../utils/version', () => ({
  checkForUpdates: vi.fn()
}))

import { checkForUpdates } from '../../../utils/version'
const mockCheckForUpdates = checkForUpdates as import('vitest').MockedFunction<typeof checkForUpdates>

describe('VersionInfoSettings', () => {
  const defaultProps = {
    currentVersion: 'v0.2.8-dev',
    githubVersion: 'v0.2.8-dev',
    isCheckingUpdate: false
  }

  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('should render the component correctly', () => {
    const wrapper = mount(VersionInfoSettings, {
      props: defaultProps
    })

    expect(wrapper.exists()).toBe(true)
    expect(wrapper.classes()).toContain('border-2')
    expect(wrapper.classes()).toContain('border-hoi4-border')
    expect(wrapper.classes()).toContain('rounded-lg')
  })

  it('should display current and GitHub versions correctly', () => {
    const wrapper = mount(VersionInfoSettings, {
      props: defaultProps
    })

    // Check current version
    const versionElements = wrapper.findAll('.text-hoi4-text.font-semibold')
    expect(versionElements).toHaveLength(2)
    expect(versionElements[0].text()).toBe(defaultProps.currentVersion)
    expect(versionElements[1].text()).toBe(defaultProps.githubVersion)
  })

  it('should display "检查中..." text when isCheckingUpdate is true', () => {
    const wrapper = mount(VersionInfoSettings, {
      props: {
        ...defaultProps,
        isCheckingUpdate: true
      }
    })

    const buttonText = wrapper.find('button span').text()
    expect(buttonText).toBe('检查中...')
  })

  it('should disable the button when isCheckingUpdate is true', () => {
    const wrapper = mount(VersionInfoSettings, {
      props: {
        ...defaultProps,
        isCheckingUpdate: true
      }
    })

    const button = wrapper.find('button')
    expect(button.attributes('disabled')).toBeDefined()
    expect(button.classes()).toContain('opacity-50')
    expect(button.classes()).toContain('cursor-not-allowed')
  })

  it('should emit events when check update button is clicked', async () => {
    // Mock successful update check with no new version
    mockCheckForUpdates.mockResolvedValue({
      hasUpdate: false,
      latestVersion: 'v0.2.8-dev',
      releaseUrl: 'https://github.com/cyberlieflife/HOI4-Code-Studio/releases/latest'
    })

    const wrapper = mount(VersionInfoSettings, {
      props: defaultProps
    })

    const button = wrapper.find('button')
    await button.trigger('click')

    // Check that events are emitted
    expect(wrapper.emitted('update:isCheckingUpdate')).toHaveLength(2) // true then false
    expect(wrapper.emitted('update:isCheckingUpdate')?.[0]).toEqual([true])
    expect(wrapper.emitted('update:githubVersion')).toHaveLength(2) // "检查中..." then actual version
    expect(wrapper.emitted('update:githubVersion')?.[0]).toEqual(['检查中...'])
    expect(wrapper.emitted('status-message')).toHaveLength(1)
    expect(wrapper.emitted('status-message')?.[0]).toEqual(['当前已是最新版本'])
  })

  it('should handle successful check with update available', async () => {
    // Mock successful update check with new version
    mockCheckForUpdates.mockResolvedValue({
      hasUpdate: true,
      latestVersion: 'v0.3.0',
      releaseUrl: 'https://github.com/cyberlieflife/HOI4-Code-Studio/releases/v0.3.0'
    })

    const wrapper = mount(VersionInfoSettings, {
      props: defaultProps
    })

    const button = wrapper.find('button')
    await button.trigger('click')

    // Check that events are emitted correctly
    expect(wrapper.emitted('update:githubVersion')).toHaveLength(2)
    expect(wrapper.emitted('update:githubVersion')?.[1]).toEqual(['v0.3.0'])
    expect(wrapper.emitted('show-update-dialog')).toHaveLength(1)
    expect(wrapper.emitted('show-update-dialog')?.[0]).toEqual([{
      version: 'v0.3.0',
      url: 'https://github.com/cyberlieflife/HOI4-Code-Studio/releases/v0.3.0'
    }])
    // No status message for update available
    expect(wrapper.emitted('status-message')).toBeUndefined()
  })

  it('should handle check update failure', async () => {
    // Mock failed update check
    mockCheckForUpdates.mockResolvedValue({
      hasUpdate: false,
      error: 'Network error'
    })

    const wrapper = mount(VersionInfoSettings, {
      props: defaultProps
    })

    const button = wrapper.find('button')
    await button.trigger('click')

    // Check that events are emitted correctly
    expect(wrapper.emitted('update:githubVersion')).toHaveLength(2)
    expect(wrapper.emitted('update:githubVersion')?.[1]).toEqual(['检查失败'])
    expect(wrapper.emitted('status-message')).toHaveLength(1)
    expect(wrapper.emitted('status-message')?.[0]).toEqual(['检查更新失败: Network error'])
    expect(wrapper.emitted('show-update-dialog')).toBeUndefined()
  })

  it('should handle exception during check update', async () => {
    // Mock exception during update check
    mockCheckForUpdates.mockRejectedValue(new Error('Unexpected error'))

    const wrapper = mount(VersionInfoSettings, {
      props: defaultProps
    })

    const button = wrapper.find('button')
    await button.trigger('click')

    // Check that events are emitted correctly
    expect(wrapper.emitted('update:githubVersion')).toHaveLength(2)
    expect(wrapper.emitted('update:githubVersion')?.[1]).toEqual(['检查失败'])
    expect(wrapper.emitted('status-message')).toHaveLength(1)
    expect(wrapper.emitted('status-message')?.[0]).toEqual(['检查更新失败'])
    expect(wrapper.emitted('show-update-dialog')).toBeUndefined()
  })

  it('should handle no update available scenario', async () => {
    // Mock successful update check with no new version
    mockCheckForUpdates.mockResolvedValue({
      hasUpdate: false,
      latestVersion: 'v0.2.8-dev',
      releaseUrl: 'https://github.com/cyberlieflife/HOI4-Code-Studio/releases/latest'
    })

    const wrapper = mount(VersionInfoSettings, {
      props: defaultProps
    })

    const button = wrapper.find('button')
    await button.trigger('click')

    // Check that events are emitted correctly
    expect(wrapper.emitted('update:githubVersion')).toHaveLength(2)
    expect(wrapper.emitted('update:githubVersion')?.[1]).toEqual(['v0.2.8-dev'])
    expect(wrapper.emitted('status-message')).toHaveLength(1)
    expect(wrapper.emitted('status-message')?.[0]).toEqual(['当前已是最新版本'])
    expect(wrapper.emitted('show-update-dialog')).toBeUndefined()
  })

  it('should display loading state correctly during check', async () => {
    // Create a delayed promise to keep the loading state visible
    const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms))
    mockCheckForUpdates.mockImplementation(async () => {
      await delay(100)
      return {
        hasUpdate: false,
        latestVersion: 'v0.2.8-dev',
        releaseUrl: 'https://github.com/cyberlieflife/HOI4-Code-Studio/releases/latest'
      }
    })

    const wrapper = mount(VersionInfoSettings, {
      props: defaultProps
    })

    const button = wrapper.find('button')
    await button.trigger('click')

    // Check that loading state is set immediately
    expect(wrapper.emitted('update:isCheckingUpdate')?.[0]).toEqual([true])
    expect(wrapper.emitted('update:githubVersion')?.[0]).toEqual(['检查中...'])
  })

  it('should render correct button text based on isCheckingUpdate prop', async () => {
    const wrapper = mount(VersionInfoSettings, {
      props: defaultProps
    })

    // When not checking, button should say "检查更新"
    expect(wrapper.find('button span').text()).toBe('检查更新')

    // When checking, button should say "检查中..."
    wrapper.setProps({ isCheckingUpdate: true })
    await nextTick()
    expect(wrapper.find('button span').text()).toBe('检查中...')
  })

  it('should have the correct button styling when enabled and disabled', async () => {
    const wrapper = mount(VersionInfoSettings, {
      props: defaultProps
    })

    let button = wrapper.find('button')
    expect(button.classes()).toContain('btn-primary')
    expect(button.classes()).not.toContain('opacity-50')
    expect(button.classes()).not.toContain('cursor-not-allowed')

    // When disabled
    wrapper.setProps({ isCheckingUpdate: true })
    await nextTick()
    button = wrapper.find('button')
    expect(button.classes()).toContain('opacity-50')
    expect(button.classes()).toContain('cursor-not-allowed')
  })

  it('should handle different version formats correctly', () => {
    const testCases = [
      { currentVersion: 'v1.0.0', githubVersion: 'v1.0.0' },
      { currentVersion: 'v0.2.8-dev', githubVersion: 'v0.3.0' },
      { currentVersion: 'v0.1.0', githubVersion: 'v1.0.0-beta' }
    ]

    for (const testCase of testCases) {
      const wrapper = mount(VersionInfoSettings, {
        props: {
          ...defaultProps,
          currentVersion: testCase.currentVersion,
          githubVersion: testCase.githubVersion
        }
      })

      const versionElements = wrapper.findAll('.text-hoi4-text.font-semibold')
      expect(versionElements).toHaveLength(2)
      expect(versionElements[0].text()).toBe(testCase.currentVersion)
      expect(versionElements[1].text()).toBe(testCase.githubVersion)

      wrapper.unmount()
    }
  })

  it('should call checkForUpdates with correct parameters', async () => {
    mockCheckForUpdates.mockResolvedValue({
      hasUpdate: false,
      latestVersion: 'v0.2.8-dev',
      releaseUrl: 'https://github.com/cyberlieflife/HOI4-Code-Studio/releases/latest'
    })

    const wrapper = mount(VersionInfoSettings, {
      props: defaultProps
    })

    const button = wrapper.find('button')
    await button.trigger('click')

    // Check that checkForUpdates is called with correct parameters
    expect(mockCheckForUpdates).toHaveBeenCalledTimes(1)
    expect(mockCheckForUpdates).toHaveBeenCalledWith('v0.2.8-dev', '', false)
  })
})
