import { defineStore } from 'pinia'
import { Octokit } from '@octokit/rest'
import { useHostsStore } from './hosts'

interface Settings {
  defaultPort: number;
  defaultUsername: string;
  theme: 'light' | 'dark';
  githubToken: string;
  gistId: string;
  lastSync: number | null; // Change undefined to null
}

export const useSettingsStore = defineStore('settings', {
  state: (): Settings => ({
    defaultPort: 22,
    defaultUsername: '',
    theme: 'light',
    githubToken: localStorage.getItem('githubToken') || '',
    gistId: localStorage.getItem('gistId') || '',
    lastSync: null // Initialize as null instead of undefined
  }),

  actions: {
    async saveSettings() {
      localStorage.setItem('settings', JSON.stringify(this.$state))
      localStorage.setItem('githubToken', this.githubToken)
      localStorage.setItem('gistId', this.gistId)
      
      // Update HTML class for theme
      document.documentElement.classList.toggle('dark-mode', this.theme === 'dark')
    },

    async loadSettings() {
      const stored = localStorage.getItem('settings')
      if (stored) {
        const parsed = JSON.parse(stored)
        this.$state = parsed
      }
      
      // Set initial theme class
      document.documentElement.classList.toggle('dark-mode', this.theme === 'dark')
    },

    async testGithubConnection(token: string): Promise<boolean> {
      try {
        const octokit = new Octokit({ auth: token })
        await octokit.users.getAuthenticated()
        return true
      } catch (error) {
        console.error('GitHub connection test failed:', error)
        return false
      }
    },

    async createHostsGist(): Promise<string> {
      if (!this.githubToken) throw new Error('GitHub token not set')
      
      const octokit = new Octokit({ auth: this.githubToken })
      const response = await octokit.gists.create({
        description: 'SSH Client Hosts',
        public: false,
        files: {
          'hosts.json': {
            content: '[]'
          }
        }
      })
      
      if (!response.data.id) {
        throw new Error('Failed to create gist: ID is undefined')
      }
      return response.data.id
    },

    async updateGist(gistId: string, content: string): Promise<void> {
      if (!this.githubToken) throw new Error('GitHub token not set')
      
      const octokit = new Octokit({ auth: this.githubToken })
      await octokit.gists.update({
        gist_id: gistId,
        files: {
          'hosts.json': {
            content
          }
        }
      })
    },

    async fetchGist(gistId: string): Promise<string> {
      if (!this.githubToken) throw new Error('GitHub token not set')
      
      const octokit = new Octokit({ auth: this.githubToken })
      const response = await octokit.gists.get({ gist_id: gistId })
      return response.data.files?.['hosts.json']?.content || '[]'
    },

    async syncHosts() {
      if (!this.githubToken) throw new Error('GitHub token not set')
      const hostsStore = useHostsStore()

      try {
        const gistId = await this.getOrCreateGistId()
        await this.updateGist(gistId, JSON.stringify(hostsStore.hosts))
        this.lastSync = Date.now()
        await this.saveSettings()
        return true
      } catch (error) {
        console.error('Sync failed:', error)
        throw error
      }
    },

    async getOrCreateGistId(): Promise<string> {
      const octokit = new Octokit({ auth: this.githubToken })
      
      // Try to find existing gist
      const gists = await octokit.gists.list()
      const existingGist = gists.data.find(g => 
        g.description === 'SSH Client Hosts' && 
        g.files?.['hosts.json']
      )

      if (existingGist) return existingGist.id
      
      // Create new gist if none exists
      const response = await this.createHostsGist()
      return response
    }
  }
})
