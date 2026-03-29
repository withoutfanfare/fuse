import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { LabelSummary } from '../types'

/**
 * Composable for fetching label summaries and managing label-based quick filters.
 * Provides a list of all labels across open PRs with counts and colours,
 * plus selection state for filtering the PR list.
 */
export function useLabelFilter() {
  const labels = ref<LabelSummary[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  const selectedLabels = ref<Set<string>>(new Set())

  async function fetchLabels(repoId?: number) {
    loading.value = true
    error.value = null
    try {
      labels.value = await invoke<LabelSummary[]>('get_all_labels', {
        repoId: repoId ?? null,
      })
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  function toggleLabel(label: string) {
    const next = new Set(selectedLabels.value)
    if (next.has(label)) {
      next.delete(label)
    } else {
      next.add(label)
    }
    selectedLabels.value = next
  }

  function clearSelection() {
    selectedLabels.value = new Set()
  }

  const hasSelection = computed(() => selectedLabels.value.size > 0)

  return { labels, loading, error, selectedLabels, hasSelection, fetchLabels, toggleLabel, clearSelection }
}
