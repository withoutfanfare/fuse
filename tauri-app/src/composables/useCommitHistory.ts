import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { CommitInfo } from '../types'

/**
 * Composable for fetching and managing commit history for a pull request.
 * Retrieves individual commits via the `gh` CLI backend command.
 */
export function useCommitHistory() {
  const commits = ref<CommitInfo[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchCommits(prId: number) {
    loading.value = true
    error.value = null
    try {
      commits.value = await invoke<CommitInfo[]>('get_pr_commits', { prId })
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  return { commits, loading, error, fetchCommits }
}
