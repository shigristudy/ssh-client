import { defineStore } from 'pinia'
import { Octokit } from '@octokit/rest'
import CryptoJS from 'crypto-js'
import { useSettingsStore } from './settings'

interface Host {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
  password?: string;
  group?: string;
  gistId?: string;
}

export const useHostsStore = defineStore('hosts', {
  state: () => ({
    hosts: [] as Host[],
    gistId: localStorage.getItem('gistId') || '',
    githubToken: localStorage.getItem('githubToken') || ''
  }),

  actions: {
    async loadHosts() {
      // First try to load from GitHub
      if (this.githubToken && this.gistId) {
        const octokit = new Octokit({ auth: this.githubToken })
        try {
          const response = await octokit.gists.get({ 
            gist_id: this.gistId 
          })
          const content = response.data.files?.['ssh-hosts.json']?.content
          if (content) {
            this.hosts = JSON.parse(content)
            // Update local storage
            localStorage.setItem('hosts', content)
            return
          }
        } catch (error) {
          console.error('Failed to load from GitHub:', error)
        }
      }

      // Fallback to local storage
      const localHosts = localStorage.getItem('hosts')
      if (localHosts) {
        this.hosts = JSON.parse(localHosts)
      }
    },

    async saveHosts() {
      // Save to local storage first
      localStorage.setItem('hosts', JSON.stringify(this.hosts))

      // Then try to save to GitHub if credentials exist
      if (this.githubToken && this.gistId) {
        const octokit = new Octokit({ auth: this.githubToken })
        try {
          await octokit.gists.update({
            gist_id: this.gistId,
            files: {
              'ssh-hosts.json': {
                content: JSON.stringify(this.hosts, null, 2)
              }
            }
          })
        } catch (error) {
          console.error('Failed to save to GitHub:', error)
          throw error
        }
      }
    },

    async addHost(host: Omit<Host, 'id'>) {
      const newHost = {
        ...host,
        id: crypto.randomUUID(),
        password: host.password 
          ? CryptoJS.AES.encrypt(host.password, 'secret-key').toString()
          : undefined
      };
      this.hosts.push(newHost);
      await this.saveHosts();
    },

    async deleteHost(id: string) {
      this.hosts = this.hosts.filter(h => h.id !== id);
      await this.saveHosts();
    },

    setGitHubCredentials(token: string, gistId: string) {
      this.githubToken = token
      this.gistId = gistId
      localStorage.setItem('githubToken', token)
      localStorage.setItem('gistId', gistId)
    },

    async initializeGist() {
      if (!this.githubToken) return;
      
      const octokit = new Octokit({ auth: this.githubToken });
      try {
        const response = await octokit.gists.create({
          description: 'SSH Hosts Configuration',
          public: false,
          files: {
            'ssh-hosts.json': {
              content: JSON.stringify([], null, 2)
            }
          }
        });
        this.gistId = response.data.id as string;
        localStorage.setItem('gistId', this.gistId);
      } catch (error) {
        console.error('Failed to create gist:', error);
        throw error;
      }
    },

    getHostPassword(host: Host): string | undefined {
      if (!host.password) return undefined;
      try {
        const bytes = CryptoJS.AES.decrypt(host.password, 'secret-key');
        return bytes.toString(CryptoJS.enc.Utf8);
      } catch (error) {
        console.error('Failed to decrypt password:', error);
        return undefined;
      }
    },

    async syncWithGithub(token: string, gistId: string) {
      const settingsStore = useSettingsStore()
      
      try {
        // Upload current hosts to GitHub
        await settingsStore.updateGist(gistId, JSON.stringify(this.hosts))
        
        // Fetch and merge remote hosts
        const remoteData = await settingsStore.fetchGist(gistId)
        const remoteHosts = JSON.parse(remoteData)
        
        // Merge hosts based on ID
        const merged = [...this.hosts]
        for (const remoteHost of remoteHosts) {
          if (!merged.find(h => h.id === remoteHost.id)) {
            merged.push(remoteHost)
          }
        }
        
        this.hosts = merged
        await this.saveHosts()
      } catch (error) {
        console.error('Sync failed:', error)
        throw error
      }
    }
  }
})