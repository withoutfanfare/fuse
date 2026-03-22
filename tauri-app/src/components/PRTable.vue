<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { AlertTriangle, GitPullRequest, Search, Bookmark, BookmarkCheck } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'
import type { PullRequest, ReviewStatus, Bookmark as BookmarkType } from '../types'
import type { SortKey } from '../stores/filters'
import { computeRiskScore } from '../composables/useRiskScore'
import { useHoverPreview } from '../composables/useHoverPreview'
import { usePullRequestsStore } from '../stores/pullRequests'
import RiskGauge from './RiskGauge.vue'
import SizeBar from './SizeBar.vue'
import AuthorAvatar from './AuthorAvatar.vue'
import { SDataTable, SEmptyState, SIconButton } from '@stuntrocket/ui'
import type { SDataTableColumn } from '@stuntrocket/ui'
import SkeletonPRTableRow from './skeletons/SkeletonPRTableRow.vue'
import PRHoverPreview from './PRHoverPreview.vue'
import QuickStatusPopover from './QuickStatusPopover.vue'

const props = withDefaults(defineProps<{
  prs: PullRequest[]
  loading?: boolean
  hasFilters?: boolean
  selectable?: boolean
  selectedIds?: Set<number>
  sortBy?: SortKey
  sortAsc?: boolean
  /** Index of the keyboard-focused row, or -1 for none */
  focusedIndex?: number
}>(), {
  loading: false,
  hasFilters: false,
  selectable: false,
  sortBy: 'risk',
  sortAsc: false,
  focusedIndex: -1,
})

const emit = defineEmits<{
  'open-detail': [id: number]
  'update:selectedIds': [ids: Set<number>]
  'update:sortBy': [key: SortKey]
  'update:sortAsc': [asc: boolean]
}>()

const allSelected = computed(() => {
  if (!props.selectedIds || props.prs.length === 0) return false
  return props.prs.every(pr => props.selectedIds!.has(pr.id))
})

function toggleSelectAll() {
  if (!props.selectedIds) return
  const next = new Set(props.selectedIds)
  if (allSelected.value) {
    props.prs.forEach(pr => next.delete(pr.id))
  } else {
    props.prs.forEach(pr => next.add(pr.id))
  }
  emit('update:selectedIds', next)
}

function toggleSelect(prId: number, event: Event) {
  event.stopPropagation()
  if (!props.selectedIds) return
  const next = new Set(props.selectedIds)
  if (next.has(prId)) {
    next.delete(prId)
  } else {
    next.add(prId)
  }
  emit('update:selectedIds', next)
}

const tableWrapper = ref<HTMLElement | null>(null)
const scrolled = ref(false)

function onScroll() {
  if (tableWrapper.value) {
    scrolled.value = tableWrapper.value.scrollTop > 0
  }
}

onMounted(() => {
  tableWrapper.value?.addEventListener('scroll', onScroll, { passive: true })
})

onUnmounted(() => {
  tableWrapper.value?.removeEventListener('scroll', onScroll)
})

/**
 * Scroll the focused row into view.
 * We query the DOM for the marker <td> carrying data-row-index, then
 * scroll its parent <tr> into view.
 */
watch(() => props.focusedIndex, (idx) => {
  if (idx >= 0) {
    nextTick(() => {
      const marker = tableWrapper.value?.querySelector<HTMLElement>(`td[data-row-index="${idx}"]`)
      marker?.closest('tr')?.scrollIntoView({ block: 'nearest' })
    })
  }
})

/** Memoised risk score map — only recomputes when the PR data changes, not on focusedIndex shifts. */
const riskScoreMap = computed(() => {
  const map = new Map<number, number>()
  for (const pr of props.prs) {
    map.set(pr.id, computeRiskScore(pr))
  }
  return map
})

const sortedPrs = computed(() => {
  const scores = riskScoreMap.value
  const withScores = props.prs.map(pr => ({
    ...pr,
    _riskScore: scores.get(pr.id) ?? 0,
  }))
  withScores.sort((a, b) => {
    let cmp = 0
    switch (props.sortBy) {
      case 'risk':
        cmp = b._riskScore - a._riskScore
        break
      case 'updated':
        cmp = Date.parse(b.updated_at) - Date.parse(a.updated_at)
        break
      case 'age':
        cmp = Date.parse(a.created_at) - Date.parse(b.created_at)
        break
      case 'size':
        cmp = (b.additions + b.deletions) - (a.additions + a.deletions)
        break
    }
    return props.sortAsc ? -cmp : cmp
  })
  return withScores
})

/**
 * Column definitions for SDataTable.
 *
 * These define the header structure. Sortable columns emit sort events via
 * SDataTable's built-in update:sortKey / update:sortDir. Non-sortable columns
 * (PR, Author, Branch, Status, Review, Bookmark) use sortable: false.
 *
 * We use the #header slot to render our custom header (including the select-all
 * checkbox) because SDataTable's built-in selectable uses index-based selection
 * that doesn't match our ID-based model.
 */
const columns = computed<SDataTableColumn[]>(() => {
  const cols: SDataTableColumn[] = []
  if (props.selectable) {
    cols.push({ key: '_select', label: '', sortable: false, width: '40px', align: 'center' })
  }
  cols.push(
    { key: 'risk', label: 'Risk', sortable: true },
    { key: 'pr', label: 'PR', sortable: false },
    { key: 'author', label: 'Author', sortable: false },
    { key: 'branch', label: 'Branch', sortable: false },
    { key: 'size', label: 'Size', sortable: true },
    { key: 'age', label: 'Age', sortable: true },
    { key: 'status', label: 'Status', sortable: false },
    { key: 'review', label: 'Review', sortable: false },
    { key: '_bookmark', label: '', sortable: false, width: '36px', align: 'center' },
  )
  return cols
})

/** Bridge SDataTable's sortKey string to our SortKey type */
function onSortKeyUpdate(key: string) {
  if (key === 'risk' || key === 'size' || key === 'age' || key === 'updated') {
    if (props.sortBy === key) {
      emit('update:sortAsc', !props.sortAsc)
    } else {
      emit('update:sortBy', key as SortKey)
      emit('update:sortAsc', false)
    }
  }
}

/** Bridge SDataTable's sortDir to our boolean sortAsc */
function onSortDirUpdate(dir: 'asc' | 'desc') {
  emit('update:sortAsc', dir === 'asc')
}

/** Map our boolean sortAsc to SDataTable's 'asc' | 'desc' */
const tableSortDir = computed<'asc' | 'desc'>(() => props.sortAsc ? 'asc' : 'desc')

function formatAge(createdAt: string): string {
  const hours = Math.floor((Date.now() - Date.parse(createdAt)) / 3_600_000)
  if (hours < 1) return '<1h'
  if (hours < 24) return `${hours}h`
  const days = Math.floor(hours / 24)
  return `${days}d`
}

function ageColourClass(createdAt: string): string {
  const hours = (Date.now() - Date.parse(createdAt)) / 3_600_000
  if (hours < 24) return 'age-fresh'
  if (hours < 72) return 'age-normal'
  if (hours < 168) return 'age-aging'
  if (hours < 336) return 'age-old'
  return 'age-stale'
}

function stateClass(pr: PullRequest): string {
  if (pr.merged_at) return 'state-merged'
  if (pr.closed_at) return 'state-closed'
  return 'state-open'
}

function isForbiddenTarget(pr: PullRequest): boolean {
  const base = pr.base_branch.toLowerCase()
  return base === 'main' || base === 'master'
}

/* --- Hover preview (Improvement 4) --- */
const { hoveredId, previewPosition, isVisible: hoverVisible, onRowEnter, onRowMove, onRowLeave } = useHoverPreview(400)

const hoveredPr = computed(() => {
  if (!hoveredId.value) return null
  return sortedPrs.value.find(p => p.id === hoveredId.value) ?? null
})

/* --- Inline quick-status popover (Improvement 6) --- */
const prStore = usePullRequestsStore()
const quickStatusPrId = ref<number | null>(null)
const quickStatusAnchorRect = ref<DOMRect | null>(null)

const quickStatusPr = computed(() => {
  if (!quickStatusPrId.value) return null
  return sortedPrs.value.find(p => p.id === quickStatusPrId.value) ?? null
})

function openQuickStatus(prId: number, event: MouseEvent) {
  event.stopPropagation()
  const target = event.currentTarget as HTMLElement
  quickStatusAnchorRect.value = target.getBoundingClientRect()
  quickStatusPrId.value = prId
}

function closeQuickStatus() {
  quickStatusPrId.value = null
  quickStatusAnchorRect.value = null
}

async function handleQuickStatusSelect(status: ReviewStatus) {
  if (!quickStatusPrId.value) return
  await prStore.updateReviewStatus(quickStatusPrId.value, status)
  /* Refresh the PR list so the badge updates */
  await prStore.fetchAll()
}

/* --- Bookmark quick-add (Phase 5.4) --- */
const bookmarkedPrIds = ref(new Set<number>())

onMounted(async () => {
  // Load bookmark counts for all visible PRs to show active state
  try {
    const allBookmarks = await invoke<BookmarkType[]>('list_all_bookmarks')
    bookmarkedPrIds.value = new Set(allBookmarks.map(b => b.pr_id))
  } catch {
    // Silently ignore — badge is optional
  }
})

async function handleQuickBookmark(prId: number, event: MouseEvent) {
  event.stopPropagation()
  try {
    await invoke('create_bookmark', {
      prId,
      filePath: '(general)',
      lineStart: null,
      lineEnd: null,
      note: '',
      category: 'note',
    })
    bookmarkedPrIds.value = new Set([...bookmarkedPrIds.value, prId])
  } catch {
    // Silently fail
  }
}

/**
 * Extract the PR id from a mouse event by walking up to the nearest
 * <td> with a data-pr-id attribute (the marker cell in each row).
 */
function getPrIdFromEvent(event: MouseEvent): number | null {
  const target = event.target as HTMLElement
  const marker = target.closest('td[data-pr-id]') ?? target.closest('tr')?.querySelector('td[data-pr-id]')
  if (!marker) return null
  return Number((marker as HTMLElement).dataset.prId)
}

/**
 * Row click via event delegation — SDataTable owns the <tr>, so we
 * delegate from the wrapper and identify the PR via data-pr-id on a <td>.
 */
function onTableClick(event: MouseEvent) {
  const target = event.target as HTMLElement
  /* Skip clicks on interactive elements handled elsewhere */
  if (target.closest('input[type="checkbox"], button, .review-badge--clickable, .col-bookmark, .col-select')) {
    return
  }
  const prId = getPrIdFromEvent(event)
  if (prId != null) {
    emit('open-detail', prId)
  }
}

/** Hover preview — enter */
function onTableMouseOver(event: MouseEvent) {
  const prId = getPrIdFromEvent(event)
  if (prId != null) {
    onRowEnter(prId, event)
  }
}

/** Hover preview — move */
function onTableMouseMove(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (target.closest('tbody tr')) {
    onRowMove(event)
  }
}

/** Hover preview — leave (only fires when actually leaving the row) */
function onTableMouseOut(event: MouseEvent) {
  const relatedTarget = event.relatedTarget as HTMLElement | null
  const fromTr = (event.target as HTMLElement).closest('tbody tr')
  const toTr = relatedTarget?.closest('tbody tr')
  if (fromTr && fromTr !== toTr) {
    onRowLeave()
  }
}
</script>

<template>
  <div
    ref="tableWrapper"
    class="pr-table-wrapper"
    :class="{ scrolled }"
    @click="onTableClick"
    @mouseover="onTableMouseOver"
    @mousemove="onTableMouseMove"
    @mouseout="onTableMouseOut"
  >
    <SDataTable
      :columns="columns"
      :rows="sortedPrs"
      :sort-key="sortBy"
      :sort-dir="tableSortDir"
      :loading="loading"
      :selectable="false"
      class="pr-data-table"
      @update:sort-key="onSortKeyUpdate"
      @update:sort-dir="onSortDirUpdate"
    >
      <!--
        Custom header: we override SDataTable's default <thead> content to include
        the select-all checkbox column and preserve our existing header styling with
        sort chevron indicators that match the app's design language.
      -->
      <template #header>
        <tr>
          <th v-if="selectable" class="col-select">
            <input
              type="checkbox"
              class="row-checkbox"
              :checked="allSelected"
              @change="toggleSelectAll"
            />
          </th>
          <th class="col-risk sortable" @click="onSortKeyUpdate('risk')">
            <span class="header-label">
              Risk
              <svg v-if="sortBy === 'risk'" class="sort-chevron" :class="{ 'sort-asc': sortAsc }" width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 5l3 3 3-3" /></svg>
            </span>
          </th>
          <th class="col-pr">PR</th>
          <th class="col-author">Author</th>
          <th class="col-branch">Branch</th>
          <th class="col-size sortable" @click="onSortKeyUpdate('size')">
            <span class="header-label">
              Size
              <svg v-if="sortBy === 'size'" class="sort-chevron" :class="{ 'sort-asc': sortAsc }" width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 5l3 3 3-3" /></svg>
            </span>
          </th>
          <th class="col-age sortable" @click="onSortKeyUpdate('age')">
            <span class="header-label">
              Age
              <svg v-if="sortBy === 'age'" class="sort-chevron" :class="{ 'sort-asc': sortAsc }" width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 5l3 3 3-3" /></svg>
            </span>
          </th>
          <th class="col-status">Status</th>
          <th class="col-review">Review</th>
          <th class="col-bookmark" title="Bookmark"></th>
        </tr>
      </template>

      <!-- Custom loading skeleton -->
      <template #loading>
        <SkeletonPRTableRow />
      </template>

      <!-- Empty state handled by SEmptyState outside the table -->
      <template #empty><span /></template>

      <!--
        Custom row rendering.

        Each row's first content <td> (col-risk or col-select) carries data-pr-id
        and data-row-index attributes for event delegation and scroll-to-focus.

        Selection and keyboard-focus styling use CSS :has() to target the parent
        <tr> based on data-selected / data-focused attributes on a child <td>.
        This works in Tauri v2's WKWebView (Safari 15.4+).
      -->
      <template #row="{ row: pr, index: idx }">
        <td
          v-if="selectable"
          class="col-select"
          :data-pr-id="pr.id"
          :data-row-index="idx"
          :data-selected="selectedIds?.has(pr.id) ? 'true' : undefined"
          :data-focused="idx === focusedIndex ? 'true' : undefined"
          @click.stop
        >
          <input
            type="checkbox"
            class="row-checkbox"
            :checked="selectedIds?.has(pr.id)"
            @change="toggleSelect(pr.id, $event)"
          />
        </td>
        <td
          class="col-risk"
          :data-pr-id="selectable ? undefined : pr.id"
          :data-row-index="selectable ? undefined : idx"
          :data-selected="!selectable && selectedIds?.has(pr.id) ? 'true' : undefined"
          :data-focused="!selectable && idx === focusedIndex ? 'true' : undefined"
        >
          <RiskGauge :score="pr._riskScore" :size="28" />
        </td>
        <td class="col-pr">
          <span class="pr-number">#{{ pr.number }}</span>
          <span class="pr-title-text">{{ pr.title }}</span>
        </td>
        <td class="col-author">
          <AuthorAvatar :username="pr.author" />
          {{ pr.author }}
        </td>
        <td class="col-branch">
          <code>{{ pr.head_branch }}</code>
          <span class="branch-arrow">&rarr;</span>
          <code :class="{ 'forbidden-target': isForbiddenTarget(pr) }">{{ pr.base_branch }}</code>
          <AlertTriangle v-if="isForbiddenTarget(pr)" :size="14" class="target-warn" title="PRs should target staging, not main/master" />
        </td>
        <td class="col-size">
          <span class="additions">+{{ pr.additions }}</span>
          <span class="deletions">-{{ pr.deletions }}</span>
          <span class="files">{{ pr.changed_files }}f</span>
          <SizeBar :additions="pr.additions" :deletions="pr.deletions" />
        </td>
        <td class="col-age" :class="ageColourClass(pr.created_at)">{{ formatAge(pr.created_at) }}</td>
        <td class="col-status">
          <span class="state-badge" :class="stateClass(pr)">
            {{ pr.merged_at ? 'Merged' : pr.closed_at ? 'Closed' : pr.is_draft ? 'Draft' : 'Open' }}
          </span>
          <span
            v-if="pr.mergeable === 'CONFLICTING'"
            class="conflict-badge-inline"
            title="This PR has merge conflicts"
          >Conflicts</span>
        </td>
        <td class="col-review">
          <span
            v-if="pr.review_status"
            class="review-badge review-badge--clickable"
            :class="[`review-${pr.review_status}`]"
            title="Click to change status"
            @click.stop="openQuickStatus(pr.id, $event)"
          >
            {{ (pr.review_status ?? '').replace(/_/g, ' ') }}
          </span>
          <span
            v-else
            class="review-badge review-badge--clickable review-pending"
            title="Click to change status"
            @click.stop="openQuickStatus(pr.id, $event)"
          >
            pending
          </span>
        </td>
        <td class="col-bookmark" @click.stop>
          <SIconButton
            size="sm"
            :class="{ 'bookmark-active': bookmarkedPrIds.has(pr.id) }"
            :title="bookmarkedPrIds.has(pr.id) ? 'PR has bookmarks' : 'Quick bookmark this PR'"
            @click="handleQuickBookmark(pr.id, $event)"
          >
            <component :is="bookmarkedPrIds.has(pr.id) ? BookmarkCheck : Bookmark" :size="14" />
          </SIconButton>
        </td>
      </template>
    </SDataTable>

    <SEmptyState
      v-if="!loading && prs.length === 0"
      :title="hasFilters ? 'No matches' : 'No pull requests'"
      :description="hasFilters ? 'No pull requests match the current filters. Try adjusting your selection.' : 'There are no pull requests to display yet.'"
    >
      <template #icon><component :is="hasFilters ? Search : GitPullRequest" :size="36" /></template>
    </SEmptyState>

    <!-- Hover preview card (Improvement 4) -->
    <PRHoverPreview
      v-if="hoverVisible && hoveredPr"
      :pr="hoveredPr"
      :position="previewPosition"
    />

    <!-- Inline quick-status popover (Improvement 6) -->
    <QuickStatusPopover
      v-if="quickStatusPrId !== null && quickStatusAnchorRect && quickStatusPr"
      :current-status="(quickStatusPr.review_status as ReviewStatus | null)"
      :anchor-rect="quickStatusAnchorRect"
      @select="handleQuickStatusSelect"
      @close="closeQuickStatus"
    />
  </div>
</template>

<style scoped>
.pr-table-wrapper {
  overflow-x: auto;
  overflow-y: auto;
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
}

.pr-table-wrapper.scrolled :deep(thead th) {
  box-shadow: var(--shadow-scroll);
}

/* Override SDataTable's default overflow-x:auto wrapper to avoid double scrollbar */
:deep(.pr-data-table) {
  overflow: visible !important;
}

/* Sticky header */
:deep(.pr-data-table thead) {
  position: sticky;
  top: 0;
  z-index: 2;
}

:deep(.pr-data-table th) {
  text-align: left;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: var(--space-3) var(--space-3);
  border-bottom: 1px solid var(--color-border-default);
  white-space: nowrap;
  user-select: none;
  background: var(--color-surface-raised);
}

:deep(.pr-data-table th.sortable) {
  cursor: pointer;
  transition: color var(--transition-fast);
}

:deep(.pr-data-table th.sortable:hover) {
  color: var(--color-accent);
}

.header-label {
  display: inline-flex;
  align-items: center;
  gap: 2px;
}

/* Sort direction chevron indicator */
.sort-chevron {
  display: inline-block;
  vertical-align: middle;
  margin-left: 2px;
  transition: transform var(--transition-fast);
}

.sort-chevron.sort-asc {
  transform: rotate(180deg);
}

/*
 * Row styling — SDataTable renders each <tr> with Tailwind classes.
 * We override those to match the app's custom-property design system.
 */
:deep(.pr-data-table tbody tr) {
  cursor: pointer;
  transition: background var(--transition-fast);
  border-bottom: none !important;
}

:deep(.pr-data-table tbody tr:hover) {
  background: var(--color-surface-hover) !important;
}

:deep(.pr-data-table tbody tr td) {
  padding: var(--space-3);
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  font-size: 13px;
  vertical-align: middle;
}

:deep(.pr-data-table tbody tr:last-child td) {
  border-bottom: none;
}

/*
 * Selection highlight — uses CSS :has() to target the parent <tr>
 * based on data-selected attribute on a child <td> marker cell.
 * WKWebView (macOS 13+) supports :has().
 */
:deep(.pr-data-table tbody tr:has(> td[data-selected="true"])) {
  background: rgba(20, 184, 166, 0.06);
}

:deep(.pr-data-table tbody tr:has(> td[data-selected="true"]):hover) {
  background: rgba(20, 184, 166, 0.1) !important;
}

/* Keyboard-focused row highlight */
:deep(.pr-data-table tbody tr:has(> td[data-focused="true"])) {
  outline: 2px solid var(--color-accent);
  outline-offset: -2px;
  background: rgba(20, 184, 166, 0.08);
}

:deep(.pr-data-table tbody tr:has(> td[data-focused="true"]):hover) {
  background: rgba(20, 184, 166, 0.12) !important;
}

.col-pr {
  max-width: 350px;
}

.pr-number {
  font-family: var(--font-mono);
  color: var(--color-text-muted);
  margin-right: var(--space-2);
  font-size: 12px;
}

.pr-title-text {
  font-weight: 500;
  color: var(--color-text-primary);
}

.col-author {
  color: var(--color-text-secondary);
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.col-branch code {
  font-size: 12px;
  color: var(--color-text-secondary);
  background: var(--color-surface-raised);
  padding: 2px var(--space-2);
  border-radius: var(--radius-sm);
}

.branch-arrow {
  color: var(--color-text-muted);
  font-size: 11px;
  margin: 0 var(--space-1);
}

.forbidden-target {
  background: rgba(234, 179, 8, 0.15) !important;
  color: var(--color-status-warning) !important;
  border: 1px solid rgba(234, 179, 8, 0.3);
}

.target-warn {
  font-size: 12px;
  margin-left: var(--space-1);
  cursor: help;
}

.col-size {
  font-family: var(--font-mono);
  font-size: 12px;
  white-space: nowrap;
}

.additions { color: var(--color-status-success); }
.deletions { color: var(--color-status-danger); margin: 0 var(--space-1); }
.files { color: var(--color-text-muted); }

.col-age {
  font-family: var(--font-mono);
}

/* PR age colour coding */
.age-fresh {
  color: var(--color-status-success);
}

.age-normal {
  color: var(--color-text-secondary);
}

.age-aging {
  color: var(--color-status-warning);
}

.age-old {
  color: var(--color-risk-high);
}

.age-stale {
  color: var(--color-status-danger);
}

.state-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 2px var(--space-2);
  border-radius: var(--radius-full);
}

.state-open { background: rgba(34, 197, 94, 0.2); color: var(--color-status-success); }
.state-merged { background: rgba(139, 92, 246, 0.2); color: #a78bfa; }
.state-closed { background: rgba(220, 38, 38, 0.2); color: var(--color-status-danger); }

.review-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 2px var(--space-2);
  border-radius: var(--radius-full);
  text-transform: capitalize;
}

.review-pending { background: rgba(100, 116, 139, 0.2); color: var(--color-text-muted); }
.review-in_progress { background: rgba(59, 130, 246, 0.2); color: var(--color-status-info); }
.review-reviewed { background: rgba(234, 179, 8, 0.2); color: var(--color-status-warning); }
.review-approved { background: rgba(34, 197, 94, 0.2); color: var(--color-status-success); }
.review-changes_requested { background: rgba(220, 38, 38, 0.2); color: var(--color-status-danger); }

/* Clickable review badge for quick-status popover */
.review-badge--clickable {
  cursor: pointer;
  transition: filter var(--transition-fast), box-shadow var(--transition-fast);
}

.review-badge--clickable:hover {
  filter: brightness(1.2);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
}

.col-select {
  width: 40px;
  text-align: center;
}

.row-checkbox {
  width: 16px;
  height: 16px;
  accent-color: var(--color-accent);
  cursor: pointer;
}

.conflict-badge-inline {
  display: inline-block;
  margin-left: var(--space-1);
  font-size: 11px;
  font-weight: 600;
  color: var(--color-status-danger);
  background: rgba(220, 38, 38, 0.2);
  padding: 1px var(--space-2);
  border-radius: var(--radius-full);
  cursor: help;
}

.col-bookmark {
  width: 36px;
  text-align: center;
}

.bookmark-active {
  color: var(--color-accent);
}

.empty-cell {
  padding: 0 !important;
  border: none !important;
}
</style>
