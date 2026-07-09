import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { StaleReviewItem } from '../types'

/**
 * Composable for fetching stale review-requested PRs with escalation levels.
 * Used by the dashboard to show "needs your attention" items and by PRTable
 * to display stale review badges.
 */
export function useStaleReviews() {
  const items = ref<StaleReviewItem[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchStaleReviews() {
    loading.value = true
    error.value = null
    try {
      items.value = await invoke<StaleReviewItem[]>('get_stale_review_requests')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  /** Map of pr_id → StaleReviewItem for quick lookups in table rows. */
  const stalePrMap = computed(() => {
    const map = new Map<number, StaleReviewItem>()
    for (const item of items.value) {
      map.set(item.pr.id, item)
    }
    return map
  })

  /** Items needing attention: stale reviews without local progress, sorted by urgency. */
  const attentionItems = computed(() =>
    items.value
      .filter(i => !i.hasLocalProgress)
      .sort((a, b) => b.escalationLevel - a.escalationLevel || b.hoursWaiting - a.hoursWaiting)
  )

  const totalStaleCount = computed(() => items.value.length)
  const urgentCount = computed(() => items.value.filter(i => i.escalationLevel >= 2).length)

  return {
    items,
    loading,
    error,
    stalePrMap,
    attentionItems,
    totalStaleCount,
    urgentCount,
    fetchStaleReviews,
  }
}
