<template>
  <n-config-provider :theme="theme">
    <n-message-provider>
      <div class="app-container" :class="{ 'dark-mode': isDarkMode }">
        <n-layout has-sider>
          <n-layout-sider
            bordered
            collapse-mode="width"
            :collapsed-width="64"
            :width="240"
            :native-scrollbar="false"
            :collapsed="collapsed"
            @update:collapsed="handleCollapse"
            class="sidebar"
          >
            <n-menu
              :options="menuOptions"
              :value="activeKey"
              :collapsed="collapsed"
              @update:value="handleMenuSelect"
              class="menu"
            />
          </n-layout-sider>
          <n-layout-content class="main-content">
            <router-view />
          </n-layout-content>
        </n-layout>
      </div>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { h, computed, ref, Component } from 'vue'
import { useRouter } from 'vue-router'

import { 
  NConfigProvider, 
  NMessageProvider, 
  NLayout, 
  NLayoutSider,
  NLayoutContent,
  NMenu,
  NIcon,
  darkTheme,
  type MenuOption, 
  NButton,
  type MenuGroupOption,
  type MenuDividerOption,
  useMessage  // Add this import
} from 'naive-ui'
import { 
  DesktopOutline, 
  KeyOutline, 
  SettingsOutline, 
  Terminal as TerminalIcon, 
  Close as CloseIcon 
} from '@vicons/ionicons5'
import { useSettingsStore } from './stores/settings'
import { useTerminalsStore } from './stores/terminals'

const router = useRouter()
const settingsStore = useSettingsStore()
const terminalsStore = useTerminalsStore()
const collapsed = ref(false)

const handleCollapse = (value: boolean) => {
  collapsed.value = value
}
function renderIcon(icon: Component) {
  return () => h(NIcon, null, { default: () => h(icon) })
}
const menuOptions = computed(() => {
  const baseOptions: MenuOption[] = [
    {
      label: 'Hosts',
      key: 'hosts',
      icon: renderIcon(DesktopOutline)
    },
    {
      label: 'Keys',
      key: 'keys',
      icon: renderIcon(KeyOutline)
    },
    {
      label: 'Settings',
      key: 'settings',
      icon: renderIcon(SettingsOutline)
    }
  ]

  // Add active terminals as menu items if there are any
  if (terminalsStore.terminals.length > 0) {
    baseOptions.push({
      type: 'divider',
      key: 'divider'
    } as MenuDividerOption)

    baseOptions.push({
      type: 'group',
      label: 'Active Sessions',
      key: 'sessions',
      children: terminalsStore.terminals.map(term => ({
        label: () => h('div', { 
          class: 'session-item',
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
        icon: renderIcon(TerminalIcon),
        class: 'active-session-item'
      }))
    } as MenuGroupOption)
  }

  return baseOptions
})

async function handleTerminalClose(id: string) {
  try {
    const terminal = terminalsStore.getTerminal(id)
    if (!terminal) return

    if (router.currentRoute.value.name === 'terminal' && 
        router.currentRoute.value.params.id === id) {
      await router.push({ name: 'hosts' })
    }
    
    await terminalsStore.closeTerminal(id)
    console.error('Terminal closed successfully')
    // message.success('Terminal closed successfully')
  } catch (error) {
    console.error('Failed to close terminal:', error)
    // message.error('Failed to close terminal')
  }
}

const activeKey = computed(() => {
  return router.currentRoute.value.name as string
})

function handleMenuSelect(key: string) {
  if (key.startsWith('terminal-')) {
    const terminalId = key.replace('terminal-', '')
    router.push({ name: 'terminal', params: { id: terminalId } })
  } else {
    router.push({ name: key })
  }
}

const isDarkMode = computed(() => settingsStore.theme === 'dark')
const theme = computed(() => isDarkMode.value ? darkTheme : null)
</script>

<style>
/* Reset styles - use non-scoped style */
html, body {
  margin: 0;
  padding: 0;
  height: 100vh;
  width: 100vw;
}

#app, .app-container {
  height: 100vh;
  width: 100vw;
  background: white;
}

html, body {
  margin: 0;
  padding: 0;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}

.app-container {
  height: 100vh;
  width: 100vw;
  background: white;
}

.container {
  display: flex;
  height: 100vh;
}

.sidebar {
  width: 300px;
  padding: 1rem;
  border-right: 1px solid #eee;
  background: #f0f0f0;
}

.main-content {
  flex: 1;
  padding: 1rem;
}

.connection-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

input {
  padding: 0.5rem;
  border: 1px solid #ddd;
  border-radius: 4px;
}

button {
  padding: 0.5rem;
  background: #00bd7e;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

button:hover {
  background: #00a36d;
}

.error-message {
  color: red;
  margin-top: 1rem;
}

.app-container {
  height: 100vh;
  width: 100vw;
}

.sidebar {
  background: #fff;
  border-right: 1px solid #eee;
}

.main-content {
  padding: 20px;
  height: 100vh;
  overflow: auto;
}

.terminal-view {
  padding: 0;
  overflow: hidden;
}

.dark-mode .sidebar {
  background: #18181c;
  border-color: #333;
}

.dark-mode .main-content {
  background: #18181c;
}

.dark-mode .n-menu {
  background-color: #18181c;
}

.dark-mode .n-menu-item:not(.n-menu-item--selected):hover {
  background-color: #252529;
}

.dark-mode .n-menu-item--selected {
  background-color: #2b2b2f;
}

.n-menu-item--selected {
  background-color: #e6ffe6 !important;
  color: #00a06d !important;
}

.dark-mode .n-menu-item--selected {
  background-color: #1a2b1a !important;
  color: #00bd7e !important;
}

.active-session-item {
  margin: 2px 8px !important;
  border-radius: 4px !important;
}

.active-session-item:hover {
  background-color: #f0fff0 !important;
  color: #00a06d !important;
}

.dark-mode .active-session-item:hover {
  background-color: #182218 !important;
  color: #00bd7e !important;
}

.session-item {
  font-size: 0.95em;
  font-weight: 500;
  padding: 4px 0;
}

.n-menu-item-content {
  padding: 4px 12px !important;
}

.n-menu-item-content__icon {
  margin-right: 8px !important;
}

.n-menu-item-group-title {
  font-size: 0.85em;
  color: #666;
  padding: 8px 12px 4px !important;
}

.dark-mode .n-menu-item-group-title {
  color: #888;
}

.menu {
  padding-left: 0 !important; /* Remove extra spacing from left side of menu */
}
.n-menu-item-group-title{
  padding-left: 14px !important; /* Add padding to group titles */
}
.n-menu-item-content{
  padding-left: 14px !important; /* Remove extra padding from left side of menu item */
}
</style>