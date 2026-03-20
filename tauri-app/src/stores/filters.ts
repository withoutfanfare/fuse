import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export type SortKey = 'risk' | 'updated' | 'age' | 'size'

const STORAGE_KEY = 'pr-review-filters'

export const useFiltersStore = defineStore('filters', () => {
  const saved = sessionStorage.getItem(STORAGE_KEY)
  const initial = saved ? JSON.parse(saved) : {}

  const filterRepoId = ref<number | null>(initial.filterRepoId ?? null)
  const filterState = ref<string>(initial.filterState ?? 'OPEN')
  const searchQuery = ref<string>(initial.searchQuery ?? '')
  const sortBy = ref<SortKey>(initial.sortBy ?? 'risk')
  const sortAsc = ref<boolean>(initial.sortAsc ?? false)

  function persist() {
    sessionStorage.setItem(STORAGE_KEY, JSON.stringify({
      filterRepoId: filterRepoId.value,
      filterState: filterState.value,
      searchQuery: searchQuery.value,
      sortBy: sortBy.value,
      sortAsc: sortAsc.value,
    }))
  }

  let persistTimer: ReturnType<typeof setTimeout> | null = null
  watch([filterRepoId, filterState, searchQuery, sortBy, sortAsc], () => {
    if (persistTimer) clearTimeout(persistTimer)
    persistTimer = setTimeout(persist, 80)
  }, { deep: true })

  function resetFilters() {
    filterRepoId.value = null
    filterState.value = 'OPEN'
    searchQuery.value = ''
    sortBy.value = 'risk'
    sortAsc.value = false
  }

  return {
    filterRepoId,
    filterState,
    searchQuery,
    sortBy,
    sortAsc,
    resetFilters,
  }
})
