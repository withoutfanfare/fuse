import { ref, computed, onUnmounted } from 'vue'

/**
 * Composable for automatic file-level review coverage tracking.
 *
 * Tracks how long each file's diff has been viewed. After a configurable
 * threshold (default: 5 seconds), the file is automatically marked as
 * "viewed". This complements the manual checkbox-based review tracking
 * in useReviewSession with a passive, time-based coverage metric.
 */
export function useReviewCoverage(options?: { viewThresholdMs?: number }) {
  const viewThresholdMs = options?.viewThresholdMs ?? 5000

  /** Files that have been automatically marked as "viewed" (5+ seconds). */
  const viewedFiles = ref<Set<string>>(new Set())

  /** The file currently being viewed. */
  const currentFile = ref<string | null>(null)

  /** Timer that fires when the threshold is reached. */
  let viewTimer: ReturnType<typeof setTimeout> | null = null

  /** Total files in the changeset (set externally). */
  const totalFiles = ref(0)

  /** Number of files that have met the view threshold. */
  const viewedCount = computed(() => viewedFiles.value.size)

  /** Coverage percentage (0-100). */
  const coveragePercent = computed(() => {
    if (totalFiles.value === 0) return 0
    return Math.round((viewedFiles.value.size / totalFiles.value) * 100)
  })

  /** Whether coverage is complete (all files viewed). */
  const isComplete = computed(() =>
    totalFiles.value > 0 && viewedFiles.value.size >= totalFiles.value,
  )

  /** Check if a specific file has been automatically viewed. */
  function isViewed(filePath: string): boolean {
    return viewedFiles.value.has(filePath)
  }

  /**
   * Notify the tracker that a file is now being actively viewed.
   * Call this when the user selects/expands a file in the diff viewer.
   */
  function startViewing(filePath: string) {
    if (currentFile.value === filePath) return

    // Stop tracking the previous file
    stopViewing()

    currentFile.value = filePath

    // If already viewed, no need to set a timer
    if (viewedFiles.value.has(filePath)) return

    viewTimer = setTimeout(() => {
      const updated = new Set(viewedFiles.value)
      updated.add(filePath)
      viewedFiles.value = updated
    }, viewThresholdMs)
  }

  /** Stop tracking the current file (e.g. when collapsed or navigated away). */
  function stopViewing() {
    if (viewTimer !== null) {
      clearTimeout(viewTimer)
      viewTimer = null
    }
    currentFile.value = null
  }

  /** Set total file count for coverage calculation. */
  function setTotalFiles(count: number) {
    totalFiles.value = count
  }

  /** Restore previously viewed files (e.g. from a saved session). */
  function restoreViewed(files: string[]) {
    viewedFiles.value = new Set(files)
  }

  /** Get all viewed file paths as an array (for persistence). */
  function getViewedArray(): string[] {
    return [...viewedFiles.value]
  }

  onUnmounted(() => {
    stopViewing()
  })

  return {
    viewedFiles,
    viewedCount,
    totalFiles,
    coveragePercent,
    isComplete,
    currentFile,
    isViewed,
    startViewing,
    stopViewing,
    setTotalFiles,
    restoreViewed,
    getViewedArray,
  }
}
