import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { SyncHealthStatus } from '../types'

/**
 * Composable for monitoring repository sync health.
 * Reports per-repo failure streaks, last successful sync, and surfaces
 * repos that may need attention.
 */
export function useSyncHealth() {
  const health = ref<SyncHealthStatus[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchHealth() {
    loading.value = true
    error.value = null
    try {
      health.value = await invoke<SyncHealthStatus[]>('get_sync_health')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  /** Repos with at least one consecutive failure. */
  const unhealthyRepos = computed(() =>
    health.value.filter((r) => r.consecutive_failures > 0)
  )

  /** True if any repo has failing syncs. */
  const hasIssues = computed(() => unhealthyRepos.value.length > 0)

  return { health, loading, error, unhealthyRepos, hasIssues, fetchHealth }
}
