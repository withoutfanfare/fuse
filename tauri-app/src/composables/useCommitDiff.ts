import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { parseDiff } from './useDiff'
import type { DiffFile, CommitInfo } from '../types'

/**
 * Composable for commit-level diff navigation within a PR.
 *
 * Allows viewing the diff for a single commit or a range of commits,
 * rather than the aggregate diff across all PR changes. Uses the GitHub
 * API to fetch per-commit patches.
 */
export function useCommitDiff() {
  const selectedCommit = ref<string | null>(null)
  const selectedRangeStart = ref<string | null>(null)
  const commitDiffFiles = ref<DiffFile[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  /** Whether a specific commit (or range) is selected, rather than showing all changes. */
  const isFiltered = computed(() => selectedCommit.value !== null)

  /**
   * Select a single commit to view its diff.
   * Pass null to return to the aggregate (all changes) view.
   */
  async function selectCommit(prId: number, commitOid: string | null) {
    // Clear range selection when selecting a single commit
    selectedRangeStart.value = null

    if (commitOid === null) {
      selectedCommit.value = null
      commitDiffFiles.value = []
      return
    }

    selectedCommit.value = commitOid
    loading.value = true
    error.value = null

    try {
      const rawDiff = await invoke<string>('fetch_commit_diff', {
        prId,
        commitOid,
      })
      commitDiffFiles.value = parseDiff(rawDiff)
    } catch (e) {
      error.value = String(e)
      commitDiffFiles.value = []
    } finally {
      loading.value = false
    }
  }

  /**
   * Select a range of commits (from baseOid to headOid) to view the combined diff.
   * Used for Shift-click commit range selection.
   */
  async function selectCommitRange(
    prId: number,
    commits: CommitInfo[],
    startOid: string,
    endOid: string,
  ) {
    // Find the indices to determine which is earlier
    const startIdx = commits.findIndex(c => c.oid === startOid)
    const endIdx = commits.findIndex(c => c.oid === endOid)
    if (startIdx === -1 || endIdx === -1) return

    // Commits are in reverse chronological order (newest first)
    // so the "base" is the later index (older commit's parent)
    const [baseIdx, headIdx] = startIdx > endIdx ? [startIdx, endIdx] : [endIdx, startIdx]
    const baseOid = commits[baseIdx].oid
    const headOid = commits[headIdx].oid

    selectedCommit.value = headOid
    selectedRangeStart.value = baseOid
    loading.value = true
    error.value = null

    try {
      // For range, we use the parent of the base commit as the compare start
      const rawDiff = await invoke<string>('fetch_commit_range_diff', {
        prId,
        baseOid: `${baseOid}^`,
        headOid,
      })
      commitDiffFiles.value = parseDiff(rawDiff)
    } catch (e) {
      error.value = String(e)
      commitDiffFiles.value = []
    } finally {
      loading.value = false
    }
  }

  /** Clear selection and return to aggregate diff view. */
  function clearSelection() {
    selectedCommit.value = null
    selectedRangeStart.value = null
    commitDiffFiles.value = []
    error.value = null
  }

  return {
    selectedCommit,
    selectedRangeStart,
    commitDiffFiles,
    loading,
    error,
    isFiltered,
    selectCommit,
    selectCommitRange,
    clearSelection,
  }
}
