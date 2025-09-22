import { invoke } from '@tauri-apps/api/core'

/**
 * 检测运行环境
 */
export function isWebMode(): boolean {
  return !window.__TAURI__
}

/**
 * Web API 客户端
 */
class WebApiClient {
  private baseUrl: string

  constructor() {
    this.baseUrl = window.location.origin
  }

  async get(endpoint: string): Promise<any> {
    const response = await fetch(`${this.baseUrl}${endpoint}`)
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    }
    return response.json()
  }

  async post(endpoint: string, data?: any): Promise<any> {
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: data ? JSON.stringify(data) : undefined,
    })
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    }
    return response.json()
  }
}

const webApiClient = new WebApiClient()

/**
 * 统一的 API 调用接口
 * 自动检测环境并使用相应的调用方式
 */
export function useApi() {
  const isWeb = isWebMode()

  return {
    // 应用信息
    async getAppInfo(): Promise<string> {
      if (isWeb) {
        const result = await webApiClient.get('/api/app/info')
        return `${result.name} v${result.version} (${result.mode})`
      } else {
        return invoke('get_app_info')
      }
    },

    async getCurrentVersion(): Promise<string> {
      if (isWeb) {
        const result = await webApiClient.get('/api/app/version')
        return result.version
      } else {
        return invoke('get_current_version')
      }
    },

    // 主题设置
    async getTheme(): Promise<string> {
      if (isWeb) {
        const result = await webApiClient.get('/api/theme')
        return result.theme
      } else {
        return invoke('get_theme')
      }
    },

    async setTheme(theme: string): Promise<void> {
      if (isWeb) {
        await webApiClient.post('/api/theme', { theme })
      } else {
        await invoke('set_theme', { theme })
      }
    },

    // 窗口设置
    async getAlwaysOnTop(): Promise<boolean> {
      if (isWeb) {
        const result = await webApiClient.get('/api/window/always-on-top')
        return result.always_on_top
      } else {
        return invoke('get_always_on_top')
      }
    },

    async setAlwaysOnTop(alwaysOnTop: boolean): Promise<void> {
      if (isWeb) {
        await webApiClient.post('/api/window/always-on-top', { always_on_top: alwaysOnTop })
      } else {
        await invoke('set_always_on_top', { alwaysOnTop })
      }
    },

    // 音频设置
    async getAudioNotificationEnabled(): Promise<boolean> {
      if (isWeb) {
        const result = await webApiClient.get('/api/audio/enabled')
        return result.enabled
      } else {
        return invoke('get_audio_notification_enabled')
      }
    },

    async setAudioNotificationEnabled(enabled: boolean): Promise<void> {
      if (isWeb) {
        await webApiClient.post('/api/audio/enabled', { enabled })
      } else {
        await invoke('set_audio_notification_enabled', { enabled })
      }
    },

    async getAudioUrl(): Promise<string> {
      if (isWeb) {
        const result = await webApiClient.get('/api/audio/url')
        return result.url
      } else {
        return invoke('get_audio_url')
      }
    },

    async setAudioUrl(url: string): Promise<void> {
      if (isWeb) {
        await webApiClient.post('/api/audio/url', { url })
      } else {
        await invoke('set_audio_url', { url })
      }
    },

    // 配置管理
    async getConfig(): Promise<any> {
      if (isWeb) {
        return webApiClient.get('/api/config')
      } else {
        // Tauri 模式下可能需要分别获取各个配置项
        return {
          theme: await this.getTheme(),
          always_on_top: await this.getAlwaysOnTop(),
          audio_enabled: await this.getAudioNotificationEnabled(),
          audio_url: await this.getAudioUrl(),
        }
      }
    },

    async updateConfig(config: any): Promise<void> {
      if (isWeb) {
        await webApiClient.post('/api/config', { config })
      } else {
        // Tauri 模式下需要分别设置各个配置项
        if (config.theme) await this.setTheme(config.theme)
        if (typeof config.always_on_top === 'boolean') await this.setAlwaysOnTop(config.always_on_top)
        if (typeof config.audio_enabled === 'boolean') await this.setAudioNotificationEnabled(config.audio_enabled)
        if (config.audio_url) await this.setAudioUrl(config.audio_url)
      }
    },

    // MCP 相关
    async sendMcpResponse(response: any): Promise<void> {
      if (isWeb) {
        await webApiClient.post('/api/mcp/response', response)
      } else {
        await invoke('send_mcp_response', { response })
      }
    },

    // Telegram 配置
    async getTelegramConfig(): Promise<any> {
      if (isWeb) {
        return webApiClient.get('/api/telegram/config')
      } else {
        return invoke('get_telegram_config')
      }
    },

    async setTelegramConfig(config: any): Promise<void> {
      if (isWeb) {
        await webApiClient.post('/api/telegram/config', { config })
      } else {
        await invoke('set_telegram_config', { config })
      }
    },

    // 环境检测
    isWebMode: () => isWeb,
    isTauriMode: () => !isWeb,
  }
}
