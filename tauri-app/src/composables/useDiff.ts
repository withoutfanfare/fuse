import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { DiffFile, DiffHunk, DiffLine } from '../types'

/**
 * Parse a unified diff string into structured DiffFile objects.
 */
export function parseDiff(rawDiff: string): DiffFile[] {
  const files: DiffFile[] = []

  // Split on diff --git boundaries
  const fileSections = rawDiff.split(/^diff --git /m).filter(Boolean)

  for (const section of fileSections) {
    const lines = section.split('\n')

    // Extract file path from the first line: a/path b/path
    const headerMatch = lines[0]?.match(/a\/(.+?)\s+b\/(.+)/)
    const filePath = headerMatch ? headerMatch[2] : 'unknown'

    let additions = 0
    let deletions = 0
    const hunks: DiffHunk[] = []
    let currentHunk: DiffHunk | null = null
    let oldLine = 0
    let newLine = 0

    for (let i = 1; i < lines.length; i++) {
      const line = lines[i]

      // Hunk header: @@ -oldStart,oldCount +newStart,newCount @@
      const hunkMatch = line.match(/^@@\s+-(\d+)(?:,\d+)?\s+\+(\d+)(?:,\d+)?\s+@@(.*)/)
      if (hunkMatch) {
        currentHunk = {
          header: line,
          lines: [],
        }
        hunks.push(currentHunk)
        oldLine = parseInt(hunkMatch[1], 10)
        newLine = parseInt(hunkMatch[2], 10)
        continue
      }

      // Skip metadata lines (index, ---, +++ etc.) before first hunk
      if (!currentHunk) continue

      const diffLine: DiffLine = { type: 'context', content: line }

      if (line.startsWith('+')) {
        diffLine.type = 'add'
        diffLine.content = line.substring(1)
        diffLine.newLineNumber = newLine++
        additions++
      } else if (line.startsWith('-')) {
        diffLine.type = 'remove'
        diffLine.content = line.substring(1)
        diffLine.oldLineNumber = oldLine++
        deletions++
      } else if (line.startsWith(' ') || line === '') {
        diffLine.type = 'context'
        diffLine.content = line.startsWith(' ') ? line.substring(1) : line
        diffLine.oldLineNumber = oldLine++
        diffLine.newLineNumber = newLine++
      } else if (line.startsWith('\\')) {
        // "\ No newline at end of file" — skip
        continue
      } else {
        // Other metadata lines within a hunk are rare, skip
        continue
      }

      currentHunk.lines.push(diffLine)
    }

    // Only add files that actually have hunks (skip binary files, etc.)
    if (hunks.length > 0) {
      files.push({ path: filePath, additions, deletions, hunks })
    }
  }

  return files
}

export function useDiff() {
  const rawDiff = ref('')
  const files = ref<DiffFile[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchDiff(prId: number) {
    loading.value = true
    error.value = null
    try {
      rawDiff.value = await invoke<string>('fetch_pr_diff', { prId })
      files.value = parseDiff(rawDiff.value)
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  return { rawDiff, files, loading, error, fetchDiff }
}
