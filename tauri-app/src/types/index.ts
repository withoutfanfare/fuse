export interface Repository {
  id: number
  owner: string
  name: string
  default_branch: string
  added_at: string
}

export interface PullRequest {
  id: number
  repo_id: number
  number: number
  title: string
  author: string
  state: string
  head_branch: string
  base_branch: string
  additions: number
  deletions: number
  changed_files: number
  review_decision: string | null
  is_draft: boolean
  url: string
  labels: string[]
  mergeable: string | null
  created_at: string
  updated_at: string
  merged_at: string | null
  closed_at: string | null
  body: string | null
  last_synced_at: string
  review_status: string | null
  review_notes: string | null
}

export interface PrReview {
  id: number
  pr_id: number
  status: ReviewStatus
  review_notes: string | null
  review_file_path: string | null
  reviewed_at: string | null
}

export type ReviewStatus = 'pending' | 'in_progress' | 'reviewed' | 'approved' | 'changes_requested'

export interface ReviewRule {
  id: number
  repo_id: number
  rule_text: string
  position: number
}

export interface PrChangeEvent {
  repo_name: string
  pr_number: number
  pr_title: string
  author: string
  change_type: 'new' | 'merged' | 'closed' | 'reopened' | 'updated'
}

export interface SyncResult {
  repo_id: number
  repo_name: string
  synced_at: string
  pr_count: number
  error: string | null
  changes: PrChangeEvent[]
}

export interface DashboardStats {
  total_open_prs: number
  pending_reviews: number
  in_progress: number
  approved: number
  repos_count: number
  last_synced: string | null
}

export interface Worktree {
  branch: string
  path: string
  is_current: boolean
}

export interface CiCheck {
  name: string
  state: string
  conclusion: string | null
  detailsUrl: string | null
}

export interface PromptTemplate {
  id: string
  name: string
  template: string
}

export interface ParsedAiResponse {
  summary: string
  issues: { severity: 'critical' | 'warning' | 'suggestion'; description: string; file?: string }[]
  approved: boolean
}

export interface PrComment {
  author: { login: string }
  body: string
  createdAt: string
}

export interface PrReviewComment {
  author: { login: string }
  body: string
  state: string
  comments: PrComment[] | null
  submittedAt: string | null
}

export interface PrCommentsResponse {
  comments: PrComment[]
  reviews: PrReviewComment[]
}

export interface DiffFile {
  path: string
  additions: number
  deletions: number
  hunks: DiffHunk[]
}

export interface DiffHunk {
  header: string
  lines: DiffLine[]
}

export interface DiffLine {
  type: 'add' | 'remove' | 'context'
  content: string
  oldLineNumber?: number
  newLineNumber?: number
}

export interface BatchResult {
  pr_id: number
  success: boolean
  message: string
}

export interface ReviewTemplate {
  id: number
  name: string
  body: string
  position: number
}

export interface RepoGroup {
  id: number
  name: string
  colour: string
  position: number
  repo_ids: number[]
}

export interface AgeBucket {
  label: string
  count: number
}

export interface VelocityPoint {
  date: string
  reviewed: number
  merged: number
}

/** Daily open-PR and pending-review counts for sparkline trends. */
export interface DailyPrCounts {
  open_counts: number[]
  pending_counts: number[]
}

export interface AuthorStats {
  author: string
  pr_count: number
  avg_additions: number
  avg_deletions: number
  merged_count: number
  reviewed_count: number
}

export interface AiReview {
  id: number
  pr_id: number
  review_text: string
  worktree_branch: string
  created_at: string
}

export interface LinkedIssue {
  number: number
  title: string
  state: string
  url: string
  labels: string[]
  assignees: string[]
}

export interface CommitAuthor {
  name: string
  email: string
  login: string | null
}

export interface CommitInfo {
  oid: string
  messageHeadline: string
  messageBody: string
  committedDate: string
  authors: CommitAuthor[]
}

export interface ConflictStatus {
  mergeable: string
  merge_state_status: string
  has_conflicts: boolean
}

export interface PriorityFactor {
  label: string
  points: number
}

export interface PriorityQueueItem {
  pr: PullRequest
  priority_score: number
  factors: PriorityFactor[]
}

export type LabelRuleActionType = 'set_priority' | 'add_checklist' | 'assign_group'

export interface LabelRule {
  id: number
  label_pattern: string
  action_type: LabelRuleActionType
  action_config: Record<string, unknown>
  enabled: boolean
  created_at: string
}

export interface LabelRuleMatch {
  rule: LabelRule
  matched_label: string
}

export interface Deployment {
  environment: string
  status: string
  created_at: string
  updated_at: string
  url: string | null
}

export type AiReviewIssueSeverity = 'critical' | 'warning' | 'suggestion'

export interface AiReviewIssue {
  severity: AiReviewIssueSeverity
  file: string | null
  description: string
}

export interface ParsedAiReviewStructured {
  summary: string
  issues: AiReviewIssue[]
  verdict: string
  approved: boolean
}

export type IssueComparisonStatus = 'new' | 'resolved' | 'persistent'

export interface ComparedIssue {
  status: IssueComparisonStatus
  severity: AiReviewIssueSeverity
  file: string | null
  description: string
  /** The matching issue from the other review, if persistent */
  matchedDescription?: string
}

export interface AiReviewComparisonResult {
  olderReviewId: number
  newerReviewId: number
  newIssues: ComparedIssue[]
  resolvedIssues: ComparedIssue[]
  persistentIssues: ComparedIssue[]
}

export interface ReviewerWorkloadStats {
  reviewer: string
  assigned_count: number
  completed_count: number
  overdue_count: number
  avg_response_hours: number
}

export type BookmarkCategory = 'note' | 'bug' | 'question' | 'suggestion' | 'blocker'

export interface Bookmark {
  id: number
  pr_id: number
  file_path: string
  line_start: number | null
  line_end: number | null
  note: string
  category: BookmarkCategory
  resolved: boolean
  created_at: string
}

export interface BookmarkWithContext extends Bookmark {
  pr_number: number
  pr_title: string
  repo_name: string
}

export interface HandoffNote {
  id: number
  pr_id: number
  reviewer_name: string
  files_checked: string[]
  concerns: string
  remaining_work: string
  created_at: string
}

export interface DigestComparison {
  reviewed_count: number
  pending_count: number
  merged_count: number
  avg_review_seconds: number
  stale_count: number
}

export interface ReviewDigest {
  reviewed_count: number
  pending_count: number
  merged_count: number
  avg_review_seconds: number
  stale_count: number
  total_open: number
  period_start: string
  period_end: string
  previous: DigestComparison | null
}

export interface PrDependency {
  id: number
  pr_id: number
  depends_on_pr_id: number
  dependency_type: 'body_reference' | 'branch_ancestry'
}

export interface ReviewSession {
  id: number
  pr_id: number
  started_at: string
  files_reviewed: string[]
  session_notes: string | null
  status: 'active' | 'paused' | 'completed'
}

export type ToastType = 'success' | 'error' | 'warning' | 'info'

export interface Toast {
  id: number
  type: ToastType
  title: string
  message?: string
  duration?: number
}

// --- Notification Rules ---

export type NotificationRuleType = 'risk_threshold' | 'author' | 'label' | 'review_requested' | 'status_change'

export interface NotificationRule {
  id: number
  rule_type: NotificationRuleType
  rule_config: Record<string, unknown>
  enabled: boolean
  created_at: string
}

// --- Review Time Tracking ---

export interface ReviewVelocityStats {
  avg_review_seconds: number
  total_reviews: number
  total_seconds: number
  by_risk_tier: RiskTierTime[]
  weekly_trend: WeeklyTimePoint[]
}

export interface RiskTierTime {
  tier: string
  avg_seconds: number
  count: number
}

export interface WeeklyTimePoint {
  week_start: string
  total_seconds: number
  review_count: number
}

// --- Checklist Templates ---

export interface ChecklistTemplate {
  id: number
  repo_id: number | null
  name: string
  items: ChecklistTemplateItem[]
  created_at: string
}

export interface ChecklistTemplateItem {
  id: number
  template_id: number
  text: string
  description: string | null
  position: number
}

// --- Filter Presets ---

export interface FilterPreset {
  id: number
  name: string
  is_builtin: boolean
  filter_config: FilterPresetConfig
  created_at: string
}

export interface FilterPresetConfig {
  repoId?: number | null
  state?: string
  searchQuery?: string
  sortBy?: string
  sortAsc?: boolean
  reviewRequested?: boolean
  minRiskScore?: number
  staleOnly?: boolean
  staleDays?: number
}

// --- Aggregate Dashboard ---

export interface AggregateDashboard {
  total_open_prs: number
  review_requested_count: number
  high_risk_count: number
  stale_count: number
  repo_summaries: RepoSummary[]
  top_risk_prs: TopRiskPr[]
}

export interface RepoSummary {
  repo_id: number
  repo_name: string
  open_pr_count: number
  oldest_pr_age_hours: number
  last_sync_at: string | null
}

export interface TopRiskPr {
  pr_id: number
  number: number
  title: string
  author: string
  repo_name: string
  risk_score: number
  changed_files: number
  additions: number
  deletions: number
  created_at: string
}
