import { ref, computed } from 'vue'
import type {
  AiReview,
  AiReviewIssue,
  AiReviewIssueSeverity,
  ParsedAiReviewStructured,
  ComparedIssue,
  AiReviewComparisonResult,
} from '../types'

/**
 * Parse the structured AI review text into typed data.
 *
 * Expects the review to follow the format produced by the review prompt:
 *   ### Summary
 *   ...
 *   ### Issues
 *   - **[CRITICAL|WARNING|SUGGESTION]** `path/to/file` — Description
 *   ### Verdict
 *   APPROVED / CHANGES REQUESTED ...
 */
export function parseStructuredReview(text: string): ParsedAiReviewStructured {
  const lines = text.split('\n')

  let summary = ''
  const issues: AiReviewIssue[] = []
  let verdict = ''
  let approved = false

  let currentSection: 'none' | 'summary' | 'issues' | 'verdict' = 'none'

  for (const line of lines) {
    const trimmed = line.trim()

    // Detect section headers
    if (/^###?\s+summary/i.test(trimmed)) {
      currentSection = 'summary'
      continue
    }
    if (/^###?\s+issues/i.test(trimmed)) {
      currentSection = 'issues'
      continue
    }
    if (/^###?\s+verdict/i.test(trimmed)) {
      currentSection = 'verdict'
      continue
    }
    // A new top-level heading resets section (e.g. if there are extra sections)
    if (/^###?\s+/.test(trimmed) && currentSection !== 'none') {
      currentSection = 'none'
      continue
    }

    if (currentSection === 'summary') {
      if (trimmed) {
        summary += (summary ? ' ' : '') + trimmed
      }
    } else if (currentSection === 'issues') {
      // Match: - **[SEVERITY]** `file` — description
      // or:    - **[SEVERITY]** — description (no file)
      const issueMatch = trimmed.match(
        /^[-*]\s*\*\*\[?(CRITICAL|WARNING|SUGGESTION)\]?\*\*\s*(?:`([^`]+)`\s*)?[—\-–]\s*(.+)/i
      )
      if (issueMatch) {
        const severity = issueMatch[1].toLowerCase() as AiReviewIssueSeverity
        const file = issueMatch[2] || null
        const description = issueMatch[3].trim()
        issues.push({ severity, file, description })
      }
    } else if (currentSection === 'verdict') {
      if (trimmed) {
        verdict += (verdict ? ' ' : '') + trimmed
      }
    }
  }

  // Determine approval from verdict text
  const verdictLower = verdict.toLowerCase()
  approved = verdictLower.includes('approved') && !verdictLower.includes('changes requested')

  return { summary, issues, verdict, approved }
}

/**
 * Compare two parsed AI reviews and categorise issues as new, resolved, or persistent.
 *
 * Matching is performed on file path (exact) and description similarity
 * (normalised substring overlap). An issue is considered persistent if it
 * appears in both reviews with the same file and a similar description.
 */
function compareReviews(
  older: ParsedAiReviewStructured,
  newer: ParsedAiReviewStructured
): { newIssues: ComparedIssue[]; resolvedIssues: ComparedIssue[]; persistentIssues: ComparedIssue[] } {
  const matchedOlderIndices = new Set<number>()
  const matchedNewerIndices = new Set<number>()

  const persistentIssues: ComparedIssue[] = []

  // Find persistent issues — present in both reviews
  for (let ni = 0; ni < newer.issues.length; ni++) {
    const newIssue = newer.issues[ni]
    for (let oi = 0; oi < older.issues.length; oi++) {
      if (matchedOlderIndices.has(oi)) continue
      const oldIssue = older.issues[oi]

      if (issuesMatch(oldIssue, newIssue)) {
        matchedOlderIndices.add(oi)
        matchedNewerIndices.add(ni)
        persistentIssues.push({
          status: 'persistent',
          severity: newIssue.severity,
          file: newIssue.file,
          description: newIssue.description,
          matchedDescription: oldIssue.description,
        })
        break
      }
    }
  }

  // New issues — in newer but not matched
  const newIssues: ComparedIssue[] = newer.issues
    .filter((_, i) => !matchedNewerIndices.has(i))
    .map((issue) => ({
      status: 'new' as const,
      severity: issue.severity,
      file: issue.file,
      description: issue.description,
    }))

  // Resolved issues — in older but not matched
  const resolvedIssues: ComparedIssue[] = older.issues
    .filter((_, i) => !matchedOlderIndices.has(i))
    .map((issue) => ({
      status: 'resolved' as const,
      severity: issue.severity,
      file: issue.file,
      description: issue.description,
    }))

  return { newIssues, resolvedIssues, persistentIssues }
}

/**
 * Check whether two issues are likely the same concern.
 *
 * Matches on exact file path and normalised description similarity.
 */
function issuesMatch(a: AiReviewIssue, b: AiReviewIssue): boolean {
  // File paths must match (both null or same value)
  if (a.file !== b.file) return false

  // Compare normalised descriptions using word overlap
  const wordsA = normaliseWords(a.description)
  const wordsB = normaliseWords(b.description)

  if (wordsA.length === 0 || wordsB.length === 0) return false

  const setA = new Set(wordsA)
  const setB = new Set(wordsB)
  const intersection = wordsA.filter((w) => setB.has(w))

  // Jaccard similarity — threshold of 0.4 accounts for rewording
  const union = new Set([...setA, ...setB])
  const similarity = intersection.length / union.size

  return similarity >= 0.4
}

/** Normalise a description into lowercase words, stripping punctuation. */
function normaliseWords(text: string): string[] {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9\s]/g, '')
    .split(/\s+/)
    .filter((w) => w.length > 2)
}

/**
 * Composable for comparing AI reviews on a pull request.
 *
 * Parses structured review text and provides a comparison between
 * two selected reviews, categorising issues as new, resolved, or persistent.
 */
export function useAiReviewComparison() {
  const olderReviewId = ref<number | null>(null)
  const newerReviewId = ref<number | null>(null)

  const comparison = ref<AiReviewComparisonResult | null>(null)

  function compareSelectedReviews(reviews: AiReview[]) {
    if (olderReviewId.value === null || newerReviewId.value === null) {
      comparison.value = null
      return
    }

    const older = reviews.find((r) => r.id === olderReviewId.value)
    const newer = reviews.find((r) => r.id === newerReviewId.value)

    if (!older || !newer) {
      comparison.value = null
      return
    }

    const parsedOlder = parseStructuredReview(older.review_text)
    const parsedNewer = parseStructuredReview(newer.review_text)

    const { newIssues, resolvedIssues, persistentIssues } = compareReviews(parsedOlder, parsedNewer)

    comparison.value = {
      olderReviewId: older.id,
      newerReviewId: newer.id,
      newIssues,
      resolvedIssues,
      persistentIssues,
    }
  }

  const hasComparison = computed(() => comparison.value !== null)

  return {
    olderReviewId,
    newerReviewId,
    comparison,
    hasComparison,
    compareSelectedReviews,
    parseStructuredReview,
  }
}
