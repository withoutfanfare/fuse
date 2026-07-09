import { ref, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

/**
 * Auto-saves review session state (files reviewed + notes) at a regular interval.
 * Attaches to a PR ID and periodically persists the current snapshot so progress
 * is not lost if the app crashes or the user navigates away.
 */
export function useSessionAutoSave(prId: number, intervalMs = 30_000) {
  const lastSavedAt = ref<string | null>(null)
  const saving = ref(false)
  const error = ref<string | null>(null)

  let timerId: ReturnType<typeof setInterval> | null = null
  let currentFiles: string[] = []
  let currentNotes: string | null = null
  let dirty = false

  /** Update the tracked state (call when files or notes change). */
  function track(filesReviewed: string[], sessionNotes: string | null) {
    currentFiles = filesReviewed
    currentNotes = sessionNotes
    dirty = true
  }

  /** Persist the current snapshot to the backend. */
  async function save() {
    if (!dirty) return
    saving.value = true
    error.value = null
    try {
      await invoke('save_session_snapshot', {
        prId,
        filesReviewed: currentFiles,
        sessionNotes: currentNotes,
      })
      dirty = false
      lastSavedAt.value = new Date().toISOString()
    } catch (e) {
      error.value = String(e)
    } finally {
      saving.value = false
    }
  }

  /** Start the auto-save interval. */
  function start() {
    if (timerId !== null) return
    timerId = setInterval(save, intervalMs)
  }

  /** Stop the auto-save interval and flush any pending changes. */
  async function stop() {
    if (timerId !== null) {
      clearInterval(timerId)
      timerId = null
    }
    await save()
  }

  onUnmounted(() => {
    if (timerId !== null) {
      clearInterval(timerId)
      timerId = null
    }
    // Best-effort final save (fire-and-forget)
    if (dirty) {
      save()
    }
  })

  return {
    lastSavedAt,
    saving,
    error,
    track,
    save,
    start,
    stop,
  }
}
