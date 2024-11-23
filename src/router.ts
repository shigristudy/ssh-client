
import { createRouter, createWebHistory } from 'vue-router'
import HomePage from './pages/HomePage.vue'
import HostsPage from './pages/HostsPage.vue'
import KeysPage from './pages/KeysPage.vue'
import SettingsPage from './pages/SettingsPage.vue'

const routes = [
  {
    path: '/',
    name: 'home',
    component: HomePage
  },
  {
    path: '/hosts',
    name: 'hosts',
    component: HostsPage
  },
  {
    path: '/keys',
    name: 'keys',
    component: KeysPage
  },
  {
    path: '/settings',
    name: 'settings',
    component: SettingsPage
  },
  {
    path: '/terminal/:id',
    name: 'terminal',
    component: () => import('./pages/TerminalPage.vue')
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router