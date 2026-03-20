import { ref, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { usePullRequestsStore } from '../stores/pullRequests'
import { useToastStore } from '../stores/toast'
import type { SyncResult } from '../types'

/**
 * Composable that manages the background auto-sync polling loop.
 *
 * On mount it listens for `sync-completed` and `sync-error` events emitted by
 * the Rust backend polling loop. When a sync completes the pull request store
 * is refreshed. Errors are surfaced via toast notifications so the user can
 * investigate. Native notifications are handled entirely by the Rust backend
 * (polling.rs).
 */
export function useAutoSync() {
  const isPolling = ref(true)
  const syncing = ref(false)
  const prStore = usePullRequestsStore()
  const toast = useToastStore()

  let unlisten: (() => void) | null = null
  let unlistenStarted: (() => void) | null = null
  let unlistenError: (() => void) | null = null

  onMounted(async () => {
    unlistenStarted = await listen('sync-started', () => {
      syncing.value = true
    })

    unlisten = await listen<SyncResult[]>('sync-completed', async (event) => {
      syncing.value = false

      // Surface per-repo sync errors from background polling
      const results = event.payload
      if (results) {
        const failures = results.filter(r => r.error)
        for (const f of failures) {
          toast.addToast('error', `Sync failed: ${f.repo_name}`, f.error ?? undefined, 8000)
        }
      }

      // Refresh the store data after a background sync
      await Promise.all([prStore.fetchAll(), prStore.fetchStats()])
    })

    unlistenError = await listen<string>('sync-error', (event) => {
      syncing.value = false
      toast.addToast('error', 'Background sync failed', event.payload, 8000)
    })

    await invoke('start_polling')
  })

  onUnmounted(() => {
    unlisten?.()
    unlistenStarted?.()
    unlistenError?.()
  })

  const togglePolling = async () => {
    isPolling.value = !isPolling.value
    await invoke(isPolling.value ? 'start_polling' : 'stop_polling')
  }

  const updateInterval = async (seconds: number) => {
    await invoke('update_poll_interval', { seconds })
  }

  return { isPolling, syncing, togglePolling, updateInterval }
}
