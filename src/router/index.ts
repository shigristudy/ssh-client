import { createRouter, createWebHistory } from 'vue-router'
import HostsView from '../views/HostsView.vue'
import KeysView from '../views/KeysView.vue'
import SettingsView from '../views/SettingsView.vue'
import TerminalPage from '../pages/TerminalPage.vue'

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
      component: TerminalPage,
      props: true // Ensure props are passed correctly
    }
  ]
})

export default router