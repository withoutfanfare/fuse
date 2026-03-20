import { ref, onMounted, onUnmounted, computed } from 'vue'

/**
 * Composable for detecting offline status and showing cached data indicators.
 *
 * Listens to browser online/offline events and provides reactive state
 * for the UI to show stale data banners and disable network-dependent features.
 */
export function useOfflineMode() {
  const isOnline = ref(navigator.onLine)
  const lastSyncTimestamp = ref<string | null>(null)

  function handleOnline() {
    isOnline.value = true
  }

  function handleOffline() {
    isOnline.value = false
  }

  onMounted(() => {
    window.addEventListener('online', handleOnline)
    window.addEventListener('offline', handleOffline)
  })

  onUnmounted(() => {
    window.removeEventListener('online', handleOnline)
    window.removeEventListener('offline', handleOffline)
  })

  /** Update the last sync timestamp (called after successful sync). */
  function recordSync() {
    lastSyncTimestamp.value = new Date().toISOString()
  }

  /** Human-readable time since last sync. */
  const timeSinceSync = computed(() => {
    if (!lastSyncTimestamp.value) return null
    const diffMs = Date.now() - new Date(lastSyncTimestamp.value).getTime()
    const diffMin = Math.floor(diffMs / 60000)
    if (diffMin < 1) return 'just now'
    if (diffMin < 60) return `${diffMin}m ago`
    const diffHr = Math.floor(diffMin / 60)
    if (diffHr < 24) return `${diffHr}h ago`
    const diffDays = Math.floor(diffHr / 24)
    return `${diffDays}d ago`
  })

  /** Whether the app is showing potentially stale data. */
  const isStale = computed(() => {
    if (!lastSyncTimestamp.value) return false
    const diffMs = Date.now() - new Date(lastSyncTimestamp.value).getTime()
    // Consider data stale after 10 minutes
    return diffMs > 10 * 60 * 1000
  })

  /**
   * Compute a per-PR staleness indicator based on its last_synced_at field.
   */
  function prTimeSinceSync(lastSyncedAt: string): string {
    const diffMs = Date.now() - new Date(lastSyncedAt).getTime()
    const diffMin = Math.floor(diffMs / 60000)
    if (diffMin < 1) return 'just now'
    if (diffMin < 60) return `${diffMin}m ago`
    const diffHr = Math.floor(diffMin / 60)
    if (diffHr < 24) return `${diffHr}h ago`
    const diffDays = Math.floor(diffHr / 24)
    return `${diffDays}d ago`
  }

  return {
    isOnline,
    lastSyncTimestamp,
    timeSinceSync,
    isStale,
    recordSync,
    prTimeSinceSync,
  }
}
