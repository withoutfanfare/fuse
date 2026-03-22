<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed, watch, inject, defineAsyncComponent, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import { usePullRequestsStore } from '../stores/pullRequests'
import { useRepositoriesStore } from '../stores/repositories'
import { useToastStore } from '../stores/toast'
import { computeRiskScore } from '../composables/useRiskScore'
import { useConfirm } from '@stuntrocket/ui'
import { useConfetti } from '../composables/useConfetti'
import { useDiff } from '../composables/useDiff'
import { useComments } from '../composables/useComments'
import { useReviewTimer } from '../composables/useReviewTimer'
import { useChecklist } from '../composables/useChecklist'
import { useRecentPrs } from '../composables/useRecentPrs'
import { useLinkedIssues } from '../composables/useLinkedIssues'
import { useConflictDetection } from '../composables/useConflictDetection'
import { useCommitHistory } from '../composables/useCommitHistory'
import { useDeploymentStatus } from '../composables/useDeploymentStatus'
import { useCache } from '../composables/useCache'
import { useFocusMode } from '../composables/useFocusMode'
import type { PullRequest, ReviewStatus, ReviewRule, CiCheck, PrCommentsResponse } from '../types'
import { SResizableSplit, STag } from '@stuntrocket/ui'
import RiskGauge from '../components/RiskGauge.vue'
import ReviewStatusComponent from '../components/ReviewStatus.vue'
import AuthorAvatar from '../components/AuthorAvatar.vue'
/* Lazy-loaded: only needed on non-default tabs or conditionally shown sidebar panels */
const WorktreePanel = defineAsyncComponent(() => import('../components/WorktreePanel.vue'))
import CiChecksPanel from '../components/CiChecksPanel.vue'
import CiStatusBadge from '../components/CiStatusBadge.vue'
const AiPromptBuilder = defineAsyncComponent(() => import('../components/AiPromptBuilder.vue'))
const AiReviewPanel = defineAsyncComponent(() => import('../components/AiReviewPanel.vue'))
const CommentThread = defineAsyncComponent(() => import('../components/CommentThread.vue'))
const DiffViewer = defineAsyncComponent(() => import('../components/DiffViewer.vue'))
/* SkeletonPRDetail removed — a simple spinner gives clearer loading feedback */
import MarkdownRenderer from '../components/MarkdownRenderer.vue'
import TemplateSelector from '../components/TemplateSelector.vue'
import { SBreadcrumbs } from '@stuntrocket/ui'
import ConflictBadge from '../components/ConflictBadge.vue'
const CommitTimeline = defineAsyncComponent(() => import('../components/CommitTimeline.vue'))
import DeploymentStatus from '../components/DeploymentStatus.vue'
import { SSegmentedControl } from '@stuntrocket/ui'
import BookmarksList from '../components/BookmarksList.vue'
const HandoffComposer = defineAsyncComponent(() => import('../components/HandoffComposer.vue'))
const ReviewSummaryPanel = defineAsyncComponent(() => import('../components/ReviewSummaryPanel.vue'))

const route = useRoute()
const router = useRouter()
const prStore = usePullRequestsStore()
const repoStore = useRepositoriesStore()
const toastStore = useToastStore()
const { confirm } = useConfirm()
const { fireFromElement } = useConfetti()
const { focusActive, toggle: toggleFocusMode } = useFocusMode()
const dismissNavLoading = inject<(() => void) | null>('dismissNavLoading', null)

const pr = ref<PullRequest | null>(null)
const mergeButtonRef = ref<HTMLButtonElement | null>(null)
const justMerged = ref(false)
const loading = ref(true)
const reviewRules = ref<string[]>([])
const reviewRuleObjects = ref<ReviewRule[]>([])
const checklistExpanded = ref(true)

/* Tabbed layout (Improvement 1) — four detail tabs with sidebar fixed across all */
type DetailTab = 'overview' | 'code' | 'discussion' | 'ai'
const activeTab = ref<DetailTab>('overview')
const detailTabs: { value: string; label: string }[] = [
  { value: 'overview', label: 'Overview' },
  { value: 'code', label: 'Code' },
  { value: 'discussion', label: 'Discussion' },
  { value: 'ai', label: 'AI' },
]

const prId = Number(route.params.id)
const { checkedRules, load: loadChecklist } = useChecklist(prId)
const { push: pushRecentPr } = useRecentPrs()
const { rawDiff, files: diffFiles, fetchDiff, loading: diffLoading } = useDiff()
const { comments: prComments, reviews: prReviews, loading: commentsLoading, fetchComments } = useComments()
const { linkedIssues, loading: issuesLoading, fetchLinkedIssues } = useLinkedIssues()
const { conflictStatus, loading: conflictLoading, checkConflicts } = useConflictDetection()
const { commits: prCommits, loading: commitsLoading, error: commitsError, fetchCommits } = useCommitHistory()
const { deployments, loading: deploymentsLoading, fetchDeployments } = useDeploymentStatus()

/* Cached CI checks — module-level Map survives component remounts, 60 s TTL */
const ciCache = useCache<CiCheck[]>(
  `ci-checks-${prId}`,
  () => invoke<CiCheck[]>('fetch_pr_checks', { prId }),
)
const ciChecks = computed(() => ciCache.data.value ?? [])
const ciLoading = computed(() => ciCache.loading.value)

/* Cached comments — module-level Map survives component remounts, 60 s TTL */
const commentsCache = useCache<PrCommentsResponse>(
  `comments-${prId}`,
  () => invoke<PrCommentsResponse>('fetch_pr_comments', { prId }),
)

const { elapsed: reviewElapsed } = useReviewTimer(prId)

const formattedReviewTime = computed(() => {
  const total = reviewElapsed.value
  const mins = Math.floor(total / 60)
  const secs = total % 60
  const padded = secs.toString().padStart(2, '0')
  return `${mins}m ${padded}s`
})

const riskScore = computed(() => pr.value ? computeRiskScore(pr.value) : 0)

const breadcrumbItems = computed(() => {
  const items: { label: string; to?: string }[] = [
    { label: 'Pull Requests', to: '/prs' },
  ]
  if (repoFullName.value) {
    items.push({ label: repoFullName.value })
  }
  if (pr.value) {
    items.push({ label: `#${pr.value.number} ${pr.value.title}` })
  }
  return items
})

const repoName = computed(() => {
  if (!pr.value) return ''
  const repo = repoStore.repos.find(r => r.id === pr.value!.repo_id)
  return repo ? repo.name : ''
})

const repoFullName = computed(() => {
  if (!pr.value) return ''
  const repo = repoStore.repos.find(r => r.id === pr.value!.repo_id)
  return repo ? `${repo.owner}/${repo.name}` : ''
})

/** Yield to the browser so it can paint before we continue. */
function waitForPaint(): Promise<void> {
  return new Promise(resolve => requestAnimationFrame(() => setTimeout(resolve, 0)))
}

onMounted(async () => {
  const id = Number(route.params.id)
  if (repoStore.repos.length === 0) await repoStore.fetchAll()
  pr.value = await prStore.fetchOne(id)
  loading.value = false
  dismissNavLoading?.()

  // Yield to the browser so it paints the PR header before secondary fetches start
  await waitForPaint()

  // Load checklist state after initial paint (deferred from composable setup)
  loadChecklist()

  if (pr.value) {
    // Track this PR as recently visited
    pushRecentPr({
      id: pr.value.id,
      number: pr.value.number,
      title: pr.value.title,
      repoFullName: repoFullName.value,
    })

    // Fire all secondary fetches in parallel — each section has its own loading state
    const rulesPromise = prStore.fetchRules(pr.value.repo_id)

    const secondaryFetches: Promise<unknown>[] = [
      ciCache.fetchWithCache(),
      fetchDiff(pr.value.id),
      fetchLinkedIssues(pr.value.id),
    ]

    if (!pr.value.merged_at && !pr.value.closed_at) {
      secondaryFetches.push(checkConflicts(pr.value.id))
      secondaryFetches.push(fetchDeployments(pr.value.id))
    }

    const [rules] = await Promise.all([rulesPromise, ...secondaryFetches])
    reviewRuleObjects.value = rules as ReviewRule[]
    reviewRules.value = (rules as ReviewRule[]).map(r => r.rule_text)

    // Handle bookmark deep-link navigation from global bookmarks view or command palette (Phase 5.2/5.3)
    const bookmarkFile = route.query.bookmarkFile as string | undefined
    const bookmarkLine = route.query.bookmarkLine as string | undefined
    if (bookmarkFile) {
      diffLoaded.value = true
      await handleNavigateToBookmark({
        file_path: bookmarkFile,
        line_start: bookmarkLine ? Number(bookmarkLine) : null,
      })
    }
  }
})

/* Close worktree dropdown when clicking outside */
function handleWorktreeClickOutside(e: MouseEvent) {
  const wrapper = (e.target as HTMLElement)?.closest('.worktree-dropdown-wrapper')
  if (!wrapper) showWorktreeDropdown.value = false
}
onMounted(() => document.addEventListener('click', handleWorktreeClickOutside))
onBeforeUnmount(() => document.removeEventListener('click', handleWorktreeClickOutside))

async function handleStatusChange(status: ReviewStatus, notes: string) {
  if (!pr.value) return
  await prStore.updateReviewStatus(pr.value.id, status, notes)
  pr.value = await prStore.fetchOne(pr.value.id)
}

async function openInGitHub() {
  if (pr.value?.url) {
    await openUrl(pr.value.url)
  }
}

const descriptionExpanded = ref(false)
const commentsExpanded = ref(false)
const commitsExpanded = ref(false)
const commitsLoaded = ref(false)
const diffExpanded = ref(false)
const diffLoaded = ref(false)
const approving = ref(false)
const merging = ref(false)
const approveBody = ref('')
const showWorktreeDropdown = ref(false)

/* Refs for bookmark integration (Phase 5.1 + 5.2) */
const bookmarksListRef = ref<InstanceType<typeof BookmarksList> | null>(null)
const diffViewerRef = ref<InstanceType<typeof DiffViewer> | null>(null)

/** Handle bookmark creation from DiffViewer file header or context menu (5.1). */
function handleDiffBookmark(filePath: string, lineStart: number | null, lineEnd: number | null) {
  if (bookmarksListRef.value) {
    bookmarksListRef.value.prefillBookmark(filePath, lineStart, lineEnd)
  }
}

/** Navigate to diff location when a bookmark is clicked (5.2). */
async function handleNavigateToBookmark(bookmark: { file_path: string; line_start: number | null }) {
  // Switch to code tab
  activeTab.value = 'code'
  // Ensure diff is loaded
  if (pr.value && !diffLoaded.value) {
    await fetchDiff(pr.value.id)
    diffLoaded.value = true
  }
  // Wait for DOM to update, then scroll
  await nextTick()
  await nextTick()
  if (diffViewerRef.value && bookmark.file_path !== '(general)') {
    if (bookmark.line_start != null) {
      diffViewerRef.value.scrollToLine(bookmark.file_path, bookmark.line_start)
    } else {
      diffViewerRef.value.scrollToFile(bookmark.file_path)
    }
  }
}

const isOpen = computed(() => pr.value && !pr.value.merged_at && !pr.value.closed_at)
const isDraftOpen = computed(() => Boolean(pr.value && isOpen.value && pr.value.is_draft))
const isForbiddenTarget = computed(() => {
  if (!pr.value) return false
  const base = pr.value.base_branch.toLowerCase()
  return base === 'main' || base === 'master'
})

async function handleApprove() {
  if (!pr.value) return
  approving.value = true
  const body = approveBody.value.trim() || undefined
  const result = await prStore.approvePr(pr.value.id, body)
  if (result !== null) {
    toastStore.addToast('success', 'Approved', `PR #${pr.value.number} approved on GitHub`)
    pr.value = await prStore.fetchOne(pr.value.id)
    approveBody.value = ''
  } else {
    toastStore.addToast('error', 'Approval failed', prStore.error ?? 'Failed to approve')
  }
  approving.value = false
}

function onTemplateSelected(body: string) {
  approveBody.value = body
}

async function handleMerge() {
  if (!pr.value) return
  if (pr.value.is_draft) {
    toastStore.addToast('warning', 'Merge blocked', 'Draft pull requests must be marked ready for review before merging')
    return
  }
  const confirmed = await confirm({
    title: 'Merge Pull Request',
    message: `Are you sure you want to merge #${pr.value.number}?`,
    confirmLabel: 'Merge',
  })
  if (!confirmed) return
  merging.value = true
  const result = await prStore.mergePr(pr.value.id, 'squash')
  if (result !== null) {
    toastStore.addToast('success', 'Merged', `PR #${pr.value.number} merged successfully`)
    pr.value = await prStore.fetchOne(pr.value.id)

    /* Celebration — confetti burst from the merge button */
    justMerged.value = true
    if (mergeButtonRef.value) {
      fireFromElement(mergeButtonRef.value)
    }
  } else {
    toastStore.addToast('error', 'Merge failed', prStore.error ?? 'Failed to merge')
  }
  merging.value = false
}

/** Force-refresh CI checks, bypassing cache. */
async function handleCiRefresh() {
  await ciCache.refresh()
}

/** Force-refresh comments, bypassing cache. */
async function handleCommentsRefresh() {
  const result = await commentsCache.refresh()
  if (result) {
    prComments.value = result.comments
    prReviews.value = result.reviews
  }
}

async function toggleCommits() {
  commitsExpanded.value = !commitsExpanded.value
  if (commitsExpanded.value && pr.value && !commitsLoaded.value) {
    await fetchCommits(pr.value.id)
    commitsLoaded.value = true
  }
}

/* Auto-load data when switching tabs so users don't need to manually expand sections */
watch(activeTab, async (tab) => {
  if (!pr.value) return
  if (tab === 'code' && !diffLoaded.value) {
    diffExpanded.value = true
    await fetchDiff(pr.value.id)
    diffLoaded.value = true
  }
  if (tab === 'code' && !commitsLoaded.value) {
    commitsExpanded.value = true
    await fetchCommits(pr.value.id)
    commitsLoaded.value = true
  }
  if (tab === 'discussion' && prComments.value.length === 0 && prReviews.value.length === 0) {
    commentsExpanded.value = true
    const cached = await commentsCache.fetchWithCache()
    if (cached) {
      prComments.value = cached.comments
      prReviews.value = cached.reviews
    } else {
      await fetchComments(pr.value.id)
    }
  }
})

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-GB', {
    day: 'numeric', month: 'short', year: 'numeric',
    hour: '2-digit', minute: '2-digit',
  })
}
</script>

<template>
  <div class="pr-detail-page">
  <div v-if="loading" class="view-loading">
    <div class="view-spinner" />
    <span class="view-loading-label">Loading pull request…</span>
  </div>
  <div v-else-if="!pr" class="not-found">
    <p>Pull request not found.</p>
    <button @click="router.push({ name: 'pull-requests' })">Back to PRs</button>
  </div>
  <div v-else class="pr-detail">
    <SBreadcrumbs :segments="breadcrumbItems" @navigate="$router.push($event)" />

    <div class="pr-detail-header">
      <div class="header-left">
        <div class="pr-title-row">
          <RiskGauge :score="riskScore" :size="48" />
          <h1 class="pr-title">{{ pr.title }}</h1>
        </div>
        <div class="pr-meta-row">
          <span class="pr-number">#{{ pr.number }}</span>
          <span class="separator">&middot;</span>
          <span class="pr-repo">{{ repoFullName }}</span>
          <span class="separator">&middot;</span>
          <AuthorAvatar :username="pr.author" :size="24" />
          <span class="pr-author">{{ pr.author }}</span>
          <span class="separator">&middot;</span>
          <span v-if="pr.is_draft" class="draft-badge">Draft</span>
          <span class="pr-state" :class="[pr.merged_at ? 'merged' : pr.closed_at ? 'closed' : 'open', { 'merge-pulse': justMerged }]">
            {{ pr.merged_at ? 'Merged' : pr.closed_at ? 'Closed' : 'Open' }}
          </span>
          <CiStatusBadge v-if="ciChecks.length > 0" :checks="ciChecks" />
          <ConflictBadge v-if="!pr.merged_at && !pr.closed_at" :status="conflictStatus" :loading="conflictLoading" />
          <span class="timer-badge" title="Time spent reviewing this PR">{{ formattedReviewTime }}</span>
        </div>
        <DeploymentStatus
          v-if="!pr.merged_at && !pr.closed_at && (deploymentsLoading || deployments.length > 0)"
          :deployments="deployments"
          :loading="deploymentsLoading"
          class="deployment-row"
        />
      </div>
      <div class="header-actions">
        <button
          class="btn-focus"
          :class="{ active: focusActive }"
          :title="focusActive ? 'Exit focus mode (⌘⇧F)' : 'Enter focus mode (⌘⇧F)'"
          @click="toggleFocusMode"
        >
          {{ focusActive ? 'Exit Focus' : 'Focus' }}
        </button>
        <button
          class="btn-session"
          @click="router.push({ name: 'review-session', params: { prId: prId } })"
          title="Start a focused review session"
        >
          Review Session
        </button>
        <div v-if="repoName" class="worktree-dropdown-wrapper">
          <button
            class="btn-worktree-toggle"
            @click="showWorktreeDropdown = !showWorktreeDropdown"
            title="Review worktree"
          >
            Worktree
            <span class="dropdown-chevron" :class="{ open: showWorktreeDropdown }">▾</span>
          </button>
          <Transition name="dropdown-fade">
            <div v-if="showWorktreeDropdown" class="worktree-dropdown">
              <WorktreePanel
                :repo-name="repoName"
                :branch="pr.head_branch"
                :base-branch="pr.base_branch"
              />
            </div>
          </Transition>
        </div>
        <button class="btn-github" @click="openInGitHub">
          Open in GitHub →
        </button>
      </div>
    </div>

    <Transition name="banner-fade">
      <div v-if="isForbiddenTarget && isOpen" class="target-warning-banner">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="warning-icon">
          <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
          <line x1="12" y1="9" x2="12" y2="13" />
          <line x1="12" y1="17" x2="12.01" y2="17" />
        </svg>
        <span class="warning-text">
          Targets <code>{{ pr.base_branch }}</code> — review carefully before merging to a protected branch
        </span>
      </div>
    </Transition>

    <!-- Tabbed layout (Improvement 1) — tabs sit above the split, sidebar stays fixed -->
    <SSegmentedControl v-model="activeTab" :options="detailTabs" class="detail-tabs" />

    <!-- Code tab uses full width — no sidebar split -->
    <div v-if="activeTab === 'code'" class="detail-main detail-main--full">

        <!-- ===== Code Tab (full-width) ===== -->
          <section class="detail-section">
            <h2 class="section-title">Diff</h2>
            <div v-if="diffLoading" class="section-loading">Loading diff...</div>
            <template v-else-if="diffFiles.length > 0">
              <DiffViewer ref="diffViewerRef" :files="diffFiles" @bookmark-file="handleDiffBookmark" />
            </template>
            <div v-else class="section-empty">No diff available.</div>
          </section>

          <section class="detail-section collapsible-section">
            <button class="description-toggle" @click="toggleCommits">
              <h2 class="section-title">Commit History</h2>
              <span class="toggle-icon" :class="{ expanded: commitsExpanded }">&#9656;</span>
            </button>
            <div v-if="commitsExpanded" class="collapsible-content">
              <CommitTimeline
                :commits="prCommits"
                :loading="commitsLoading"
                :error="commitsError"
              />
            </div>
          </section>
    </div>

    <SResizableSplit v-else storage-key="pr-detail-sidebar-pct" :initial-size="75" :min-size="55" :max-size="85">
      <template #first>
      <div class="detail-main">

        <!-- ===== Overview Tab ===== -->
        <template v-if="activeTab === 'overview'">
        <section class="detail-section">
          <h2 class="section-title">Change Summary</h2>
          <div class="change-stats">
            <div class="stat">
              <span class="stat-value additions">+{{ pr.additions }}</span>
              <span class="stat-label">Additions</span>
            </div>
            <div class="stat">
              <span class="stat-value deletions">-{{ pr.deletions }}</span>
              <span class="stat-label">Deletions</span>
            </div>
            <div class="stat">
              <span class="stat-value">{{ pr.changed_files }}</span>
              <span class="stat-label">Files Changed</span>
            </div>
          </div>
        </section>

        <section v-if="pr.body" class="detail-section description-section">
          <button class="description-toggle" @click="descriptionExpanded = !descriptionExpanded">
            <h2 class="section-title">Description</h2>
            <span class="toggle-icon" :class="{ expanded: descriptionExpanded }">&#9656;</span>
          </button>
          <div v-if="descriptionExpanded" class="description-content">
            <MarkdownRenderer :content="pr.body" />
          </div>
        </section>

        <section class="detail-section">
          <h2 class="section-title">Branch</h2>
          <div class="branch-info">
            <code>{{ pr.head_branch }}</code>
            <span class="branch-arrow">→</span>
            <code>{{ pr.base_branch }}</code>
          </div>
        </section>

        <section v-if="pr.labels.length > 0" class="detail-section">
          <h2 class="section-title">Labels</h2>
          <div class="labels">
            <STag v-for="label in pr.labels" :key="label">
              {{ label }}
            </STag>
          </div>
        </section>

        <section class="detail-section">
          <h2 class="section-title">Timeline</h2>
          <div class="timeline">
            <div class="timeline-item">
              <span class="timeline-label">Created</span>
              <span class="timeline-value">{{ formatDate(pr.created_at) }}</span>
            </div>
            <div class="timeline-item">
              <span class="timeline-label">Updated</span>
              <span class="timeline-value">{{ formatDate(pr.updated_at) }}</span>
            </div>
            <div v-if="pr.merged_at" class="timeline-item">
              <span class="timeline-label">Merged</span>
              <span class="timeline-value">{{ formatDate(pr.merged_at) }}</span>
            </div>
            <div v-if="pr.closed_at && !pr.merged_at" class="timeline-item">
              <span class="timeline-label">Closed</span>
              <span class="timeline-value">{{ formatDate(pr.closed_at) }}</span>
            </div>
          </div>
        </section>

        <CiChecksPanel
          v-if="!ciLoading"
          :checks="ciChecks"
          :last-fetched-at="ciCache.lastFetchedAt.value"
          class="ci-section"
          @refresh="handleCiRefresh"
        />

        <section class="detail-section">
          <ReviewStatusComponent
            :status="(pr.review_status as ReviewStatus | null)"
            :notes="pr.review_notes ?? null"
            @status-changed="handleStatusChange"
          />
        </section>

        <section v-if="isOpen" class="detail-section github-actions">
          <h2 class="section-title">GitHub Actions</h2>

          <div v-if="isForbiddenTarget" class="merge-warning-inline">
            <strong>Merge blocked</strong> — this PR targets <code>{{ pr.base_branch }}</code>. PRs must only merge into staging.
          </div>

          <div class="approve-body-group">
            <div class="approve-body-header">
              <label class="approve-body-label">Review comment (optional)</label>
              <TemplateSelector @select-template="onTemplateSelected" />
            </div>
            <textarea
              v-model="approveBody"
              class="approve-body-input"
              placeholder="Add a review comment..."
              rows="2"
            />
          </div>

          <div class="action-buttons">
            <button
              class="btn-approve"
              :disabled="approving || merging"
              @click="handleApprove"
            >
              {{ approving ? 'Approving…' : 'Approve on GitHub' }}
            </button>
            <button
              ref="mergeButtonRef"
              class="btn-merge"
              :disabled="merging || approving || isForbiddenTarget || isDraftOpen"
              :title="isDraftOpen ? 'Draft pull requests cannot be merged until they are marked ready for review' : undefined"
              @click="handleMerge"
            >
              {{ merging ? 'Merging…' : 'Squash & Merge' }}
            </button>
          </div>

          <ReviewSummaryPanel
            :pr="pr"
            :checked-rules="checkedRules"
            :review-rules="reviewRules"
            :bookmarks="[]"
            :review-elapsed-seconds="reviewElapsed"
            :files-reviewed="diffFiles.length"
            :total-files="pr.changed_files"
          />
        </section>
        </template>

        <!-- ===== Discussion Tab ===== -->
        <template v-if="activeTab === 'discussion'">
          <section class="detail-section">
            <div class="cache-status-bar">
              <h2 class="section-title" style="margin-bottom: 0; flex: 1;">Comments &amp; Reviews</h2>
              <span v-if="commentsCache.lastFetchedAt.value" class="last-fetched-label">
                {{ Math.round((Date.now() - commentsCache.lastFetchedAt.value) / 1000) < 5 ? 'just now' : Math.round((Date.now() - commentsCache.lastFetchedAt.value) / 1000) + 's ago' }}
              </span>
              <button
                class="refresh-btn-inline"
                title="Refresh comments"
                aria-label="Refresh comments"
                @click="handleCommentsRefresh"
              >
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="23 4 23 10 17 10" />
                  <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
                </svg>
              </button>
            </div>
            <div v-if="commentsLoading" class="section-loading">Loading comments...</div>
            <CommentThread
              v-else
              :comments="prComments"
              :reviews="prReviews"
            />
          </section>

          <section class="detail-section collapsible-section">
            <HandoffComposer
              :pr-id="prId"
              :available-files="diffFiles.map(f => f.path)"
            />
          </section>
        </template>

        <!-- ===== AI Tab ===== -->
        <template v-if="activeTab === 'ai'">
          <section class="detail-section">
            <AiReviewPanel :pr="pr" />
          </section>

          <section class="detail-section">
            <AiPromptBuilder
              :pr="pr"
              :diff="rawDiff || undefined"
              :rules="reviewRuleObjects.length > 0 ? reviewRuleObjects : undefined"
            />
          </section>
        </template>

      </div>
      </template>

      <template #second>
      <div class="detail-sidebar">
        <section v-if="issuesLoading || linkedIssues.length > 0" class="linked-issues-panel">
          <h3 class="linked-issues-title">Linked Issues</h3>
          <div v-if="issuesLoading" class="linked-issues-loading">Loading issues...</div>
          <div v-else class="linked-issues-list">
            <div
              v-for="issue in linkedIssues"
              :key="issue.number"
              class="linked-issue-card"
            >
              <div class="linked-issue-header">
                <span class="linked-issue-number">#{{ issue.number }}</span>
                <span
                  class="linked-issue-state"
                  :class="issue.state === 'OPEN' ? 'state-open' : 'state-closed'"
                >
                  {{ issue.state === 'OPEN' ? 'Open' : 'Closed' }}
                </span>
              </div>
              <a
                class="linked-issue-title"
                href="#"
                @click.prevent="openUrl(issue.url)"
              >
                {{ issue.title }}
              </a>
              <div v-if="issue.labels.length > 0" class="linked-issue-labels">
                <STag v-for="label in issue.labels" :key="label">
                  {{ label }}
                </STag>
              </div>
              <div v-if="issue.assignees.length > 0" class="linked-issue-assignees">
                <span class="assignee-label">Assignees:</span>
                <span
                  v-for="assignee in issue.assignees"
                  :key="assignee"
                  class="linked-issue-assignee"
                >
                  {{ assignee }}
                </span>
              </div>
            </div>
          </div>
        </section>

        <section v-if="reviewRules.length > 0" class="checklist-panel">
          <button class="checklist-toggle" @click="checklistExpanded = !checklistExpanded">
            <h3 class="checklist-title">Review Checklist</h3>
            <span class="toggle-icon" :class="{ expanded: checklistExpanded }">&#9656;</span>
          </button>
          <div v-if="checklistExpanded" class="checklist-items">
            <label
              v-for="(rule, index) in reviewRules"
              :key="index"
              class="checklist-item"
              :class="{ checked: checkedRules[index] }"
            >
              <input
                v-model="checkedRules[index]"
                type="checkbox"
                class="checklist-checkbox"
              />
              <span class="checklist-text">{{ rule }}</span>
            </label>
          </div>
        </section>

        <BookmarksList ref="bookmarksListRef" :pr-id="prId" :available-files="diffFiles.map(f => f.path)" @navigate-to-bookmark="handleNavigateToBookmark" />
      </div>
      </template>
    </SResizableSplit>
  </div>
  </div>
</template>

<style scoped>
/* Protected branch warning banner */
.target-warning-banner {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-4);
  margin-bottom: var(--space-4);
  background: rgba(234, 179, 8, 0.06);
  border: 1px solid rgba(234, 179, 8, 0.2);
  border-radius: var(--radius-md);
  font-size: 12px;
  color: var(--color-status-warning);
}

.target-warning-banner .warning-icon {
  flex-shrink: 0;
  opacity: 0.8;
}

.target-warning-banner .warning-text {
  color: var(--color-text-secondary);
}

.target-warning-banner code {
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 600;
  color: var(--color-status-warning);
  background: rgba(234, 179, 8, 0.1);
  padding: 1px var(--space-1);
  border-radius: var(--radius-sm);
}

.banner-fade-enter-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.banner-fade-enter-from {
  opacity: 0;
  transform: translateY(-4px);
}

.view-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-3);
  padding: var(--space-16, 80px) 0;
}

.view-spinner {
  width: 32px;
  height: 32px;
  border: 2.5px solid var(--color-border-default);
  border-top-color: var(--color-accent);
  border-radius: 50%;
  animation: view-spin 0.7s linear infinite;
}

@keyframes view-spin {
  to { transform: rotate(360deg); }
}

.view-loading-label {
  font-size: 13px;
  color: var(--color-text-muted);
  font-weight: 500;
}

.not-found {
  text-align: center;
  padding: var(--space-12);
  color: var(--color-text-muted);
}

.not-found button {
  margin-top: var(--space-4);
  background: var(--color-surface-raised);
  color: var(--color-text-primary);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-4);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.not-found button:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-border-hover);
}

.pr-detail-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: var(--space-4);
}

.pr-title-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin-bottom: var(--space-2);
}

.pr-title {
  font-size: 16px;
  font-weight: 600;
  letter-spacing: var(--text-heading-tracking);
  line-height: var(--text-heading-leading);
  color: var(--color-text-primary);
}

.pr-meta-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: 13px;
  color: var(--color-text-muted);
}

.pr-number { font-family: var(--font-mono); }
.pr-author { font-weight: 500; color: var(--color-text-secondary); }
.separator { color: var(--color-border-default); }

.timer-badge {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  background: rgba(59, 130, 246, 0.15);
  color: var(--color-status-info);
  font-size: 11px;
  font-weight: 600;
  font-family: var(--font-mono);
  padding: 2px var(--space-2);
  border-radius: var(--radius-full);
  border: 1px solid rgba(59, 130, 246, 0.2);
}

.deployment-row {
  margin-top: var(--space-2);
}

.draft-badge {
  background: rgba(100, 116, 139, 0.2);
  color: var(--color-text-muted);
  padding: 1px var(--space-2);
  border-radius: var(--radius-full);
  font-size: 11px;
}

.pr-state {
  font-weight: 600;
  font-size: 12px;
  padding: 2px var(--space-2);
  border-radius: var(--radius-full);
}

.pr-state.open { background: rgba(34, 197, 94, 0.2); color: var(--color-status-success); }
.pr-state.merged { background: rgba(139, 92, 246, 0.2); color: #a78bfa; }
.pr-state.closed { background: rgba(220, 38, 38, 0.2); color: var(--color-status-danger); }

/* Pulse animation when a PR has just been merged */
@keyframes merge-pulse {
  0%, 100% { box-shadow: 0 0 0 0 rgba(139, 92, 246, 0.4); }
  50% { box-shadow: 0 0 12px 4px rgba(139, 92, 246, 0.6); }
}

.pr-state.merge-pulse {
  animation: merge-pulse 0.75s ease-in-out 3;
}

.btn-session {
  background: rgba(59, 130, 246, 0.15);
  color: var(--color-status-info);
  font-weight: 500;
  white-space: nowrap;
  border: 1px solid rgba(59, 130, 246, 0.25);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-4);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-session:hover {
  background: rgba(59, 130, 246, 0.25);
  border-color: rgba(59, 130, 246, 0.4);
}

.btn-session:active {
  transform: scale(0.97);
}

/* Worktree dropdown in header actions */
.worktree-dropdown-wrapper {
  position: relative;
}

.btn-worktree-toggle {
  background: rgba(139, 92, 246, 0.15);
  color: #a78bfa;
  font-weight: 500;
  white-space: nowrap;
  border: 1px solid rgba(139, 92, 246, 0.25);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-4);
  cursor: pointer;
  transition: all var(--transition-fast);
  display: flex;
  align-items: center;
  gap: var(--space-1);
}

.btn-worktree-toggle:hover {
  background: rgba(139, 92, 246, 0.25);
  border-color: rgba(139, 92, 246, 0.4);
}

.btn-worktree-toggle:active {
  transform: scale(0.97);
}

.dropdown-chevron {
  font-size: 11px;
  transition: transform var(--transition-fast);
}

.dropdown-chevron.open {
  transform: rotate(180deg);
}

.worktree-dropdown {
  position: absolute;
  top: calc(100% + var(--space-2));
  right: 0;
  z-index: 50;
  min-width: 320px;
}

.dropdown-fade-enter-active,
.dropdown-fade-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.dropdown-fade-enter-from,
.dropdown-fade-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

.btn-github {
  background: var(--color-surface-raised);
  color: var(--color-text-primary);
  font-weight: 500;
  white-space: nowrap;
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-4);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-github:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-border-hover);
}

.btn-github:active {
  transform: scale(0.97);
}

.btn-github:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

/* Header actions row — focus toggle + GitHub link */
.header-actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-shrink: 0;
}

.btn-focus {
  background: var(--color-surface-raised);
  color: var(--color-text-secondary);
  font-weight: 500;
  font-size: 13px;
  white-space: nowrap;
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-4);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-focus:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-border-hover);
  color: var(--color-text-primary);
}

.btn-focus.active {
  background: var(--color-accent-muted);
  color: var(--color-accent);
  border-color: var(--color-accent);
}

.btn-focus:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.detail-section {
  margin-bottom: var(--space-3);
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-4);
}

.section-title {
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.03em;
  text-transform: uppercase;
  color: var(--color-text-muted);
  margin-bottom: var(--space-2);
}

.change-stats {
  display: flex;
  gap: var(--space-6);
}

.stat {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.stat-value {
  font-size: 22px;
  font-weight: 700;
  font-family: var(--font-mono);
  line-height: 1;
}

.stat-label {
  font-size: 12px;
  color: var(--color-text-muted);
}

.additions { color: var(--color-status-success); }
.deletions { color: var(--color-status-danger); }

.branch-info {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  font-size: 13px;
}

.branch-info code {
  background: var(--color-surface-raised);
  padding: var(--space-1) var(--space-3);
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  font-size: 12px;
}

.branch-arrow { color: var(--color-text-muted); }

.labels {
  display: flex;
  gap: var(--space-2);
  flex-wrap: wrap;
}


.timeline {
  display: flex;
  flex-direction: column;
}

.timeline-item {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
  padding: var(--space-2) 0;
  border-bottom: 1px solid var(--color-border-default);
}

.timeline-item:last-child {
  border-bottom: none;
}

.timeline-label { color: var(--color-text-muted); }
.timeline-value { color: var(--color-text-secondary); font-family: var(--font-mono); font-size: 12px; }

.description-section {
  padding: 0;
}

.description-toggle {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: var(--space-4);
  background: none;
  border: none;
  cursor: pointer;
  color: inherit;
  text-align: left;
  transition: background var(--transition-fast);
  border-radius: var(--radius-lg);
}

.description-toggle:hover {
  background: var(--color-surface-hover);
}

.description-toggle .section-title {
  margin-bottom: 0;
}

.toggle-icon {
  font-size: 14px;
  color: var(--color-text-muted);
  transition: transform var(--transition-fast);
  display: inline-block;
}

.toggle-icon.expanded {
  transform: rotate(90deg);
}

.description-content {
  padding: 0 var(--space-5) var(--space-5);
  border-top: 1px solid var(--color-border-default);
  margin-top: 0;
  padding-top: var(--space-4);
}

.ci-section {
  margin-bottom: var(--space-6);
}

.collapsible-section {
  padding: 0;
}

.collapsible-content {
  padding: 0 var(--space-5) var(--space-5);
  border-top: 1px solid var(--color-border-default);
  padding-top: var(--space-4);
}

.section-loading {
  text-align: center;
  color: var(--color-text-muted);
  font-size: 13px;
  padding: var(--space-4);
}

.section-empty {
  text-align: center;
  color: var(--color-text-muted);
  font-size: 13px;
  padding: var(--space-4);
}


.detail-tabs {
  margin-bottom: var(--space-4);
}

.detail-main {
  display: flex;
  flex-direction: column;
}

.detail-main--full {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.detail-sidebar {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
  align-self: flex-start;
  position: sticky;
  top: 0;
}

/* Remove individual panel top margins — sidebar gap handles spacing */
.detail-sidebar > :first-child {
  margin-top: 0;
}

.github-actions {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.merge-warning-inline {
  background: rgba(234, 179, 8, 0.1);
  border: 1px solid rgba(234, 179, 8, 0.3);
  border-radius: var(--radius-md);
  padding: var(--space-3);
  font-size: 13px;
  color: var(--color-text-secondary);
}

.merge-warning-inline strong {
  color: var(--color-status-warning);
}

.merge-warning-inline code {
  background: rgba(234, 179, 8, 0.15);
  padding: 1px var(--space-1);
  border-radius: var(--radius-sm);
  font-size: 12px;
}

.approve-body-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.approve-body-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.approve-body-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
}

.approve-body-input {
  width: 100%;
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-3);
  color: var(--color-text-primary);
  font-size: 13px;
  font-family: var(--font-sans);
  resize: vertical;
  min-height: 48px;
  transition: border-color var(--transition-fast);
}

.approve-body-input:focus {
  border-color: var(--color-border-focus);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.action-buttons {
  display: flex;
  gap: var(--space-3);
}

.btn-approve {
  flex: 1;
  background: rgba(34, 197, 94, 0.2);
  color: var(--color-status-success);
  font-weight: 600;
  padding: var(--space-3);
  border-radius: var(--radius-md);
  border: 1px solid rgba(34, 197, 94, 0.3);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-approve:hover:not(:disabled) {
  background: rgba(34, 197, 94, 0.3);
  border-color: rgba(34, 197, 94, 0.5);
}

.btn-approve:active:not(:disabled) {
  transform: scale(0.97);
}

.btn-approve:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-approve:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.btn-merge {
  flex: 1;
  background: rgba(139, 92, 246, 0.2);
  color: #a78bfa;
  font-weight: 600;
  padding: var(--space-3);
  border-radius: var(--radius-md);
  border: 1px solid rgba(139, 92, 246, 0.3);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-merge:hover:not(:disabled) {
  background: rgba(139, 92, 246, 0.3);
  border-color: rgba(139, 92, 246, 0.5);
}

.btn-merge:active:not(:disabled) {
  transform: scale(0.97);
}

.btn-merge:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-merge:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

/* Review Checklist */
.checklist-panel {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  overflow: hidden;
}

.checklist-toggle {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: var(--space-4) var(--space-5);
  background: none;
  border: none;
  cursor: pointer;
  color: inherit;
  text-align: left;
  transition: background var(--transition-fast);
}

.checklist-toggle:hover {
  background: var(--color-surface-hover);
}

.checklist-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.checklist-items {
  display: flex;
  flex-direction: column;
  padding: 0 var(--space-5) var(--space-4);
  gap: var(--space-2);
}

.checklist-item {
  display: flex;
  align-items: flex-start;
  gap: var(--space-3);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background var(--transition-fast);
  user-select: none;
}

.checklist-item:hover {
  background: var(--color-surface-hover);
}

.checklist-item.checked .checklist-text {
  color: var(--color-text-muted);
  text-decoration: line-through;
}

.checklist-checkbox {
  appearance: none;
  width: 16px;
  height: 16px;
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-sm);
  background: var(--color-surface-input);
  flex-shrink: 0;
  margin-top: 1px;
  cursor: pointer;
  transition: all var(--transition-fast);
  position: relative;
}

.checklist-checkbox:checked {
  background: var(--color-accent);
  border-color: var(--color-accent);
}

.checklist-checkbox:checked::after {
  content: '';
  position: absolute;
  left: 4px;
  top: 1px;
  width: 5px;
  height: 9px;
  border: solid var(--color-text-inverse);
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

.checklist-checkbox:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.checklist-text {
  font-size: 13px;
  color: var(--color-text-secondary);
  line-height: 1.4;
  transition: color var(--transition-fast);
}

/* Cache status bar — shown inside collapsible sections */
.cache-status-bar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--space-2);
  margin-bottom: var(--space-2);
}

.last-fetched-label {
  font-size: 11px;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
}

.refresh-btn-inline {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  padding: 0;
  background: none;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.refresh-btn-inline:hover {
  color: var(--color-text-primary);
  background: var(--color-surface-hover);
  border-color: var(--color-border-default);
}

.refresh-btn-inline:focus-visible {
  outline: 2px solid var(--color-border-focus);
  outline-offset: 2px;
}

/* Linked Issues Panel */
.linked-issues-panel {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-4) var(--space-5);
}

.linked-issues-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0 0 var(--space-3) 0;
}

.linked-issues-loading {
  text-align: center;
  color: var(--color-text-muted);
  font-size: 13px;
  padding: var(--space-2);
}

.linked-issues-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.linked-issue-card {
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-3);
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.linked-issue-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.linked-issue-number {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--color-text-muted);
}

.linked-issue-state {
  font-size: 11px;
  font-weight: 600;
  padding: 1px var(--space-2);
  border-radius: var(--radius-full);
}

.linked-issue-state.state-open {
  background: rgba(34, 197, 94, 0.2);
  color: var(--color-status-success);
}

.linked-issue-state.state-closed {
  background: rgba(139, 92, 246, 0.2);
  color: #a78bfa;
}

.linked-issue-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
  text-decoration: none;
  line-height: 1.4;
  cursor: pointer;
}

.linked-issue-title:hover {
  color: var(--color-accent);
  text-decoration: underline;
}

.linked-issue-labels {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-1);
}


.linked-issue-assignees {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  flex-wrap: wrap;
  font-size: 12px;
}

.assignee-label {
  color: var(--color-text-muted);
}

.linked-issue-assignee {
  color: var(--color-text-secondary);
  font-weight: 500;
}
</style>
