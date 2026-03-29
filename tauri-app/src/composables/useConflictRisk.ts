import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ConflictRiskEntry } from '../types'

/**
 * Composable for detecting file-level overlap between concurrent open PRs.
 *
 * Calls `detect_conflict_risks` to compare changed file lists across all
 * open PRs targeting the same base branch, identifying pairs that may
 * cause merge conflicts if merged in sequence.
 */
export function useConflictRisk() {
  const risks = ref<ConflictRiskEntry[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchRisks() {
    loading.value = true
    error.value = null
    try {
      risks.value = await invoke<ConflictRiskEntry[]>('detect_conflict_risks')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  /** Get conflict risk entries for a specific PR. */
  function risksForPr(prId: number): ConflictRiskEntry[] {
    return risks.value.filter((r) => r.pr_id === prId)
  }

  /** Total number of unique PR pairs with file overlap. */
  const pairCount = computed(() => {
    const seen = new Set<string>()
    for (const r of risks.value) {
      const key = [Math.min(r.pr_id, r.other_pr_id), Math.max(r.pr_id, r.other_pr_id)].join('-')
      seen.add(key)
    }
    return seen.size
  })

  return { risks, loading, error, fetchRisks, risksForPr, pairCount }
}
