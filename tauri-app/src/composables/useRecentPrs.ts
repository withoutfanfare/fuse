import { ref, watch } from 'vue'

const STORAGE_KEY = 'recent-prs'
const MAX_ENTRIES = 5

export interface RecentPrEntry {
  id: number
  number: number
  title: string
  repoFullName: string
}

/** Module-level state so it survives component remounts. */
const recentPrs = ref<RecentPrEntry[]>(loadFromStorage())

function loadFromStorage(): RecentPrEntry[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return []
    const parsed = JSON.parse(raw)
    if (Array.isArray(parsed)) return parsed.slice(0, MAX_ENTRIES)
  } catch {
    // Corrupted data — start fresh
  }
  return []
}

function persist() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(recentPrs.value))
}

watch(recentPrs, persist, { deep: true })

/**
 * Composable for tracking recently visited pull requests.
 * Maintains a capped list of the last 5 PRs visited, stored in localStorage.
 */
export function useRecentPrs() {
  /** Push a PR to the front of the recent list, deduplicating by id. */
  function push(entry: RecentPrEntry) {
    const filtered = recentPrs.value.filter(e => e.id !== entry.id)
    recentPrs.value = [entry, ...filtered].slice(0, MAX_ENTRIES)
  }

  return { recentPrs, push }
}
