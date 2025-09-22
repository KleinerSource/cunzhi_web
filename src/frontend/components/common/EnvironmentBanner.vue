<script setup lang="ts">
import { computed } from 'vue'
import { useApi } from '../../composables/useWebApi'

const api = useApi()

const environmentInfo = computed(() => {
  if (api.isWebMode()) {
    return {
      mode: 'Web',
      icon: 'i-carbon-cloud',
      color: 'info',
      description: '当前运行在 Web 模式下',
      features: ['跨平台访问', '无需安装', '实时同步'],
      envVar: 'CUNZHI_MODE=web'
    }
  } else {
    return {
      mode: 'Desktop',
      icon: 'i-carbon-desktop',
      color: 'success',
      description: '当前运行在桌面模式下',
      features: ['原生性能', '系统集成', '离线使用'],
      envVar: 'CUNZHI_MODE=desktop'
    }
  }
})
</script>

<template>
  <div class="environment-banner">
    <n-alert 
      :type="environmentInfo.color" 
      :show-icon="false"
      class="mb-4"
    >
      <template #header>
        <div class="flex items-center gap-2">
          <div :class="environmentInfo.icon" class="w-4 h-4" />
          <span class="font-medium">{{ environmentInfo.mode }} 模式</span>
        </div>
      </template>
      
      <div class="text-sm">
        <p class="mb-2">{{ environmentInfo.description }}</p>
        <div class="flex gap-2 flex-wrap items-center mb-2">
          <n-tag
            v-for="feature in environmentInfo.features"
            :key="feature"
            size="small"
            :type="environmentInfo.color"
          >
            {{ feature }}
          </n-tag>
        </div>
        <div class="text-xs opacity-75">
          环境变量: <code class="bg-black bg-opacity-20 px-1 rounded">{{ environmentInfo.envVar }}</code>
        </div>
      </div>
    </n-alert>
  </div>
</template>

<style scoped>
.environment-banner {
  /* 可以添加自定义样式 */
}
</style>
