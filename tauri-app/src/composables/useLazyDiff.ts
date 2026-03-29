import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { parseDiff } from './useDiff'
import type { DiffFile, DiffFileSummary } from '../types'

/**
 * Composable for lazy diff loading: fetch the file list immediately,
 * then load full diff content on demand when a file is expanded.
 *
 * For PRs with many files, this avoids the upfront cost of fetching
 * and parsing the entire unified diff before showing any content.
 */
export function useLazyDiff() {
  const fileSummaries = ref<DiffFileSummary[]>([])
  const loadingFileList = ref(false)
  const loadingDiff = ref(false)
  const error = ref<string | null>(null)

  /** Cached parsed diff files, keyed by path. Populated on first diff fetch. */
  const diffCache = ref<Map<string, DiffFile>>(new Map())

  /** Set of file paths the user has expanded (requested diff content for). */
  const expandedFiles = ref<Set<string>>(new Set())

  /** Whether the full diff has been fetched and cached. */
  const diffFetched = ref(false)

  /**
   * Fetch just the file list (path + stats) for a PR.
   * This is fast and allows rendering the file tree immediately.
   */
  async function fetchFileList(prId: number) {
    loadingFileList.value = true
    error.value = null
    try {
      fileSummaries.value = await invoke<DiffFileSummary[]>('get_pr_file_list', { prId })
    } catch (e) {
      error.value = String(e)
    } finally {
      loadingFileList.value = false
    }
  }

  /**
   * Expand a file: if we haven't fetched the full diff yet, fetch it
   * and cache all files. Then mark the requested file as expanded.
   */
  async function expandFile(prId: number, filePath: string) {
    expandedFiles.value.add(filePath)

    if (!diffFetched.value) {
      loadingDiff.value = true
      error.value = null
      try {
        const rawDiff = await invoke<string>('fetch_pr_diff', { prId })
        const files = parseDiff(rawDiff)
        const cache = new Map<string, DiffFile>()
        for (const file of files) {
          cache.set(file.path, file)
        }
        diffCache.value = cache
        diffFetched.value = true
      } catch (e) {
        error.value = String(e)
        expandedFiles.value.delete(filePath)
      } finally {
        loadingDiff.value = false
      }
    }
  }

  /** Collapse a file (hide its diff content). */
  function collapseFile(filePath: string) {
    expandedFiles.value.delete(filePath)
  }

  /** Toggle a file's expanded state. */
  function toggleFile(prId: number, filePath: string) {
    if (expandedFiles.value.has(filePath)) {
      collapseFile(filePath)
    } else {
      expandFile(prId, filePath)
    }
  }

  /** Get the cached diff for a specific file, if available. */
  function getDiffFile(filePath: string): DiffFile | undefined {
    return diffCache.value.get(filePath)
  }

  /** Whether a specific file is currently expanded. */
  function isExpanded(filePath: string): boolean {
    return expandedFiles.value.has(filePath)
  }

  /** Total number of files in the PR. */
  const fileCount = computed(() => fileSummaries.value.length)

  /** Total additions across all files. */
  const totalAdditions = computed(() =>
    fileSummaries.value.reduce((sum, f) => sum + f.additions, 0),
  )

  /** Total deletions across all files. */
  const totalDeletions = computed(() =>
    fileSummaries.value.reduce((sum, f) => sum + f.deletions, 0),
  )

  /** Reset state for navigating to a different PR. */
  function reset() {
    fileSummaries.value = []
    diffCache.value = new Map()
    expandedFiles.value = new Set()
    diffFetched.value = false
    error.value = null
  }

  return {
    fileSummaries,
    loadingFileList,
    loadingDiff,
    error,
    diffFetched,
    fileCount,
    totalAdditions,
    totalDeletions,
    fetchFileList,
    expandFile,
    collapseFile,
    toggleFile,
    getDiffFile,
    isExpanded,
    reset,
  }
}
