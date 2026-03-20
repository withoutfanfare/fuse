import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Worktree } from '../types'

export function useGrove() {
  const worktrees = ref<Worktree[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function listWorktrees(repoName: string) {
    loading.value = true
    error.value = null
    try {
      worktrees.value = await invoke<Worktree[]>('grove_list_worktrees', { repoName })
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function addWorktree(repoName: string, branch: string, baseBranch?: string): Promise<boolean> {
    loading.value = true
    error.value = null
    try {
      const params: Record<string, unknown> = { repoName, branch }
      if (baseBranch !== undefined) params.baseBranch = baseBranch
      await invoke<string>('grove_add_worktree', params)
      await listWorktrees(repoName)
      return true
    } catch (e) {
      error.value = String(e)
      return false
    } finally {
      loading.value = false
    }
  }

  async function removeWorktree(repoName: string, branch: string): Promise<boolean> {
    loading.value = true
    error.value = null
    try {
      await invoke<string>('grove_remove_worktree', { repoName, branch })
      await listWorktrees(repoName)
      return true
    } catch (e) {
      error.value = String(e)
      return false
    } finally {
      loading.value = false
    }
  }

  return { worktrees, loading, error, listWorktrees, addWorktree, removeWorktree }
}
