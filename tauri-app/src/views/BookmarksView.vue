<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { Bug, HelpCircle, Lightbulb, AlertOctagon, StickyNote, Check, Trash2, Filter } from 'lucide-vue-next'
import { useGlobalBookmarks } from '../composables/useBookmarks'
import type { BookmarkWithContext, BookmarkCategory } from '../types'
import { SBreadcrumbs } from '@stuntrocket/ui'

const router = useRouter()
const { allBookmarks, loading, error, fetchAllBookmarks, toggleResolved, removeBookmark } = useGlobalBookmarks()

const categoryFilter = ref<BookmarkCategory | 'all'>('all')
const resolvedFilter = ref<'all' | 'active' | 'resolved'>('all')

const categoryOptions: { value: BookmarkCategory | 'all'; label: string }[] = [
  { value: 'all', label: 'All categories' },
  { value: 'note', label: 'Note' },
  { value: 'bug', label: 'Bug' },
  { value: 'question', label: 'Question' },
  { value: 'suggestion', label: 'Suggestion' },
  { value: 'blocker', label: 'Blocker' },
]

const resolvedOptions: { value: 'all' | 'active' | 'resolved'; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'active', label: 'Active' },
  { value: 'resolved', label: 'Resolved' },
]

const categoryIcons: Record<BookmarkCategory, typeof Bug> = {
  note: StickyNote,
  bug: Bug,
  question: HelpCircle,
  suggestion: Lightbulb,
  blocker: AlertOctagon,
}

const categoryColours: Record<BookmarkCategory, string> = {
  note: 'var(--color-text-muted)',
  bug: 'var(--color-status-danger)',
  question: 'var(--color-status-info)',
  suggestion: 'var(--color-status-success)',
  blocker: 'var(--color-status-warning)',
}

const filteredBookmarks = computed(() => {
  let result = allBookmarks.value
  if (categoryFilter.value !== 'all') {
    result = result.filter(b => b.category === categoryFilter.value)
  }
  if (resolvedFilter.value === 'active') {
    result = result.filter(b => !b.resolved)
  } else if (resolvedFilter.value === 'resolved') {
    result = result.filter(b => b.resolved)
  }
  return result
})

/** Group filtered bookmarks by PR. */
const groupedBookmarks = computed(() => {
  const groups: { prId: number; prNumber: number; prTitle: string; repoName: string; bookmarks: BookmarkWithContext[] }[] = []
  const map = new Map<number, typeof groups[0]>()

  for (const b of filteredBookmarks.value) {
    let group = map.get(b.pr_id)
    if (!group) {
      group = {
        prId: b.pr_id,
        prNumber: b.pr_number,
        prTitle: b.pr_title,
        repoName: b.repo_name,
        bookmarks: [],
      }
      map.set(b.pr_id, group)
      groups.push(group)
    }
    group.bookmarks.push(b)
  }
  return groups
})

onMounted(() => {
  fetchAllBookmarks()
})

function navigateToBookmark(bookmark: BookmarkWithContext) {
  router.push({
    name: 'pr-detail',
    params: { id: bookmark.pr_id },
    query: {
      bookmarkFile: bookmark.file_path,
      bookmarkLine: bookmark.line_start?.toString() ?? '',
    },
  })
}

function formatLineRange(bookmark: BookmarkWithContext): string {
  if (bookmark.line_start == null) return ''
  if (bookmark.line_end == null || bookmark.line_end === bookmark.line_start) {
    return `:L${bookmark.line_start}`
  }
  return `:L${bookmark.line_start}-${bookmark.line_end}`
}

function formatRelativeTime(dateStr: string): string {
  const diff = Date.now() - new Date(dateStr).getTime()
  const minutes = Math.floor(diff / 60000)
  if (minutes < 1) return 'just now'
  if (minutes < 60) return `${minutes}m ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  if (days < 30) return `${days}d ago`
  return `${Math.floor(days / 7)}w ago`
}

const breadcrumbItems = [{ label: 'Bookmarks' }]
</script>

<template>
  <div class="bookmarks-view">
    <SBreadcrumbs :segments="breadcrumbItems" @navigate="$router.push($event)" />

    <div class="bookmarks-view-header">
      <h1 class="bookmarks-view-title">Bookmarks</h1>
      <span v-if="!loading" class="bookmarks-total-count">
        {{ filteredBookmarks.length }} bookmark{{ filteredBookmarks.length === 1 ? '' : 's' }}
      </span>
    </div>

    <div class="bookmarks-filters">
      <div class="filter-group">
        <Filter :size="14" class="filter-icon" />
        <select v-model="categoryFilter" class="filter-select">
          <option v-for="opt in categoryOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </option>
        </select>
        <select v-model="resolvedFilter" class="filter-select">
          <option v-for="opt in resolvedOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </option>
        </select>
      </div>
    </div>

    <div v-if="loading" class="bookmarks-view-loading">
      <div class="view-spinner" />
      <span>Loading bookmarks...</span>
    </div>

    <div v-else-if="error" class="bookmarks-view-error">{{ error }}</div>

    <div v-else-if="groupedBookmarks.length === 0" class="bookmarks-view-empty">
      <p>No bookmarks found{{ categoryFilter !== 'all' || resolvedFilter !== 'all' ? ' matching your filters' : '' }}.</p>
    </div>

    <div v-else class="bookmarks-groups">
      <section
        v-for="group in groupedBookmarks"
        :key="group.prId"
        class="bookmark-pr-group"
      >
        <div class="bookmark-pr-group-header">
          <router-link
            class="bookmark-pr-link"
            :to="{ name: 'pr-detail', params: { id: group.prId } }"
          >
            <span class="bookmark-pr-number">#{{ group.prNumber }}</span>
            <span class="bookmark-pr-title">{{ group.prTitle }}</span>
          </router-link>
          <span class="bookmark-pr-repo">{{ group.repoName }}</span>
        </div>

        <div class="bookmark-pr-items">
          <div
            v-for="bookmark in group.bookmarks"
            :key="bookmark.id"
            class="global-bookmark-card"
            :class="{ 'bookmark-resolved': bookmark.resolved }"
            @click="navigateToBookmark(bookmark)"
          >
            <div class="global-bookmark-header">
              <span
                class="bookmark-category-icon"
                :style="{ color: categoryColours[bookmark.category] }"
                :title="bookmark.category"
              >
                <component :is="categoryIcons[bookmark.category]" :size="14" />
              </span>
              <code class="global-bookmark-file">
                {{ bookmark.file_path }}{{ formatLineRange(bookmark) }}
              </code>
              <span class="global-bookmark-time">{{ formatRelativeTime(bookmark.created_at) }}</span>
              <button
                class="global-bookmark-resolve"
                :title="bookmark.resolved ? 'Mark as unresolved' : 'Mark as resolved'"
                @click.stop="toggleResolved(bookmark.id)"
              >
                <Check :size="14" :class="{ 'resolved-check': bookmark.resolved }" />
              </button>
              <button
                class="global-bookmark-delete"
                title="Delete bookmark"
                @click.stop="removeBookmark(bookmark.id)"
              >
                <Trash2 :size="13" />
              </button>
            </div>
            <p v-if="bookmark.note" class="global-bookmark-note">
              {{ bookmark.note.length > 120 ? bookmark.note.slice(0, 120) + '...' : bookmark.note }}
            </p>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.bookmarks-view {
  padding: var(--space-6);
  max-width: 900px;
}

.bookmarks-view-header {
  display: flex;
  align-items: baseline;
  gap: var(--space-3);
  margin-bottom: var(--space-4);
}

.bookmarks-view-title {
  font-size: 24px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin: 0;
}

.bookmarks-total-count {
  font-size: 14px;
  color: var(--color-text-muted);
}

.bookmarks-filters {
  margin-bottom: var(--space-4);
}

.filter-group {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.filter-icon {
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.filter-select {
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-1) var(--space-3);
  color: var(--color-text-primary);
  font-size: 13px;
  cursor: pointer;
  transition: border-color var(--transition-fast);
}

.filter-select:focus {
  border-color: var(--color-border-focus);
  outline: none;
}

.bookmarks-view-loading {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  color: var(--color-text-muted);
  font-size: 14px;
  padding: var(--space-6);
}

.view-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--color-border-default);
  border-top-color: var(--color-accent);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.bookmarks-view-error {
  color: var(--color-status-danger);
  font-size: 14px;
  padding: var(--space-4);
}

.bookmarks-view-empty {
  color: var(--color-text-muted);
  font-size: 14px;
  text-align: center;
  padding: var(--space-8);
}

.bookmarks-groups {
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
}

.bookmark-pr-group {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  overflow: hidden;
}

.bookmark-pr-group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-3) var(--space-4);
  background: rgba(255, 255, 255, 0.02);
  border-bottom: 1px solid var(--color-border-default);
}

.bookmark-pr-link {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  text-decoration: none;
  color: var(--color-text-primary);
  font-weight: 600;
  font-size: 14px;
  transition: color var(--transition-fast);
}

.bookmark-pr-link:hover {
  color: var(--color-accent);
}

.bookmark-pr-number {
  font-family: var(--font-mono);
  font-size: 13px;
  color: var(--color-accent);
}

.bookmark-pr-title {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.bookmark-pr-repo {
  font-size: 12px;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
  flex-shrink: 0;
}

.bookmark-pr-items {
  display: flex;
  flex-direction: column;
}

.global-bookmark-card {
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.global-bookmark-card:last-child {
  border-bottom: none;
}

.global-bookmark-card:hover {
  background: rgba(20, 184, 166, 0.04);
}

.bookmark-resolved {
  opacity: 0.6;
}

.bookmark-resolved .global-bookmark-note {
  text-decoration: line-through;
}

.global-bookmark-header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.bookmark-category-icon {
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.global-bookmark-file {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--color-accent);
  background: rgba(20, 184, 166, 0.1);
  padding: 1px var(--space-2);
  border-radius: var(--radius-sm);
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.global-bookmark-time {
  font-size: 11px;
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.global-bookmark-resolve {
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 0 2px;
  display: flex;
  align-items: center;
  transition: color var(--transition-fast);
  flex-shrink: 0;
}

.global-bookmark-resolve:hover {
  color: var(--color-status-success);
}

.resolved-check {
  color: var(--color-status-success);
}

.global-bookmark-delete {
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 0 2px;
  display: flex;
  align-items: center;
  transition: color var(--transition-fast);
  flex-shrink: 0;
}

.global-bookmark-delete:hover {
  color: var(--color-status-danger);
}

.global-bookmark-note {
  font-size: 13px;
  color: var(--color-text-secondary);
  margin: var(--space-1) 0 0;
  line-height: 1.4;
}
</style>
