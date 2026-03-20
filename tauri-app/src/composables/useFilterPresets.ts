import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { FilterPreset, FilterPresetConfig } from '../types'
import { useFiltersStore } from '../stores/filters'

/**
 * Composable for managing filter presets — save, load, apply, and delete
 * reusable filter combinations.
 */
export function useFilterPresets() {
  const presets = ref<FilterPreset[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchPresets() {
    loading.value = true
    error.value = null
    try {
      presets.value = await invoke<FilterPreset[]>('list_filter_presets')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  /**
   * Apply a preset's filter configuration to the active filters store.
   */
  function applyPreset(preset: FilterPreset) {
    const filters = useFiltersStore()
    const config = preset.filter_config as FilterPresetConfig

    if (config.repoId !== undefined) filters.filterRepoId = config.repoId ?? null
    if (config.state !== undefined) filters.filterState = config.state
    if (config.searchQuery !== undefined) filters.searchQuery = config.searchQuery
    if (config.sortBy !== undefined) filters.sortBy = config.sortBy as 'risk' | 'updated' | 'age' | 'size'
    if (config.sortAsc !== undefined) filters.sortAsc = config.sortAsc
  }

  /**
   * Save the current filter state as a new named preset.
   */
  async function saveCurrentAsPreset(name: string): Promise<FilterPreset | null> {
    error.value = null
    const filters = useFiltersStore()
    const config: FilterPresetConfig = {
      repoId: filters.filterRepoId,
      state: filters.filterState,
      searchQuery: filters.searchQuery,
      sortBy: filters.sortBy,
      sortAsc: filters.sortAsc,
    }
    try {
      const preset = await invoke<FilterPreset>('create_filter_preset', {
        name,
        filterConfig: config,
      })
      await fetchPresets()
      return preset
    } catch (e) {
      error.value = String(e)
      return null
    }
  }

  async function deletePreset(id: number) {
    error.value = null
    try {
      await invoke('delete_filter_preset', { id })
      await fetchPresets()
    } catch (e) {
      error.value = String(e)
    }
  }

  async function renamePreset(id: number, name: string) {
    error.value = null
    try {
      await invoke('rename_filter_preset', { id, name })
      await fetchPresets()
    } catch (e) {
      error.value = String(e)
    }
  }

  return {
    presets,
    loading,
    error,
    fetchPresets,
    applyPreset,
    saveCurrentAsPreset,
    deletePreset,
    renamePreset,
  }
}
