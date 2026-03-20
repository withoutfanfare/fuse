import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Repository } from '../types'

export const useRepositoriesStore = defineStore('repositories', () => {
  const repos = ref<Repository[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchAll() {
    loading.value = true
    error.value = null
    try {
      repos.value = await invoke<Repository[]>('list_repositories')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function add(owner: string, name: string, defaultBranch?: string) {
    error.value = null
    try {
      const params: Record<string, unknown> = { owner, name }
      if (defaultBranch !== undefined) params.defaultBranch = defaultBranch
      await invoke<Repository>('add_repository', params)
      await fetchAll()
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function remove(id: number) {
    error.value = null
    try {
      await invoke('remove_repository', { id })
      await fetchAll()
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function updateBranch(id: number, defaultBranch: string) {
    error.value = null
    try {
      await invoke<Repository>('update_repository_branch', { id, defaultBranch })
      await fetchAll()
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  return { repos, loading, error, fetchAll, add, remove, updateBranch }
})
