import type { PullRequest } from '../types'

export type RiskLevel = 'low' | 'medium' | 'high' | 'critical'

export function computeRiskScore(pr: PullRequest): number {
  let score = 1
  if (pr.changed_files >= 12) score += 2
  else if (pr.changed_files >= 6) score += 1
  if ((pr.additions + pr.deletions) >= 500) score += 2
  else if ((pr.additions + pr.deletions) >= 200) score += 1
  if (pr.review_decision === 'CHANGES_REQUESTED') score += 1
  const ageHours = (Date.now() - Date.parse(pr.created_at)) / 3_600_000
  if (ageHours >= 72) score += 2
  else if (ageHours >= 24) score += 1
  if (pr.is_draft) score -= 1
  return Math.min(10, Math.max(1, score))
}

export function riskLevel(score: number): RiskLevel {
  if (score <= 3) return 'low'
  if (score <= 6) return 'medium'
  if (score <= 8) return 'high'
  return 'critical'
}

/**
 * Returns the CSS custom property for the risk colour band.
 * 1-3 green, 4-5 yellow, 6-7 orange, 8-10 red.
 */
export function riskColour(score: number): string {
  if (score <= 3) return 'var(--color-risk-low)'
  if (score <= 5) return 'var(--color-risk-medium)'
  if (score <= 7) return 'var(--color-risk-high)'
  return 'var(--color-risk-critical)'
}

export interface RiskFactor {
  label: string
  points: number
}

/**
 * Returns a breakdown of each factor's contribution to the risk score.
 */
export function computeRiskBreakdown(pr: PullRequest): RiskFactor[] {
  const factors: RiskFactor[] = []

  factors.push({ label: 'Base score', points: 1 })

  if (pr.changed_files >= 12) {
    factors.push({ label: 'Very high file count', points: 2 })
  } else if (pr.changed_files >= 6) {
    factors.push({ label: 'High file count', points: 1 })
  }

  const churn = pr.additions + pr.deletions
  if (churn >= 500) {
    factors.push({ label: 'Very large diff', points: 2 })
  } else if (churn >= 200) {
    factors.push({ label: 'Large diff', points: 1 })
  }

  if (pr.review_decision === 'CHANGES_REQUESTED') {
    factors.push({ label: 'Changes requested', points: 1 })
  }

  const ageHours = (Date.now() - Date.parse(pr.created_at)) / 3_600_000
  if (ageHours >= 72) {
    factors.push({ label: 'Stale (72 h+)', points: 2 })
  } else if (ageHours >= 24) {
    factors.push({ label: 'Ageing (24 h+)', points: 1 })
  }

  if (pr.is_draft) {
    factors.push({ label: 'Draft (reduced)', points: -1 })
  }

  return factors
}
