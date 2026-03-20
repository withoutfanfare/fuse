import { ref, type Ref } from 'vue'

/** A single entry in the cache with its stored value and timestamp. */
interface CacheEntry<T> {
  value: T
  fetchedAt: number
}

/** Module-level map so cached data survives component remounts. */
const cacheStore = new Map<string, CacheEntry<unknown>>()

/** Default time-to-live in milliseconds. */
const DEFAULT_TTL_MS = 60_000

/** Maximum number of entries the cache may hold. */
const MAX_CACHE_SIZE = 50

/** Remove all entries whose TTL has expired. */
function clearExpired(ttlMs = DEFAULT_TTL_MS) {
  const now = Date.now()
  for (const [key, entry] of cacheStore) {
    if (now - entry.fetchedAt >= ttlMs) {
      cacheStore.delete(key)
    }
  }
}

/** Evict the oldest entry by `fetchedAt` timestamp. */
function evictOldest() {
  let oldestKey: string | null = null
  let oldestTime = Infinity
  for (const [key, entry] of cacheStore) {
    if (entry.fetchedAt < oldestTime) {
      oldestTime = entry.fetchedAt
      oldestKey = key
    }
  }
  if (oldestKey !== null) {
    cacheStore.delete(oldestKey)
  }
}

/**
 * Composable providing a transparent caching layer around async data fetchers.
 *
 * Entries are stored in a module-level Map keyed by an arbitrary string,
 * so data persists across component remounts. Each entry carries a
 * configurable TTL (default 60 s); stale entries are automatically
 * re-fetched on the next call to `fetchWithCache`.
 *
 * The cache is capped at {@link MAX_CACHE_SIZE} entries. When the cap is
 * reached the oldest entry (by `fetchedAt`) is evicted. Expired entries
 * are also purged on every read.
 */
export function useCache<T>(key: string, fetcher: () => Promise<T>, ttlMs = DEFAULT_TTL_MS) {
  const data: Ref<T | null> = ref(null) as Ref<T | null>
  const loading = ref(false)
  const error = ref<string | null>(null)
  const lastFetchedAt: Ref<number | null> = ref(null)

  /** Return the number of seconds since the last successful fetch, or null. */
  function secondsSinceLastFetch(): number | null {
    if (lastFetchedAt.value === null) return null
    return Math.round((Date.now() - lastFetchedAt.value) / 1000)
  }

  /** Read from cache if still valid, otherwise invoke the fetcher. */
  async function fetchWithCache(): Promise<T | null> {
    // Purge expired entries on every read
    clearExpired(ttlMs)

    const cached = cacheStore.get(key) as CacheEntry<T> | undefined
    if (cached && Date.now() - cached.fetchedAt < ttlMs) {
      data.value = cached.value
      lastFetchedAt.value = cached.fetchedAt
      return cached.value
    }

    loading.value = true
    error.value = null
    try {
      const result = await fetcher()
      const now = Date.now()

      // Evict oldest entry if at capacity (and this is a new key)
      if (!cacheStore.has(key) && cacheStore.size >= MAX_CACHE_SIZE) {
        evictOldest()
      }

      cacheStore.set(key, { value: result, fetchedAt: now })
      data.value = result
      lastFetchedAt.value = now
      return result
    } catch (e) {
      error.value = String(e)
      return null
    } finally {
      loading.value = false
    }
  }

  /** Invalidate the cache entry and re-fetch immediately. */
  async function refresh(): Promise<T | null> {
    cacheStore.delete(key)
    return fetchWithCache()
  }

  /** Invalidate without re-fetching. */
  function invalidate() {
    cacheStore.delete(key)
    data.value = null
    lastFetchedAt.value = null
  }

  return {
    data,
    loading,
    error,
    lastFetchedAt,
    secondsSinceLastFetch,
    fetchWithCache,
    refresh,
    invalidate,
  }
}
