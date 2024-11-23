import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import App from './App.vue'
import HostsView from './views/HostsView.vue'
import KeysView from './views/KeysView.vue'
import SettingsView from './views/SettingsView.vue'
import TerminalPage from './pages/TerminalPage.vue'  // Add this import
import { useSettingsStore } from './stores/settings'
import { useHostsStore } from './stores/hosts'
import {
  create,
  NButton,
  NCard,
  NConfigProvider,
  NDataTable,
  NForm,
  NFormItem,
  NIcon,
  NInput,
  NInputNumber,
  NLayout,
  NLayoutContent,
  NLayoutHeader,
  NLayoutSider,
  NMenu,
  NMessageProvider,
  NModal,
  NSelect,
  NSpace,
  NTabPane,
  NTabs
} from 'naive-ui'

// Create naive UI instance
const naive = create({
  components: [
    NButton,
    NCard,
    NConfigProvider,
    NDataTable,
    NForm,
    NFormItem,
    NIcon,
    NInput,
    NInputNumber,
    NLayout,
    NLayoutContent,
    NLayoutHeader,
    NLayoutSider,
    NMenu,
    NMessageProvider,
    NModal,
    NSelect,
    NSpace,
    NTabPane,
    NTabs
  ]
})

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/hosts'
    },
    {
      path: '/hosts',
      name: 'hosts',
      component: HostsView
    },
    {
      path: '/keys',
      name: 'keys',
      component: KeysView
    },
    {
      path: '/settings',
      name: 'settings',
      component: SettingsView
    },
    {
      path: '/terminal/:id',
      name: 'terminal',
      component: TerminalPage
    }
  ]
})

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)
app.use(naive)

// Initialize stores
const settingsStore = useSettingsStore()
const hostsStore = useHostsStore()

Promise.all([
  settingsStore.loadSettings(),
  hostsStore.loadHosts()
]).then(() => {
  app.mount('#app')
})
