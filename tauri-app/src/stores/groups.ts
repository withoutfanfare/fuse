import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { RepoGroup } from '../types'

export const useGroupsStore = defineStore('groups', () => {
  const groups = ref<RepoGroup[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchAll() {
    loading.value = true
    error.value = null
    try {
      groups.value = await invoke<RepoGroup[]>('list_groups')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function create(name: string, colour?: string): Promise<RepoGroup | null> {
    error.value = null
    try {
      const params: Record<string, unknown> = { name }
      if (colour !== undefined) params.colour = colour
      const group = await invoke<RepoGroup>('create_group', params)
      await fetchAll()
      return group
    } catch (e) {
      error.value = String(e)
      return null
    }
  }

  async function remove(id: number) {
    error.value = null
    try {
      await invoke('delete_group', { id })
      await fetchAll()
    } catch (e) {
      error.value = String(e)
    }
  }

  async function addRepo(groupId: number, repoId: number) {
    error.value = null
    try {
      await invoke('add_repo_to_group', { groupId, repoId })
      await fetchAll()
    } catch (e) {
      error.value = String(e)
    }
  }

  async function removeRepo(groupId: number, repoId: number) {
    error.value = null
    try {
      await invoke('remove_repo_from_group', { groupId, repoId })
      await fetchAll()
    } catch (e) {
      error.value = String(e)
    }
  }

  function getGroupsForRepo(repoId: number): RepoGroup[] {
    return groups.value.filter(g => g.repo_ids.includes(repoId))
  }

  return { groups, loading, error, fetchAll, create, remove, addRepo, removeRepo, getGroupsForRepo }
})
