import { createRouter, createWebHistory } from 'vue-router'
import Hosts from './views/Hosts.vue'
import Keys from './views/Keys.vue'
import Settings from './views/Settings.vue'
import Terminal from './views/Terminal.vue'

const routes = [
  { path: '/', redirect: '/hosts' },
  { path: '/hosts', name: 'hosts', component: Hosts },
  { path: '/keys', name: 'keys', component: Keys },
  { path: '/settings', name: 'settings', component: Settings },
  { path: '/terminal/:id', name: 'terminal', component: Terminal }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router