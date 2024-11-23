<template>
  <n-layout class="layout">
    <n-layout-header bordered class="sticky-header">
      <n-menu 
        v-model:value="activeKey" 
        mode="horizontal" 
        :options="menuOptions" 
        @update:value="handleMenuClick" 
      />
    </n-layout-header>
    <n-layout-content class="content">
      <router-view />
    </n-layout-content>
  </n-layout>
</template>

<script setup lang="ts">
import { Component, h, computed } from 'vue'
import { useRouter } from 'vue-router'
import { 
  NIcon, 
  useMessage, 
  NLayout, 
  NLayoutSider, 
  NLayoutHeader, 
  NLayoutContent,
  NMenu,
  NButton,
  type MenuOption
} from 'naive-ui'
import { DesktopOutline, KeyOutline, SettingsOutline, Terminal, Close as CloseIcon } from '@vicons/ionicons5'
import { useHostsStore } from '../../stores/hosts'
import { useTerminalsStore } from '../../stores/terminals'

const router = useRouter()
const hostsStore = useHostsStore()
const message = useMessage()
const terminalsStore = useTerminalsStore()

const menuOptions = computed<MenuOption[]>(() => {
  const baseOptions: MenuOption[] = [
    {
      label: 'Hosts',
      key: 'hosts',
      icon: renderIcon(DesktopOutline)
    },
    {
      label: 'Settings',
      key: 'settings',
      icon: renderIcon(SettingsOutline)
    }
  ]

  // Only add active terminals to menu
  const activeTerminals = terminalsStore.terminals.map(term => ({
    label: () => h('div', { 
      style: 'display: flex; align-items: center; justify-content: space-between; width: 100%' 
    }, [
      h('span', term.name),
      h(NButton, {
        size: 'tiny',
        quaternary: true,
        onClick: (e: Event) => {
          e.stopPropagation()
          handleTerminalClose(term.id)
        }
      }, { icon: () => h(CloseIcon) })
    ]),
    key: `terminal-${term.id}`,
    icon: renderIcon(Terminal)
  }))

  return [...baseOptions, ...activeTerminals]
})

const activeKey = computed(() => {
  const route = router.currentRoute.value
  if (route.name === 'terminal') {
    return `terminal-${route.params.id}`
  }
  return route.name as string
})

function renderIcon(icon: Component) {
  return () => h(NIcon, null, { default: () => h(icon) })
}

function handleMenuClick(key: string) {
  if (key.startsWith('terminal-')) {
    const id = key.replace('terminal-', '')
    router.push({ name: 'terminal', params: { id } })
  } else {
    router.push({ name: key })
  }
}

// Add this function to handle terminal closure
async function handleTerminalClose(id: string) {
  try {
    const terminal = terminalsStore.getTerminal(id)
    if (!terminal) return

    if (router.currentRoute.value.name === 'terminal' && 
        router.currentRoute.value.params.id === id) {
      await router.push({ name: 'hosts' })
    }
    
    await terminalsStore.closeTerminal(id)
    message.success('Terminal closed')
  } catch (error) {
    console.error('Failed to close terminal:', error)
    message.error('Failed to close terminal')
  }
}
</script>

<style scoped>
.layout {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.sticky-header {
  position: sticky;
  top: 0;
  z-index: 100;
  background: #fff;
}

.content {
  flex: 1;
  overflow: hidden;
}

:deep(.n-layout-header) {
  height: 64px;
  padding: 0;
}

:deep(.n-layout-content) {
  display: flex;
  flex-direction: column;
  height: 100%;
}

:deep(.n-button) {
  opacity: 0.7;
}
:deep(.n-button:hover) {
  opacity: 1;
}
</style>