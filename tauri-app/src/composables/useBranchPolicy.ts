import type { PullRequest, Repository } from '../types'

/**
 * A PR targets production directly when its base branch is the repository's
 * production branch (`default_branch`) and its head is NOT the integration
 * branch — the integration → production release PR is legitimate and must not
 * warn. A null/empty integration branch means the head can never match it, so
 * any PR targeting production is flagged. Matching is case-insensitive.
 */
export function isDirectToProduction(
  pr: Pick<PullRequest, 'base_branch' | 'head_branch'>,
  repo: Pick<Repository, 'default_branch' | 'integration_branch'> | null | undefined,
): boolean {
  if (!repo) return false
  const base = pr.base_branch.toLowerCase()
  const production = repo.default_branch.toLowerCase()
  if (base !== production) return false
  const integration = repo.integration_branch?.toLowerCase() ?? null
  return pr.head_branch.toLowerCase() !== integration
}

export function useBranchPolicy() {
  return { isDirectToProduction }
}
