import { ref, watch, onMounted } from 'vue'
import { useSettingsStore } from '../stores/settings'

export type DensityMode = 'comfortable' | 'compact'

const STORAGE_KEY = 'fuse-density-mode'

/**
 * Composable managing density mode (comfortable / compact).
 * Applies `.density-compact` to `#app` and persists the choice
 * via the settings store with a localStorage fallback.
 */
export function useDensity() {
  const store = useSettingsStore()
  const density = ref<DensityMode>('comfortable')

  function applyDensity(mode: DensityMode) {
    const root = document.getElementById('app')
    if (!root) return
    if (mode === 'compact') {
      root.classList.add('density-compact')
    } else {
      root.classList.remove('density-compact')
    }
  }

  function setDensity(mode: DensityMode) {
    density.value = mode
    applyDensity(mode)
    localStorage.setItem(STORAGE_KEY, mode)
    store.updateSetting('density_mode', mode).catch(() => {
      /* Silently fall back to localStorage if backend unavailable */
    })
  }

  function toggle() {
    setDensity(density.value === 'comfortable' ? 'compact' : 'comfortable')
  }

  /** Hydrate from settings store or localStorage on mount */
  onMounted(async () => {
    await store.fetchSettings()
    const stored =
      (store.settings.density_mode as DensityMode) ||
      (localStorage.getItem(STORAGE_KEY) as DensityMode) ||
      'comfortable'
    density.value = stored
    applyDensity(stored)
  })

  watch(density, applyDensity)

  return { density, setDensity, toggle }
}
