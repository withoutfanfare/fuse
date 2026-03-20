import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { LinkedIssue } from '../types'

export function useLinkedIssues() {
  const linkedIssues = ref<LinkedIssue[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchLinkedIssues(prId: number) {
    loading.value = true
    error.value = null
    try {
      linkedIssues.value = await invoke<LinkedIssue[]>('get_linked_issues', { prId })
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  return { linkedIssues, loading, error, fetchLinkedIssues }
}
