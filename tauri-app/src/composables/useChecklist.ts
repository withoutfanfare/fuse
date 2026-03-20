import { ref, watch, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

/**
 * Composable for persisting review checklist state per PR.
 * Fetches saved state on init and debounce-saves on every change.
 */
export function useChecklist(prId: number) {
  const checkedRules = ref<Record<number, boolean>>({})
  const loaded = ref(false)
  let saveTimer: ReturnType<typeof setTimeout> | null = null

  /** Fetch the persisted checklist state from the backend. */
  async function load() {
    try {
      const json = await invoke<string>('get_checklist_state', { prId })
      const parsed = JSON.parse(json)
      if (parsed && typeof parsed === 'object') {
        checkedRules.value = parsed
      }
    } catch {
      // No saved state yet — start with empty object
      checkedRules.value = {}
    }
    loaded.value = true
  }

  /** Persist the current checklist state to the backend (debounced). */
  function scheduleSave() {
    if (saveTimer) clearTimeout(saveTimer)
    saveTimer = setTimeout(async () => {
      try {
        await invoke('save_checklist_state', {
          prId,
          stateJson: JSON.stringify(checkedRules.value),
        })
      } catch {
        // Silently swallow save errors — the state remains in memory
      }
    }, 500)
  }

  // Watch for changes after initial load and debounce-save
  watch(checkedRules, () => {
    if (loaded.value) {
      scheduleSave()
    }
  }, { deep: true })

  onUnmounted(() => {
    if (saveTimer) {
      clearTimeout(saveTimer)
      saveTimer = null
    }
  })

  return { checkedRules, loaded, load }
}
