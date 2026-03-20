<script setup lang="ts">
import { ref, computed } from 'vue'
import type { AuthorStats } from '../types'
import AuthorAvatar from './AuthorAvatar.vue'

const props = defineProps<{
  authors: AuthorStats[]
}>()

type SortKey = 'author' | 'pr_count' | 'avg_size' | 'merged_count' | 'reviewed_count'

const sortBy = ref<SortKey>('pr_count')
const sortAsc = ref(false)

const sortedAuthors = computed(() => {
  const sorted = [...props.authors]
  sorted.sort((a, b) => {
    let cmp = 0
    switch (sortBy.value) {
      case 'author':
        cmp = a.author.localeCompare(b.author)
        break
      case 'pr_count':
        cmp = a.pr_count - b.pr_count
        break
      case 'avg_size':
        cmp = (a.avg_additions + a.avg_deletions) - (b.avg_additions + b.avg_deletions)
        break
      case 'merged_count':
        cmp = a.merged_count - b.merged_count
        break
      case 'reviewed_count':
        cmp = a.reviewed_count - b.reviewed_count
        break
    }
    return sortAsc.value ? cmp : -cmp
  })
  return sorted
})

function toggleSort(key: SortKey) {
  if (sortBy.value === key) {
    sortAsc.value = !sortAsc.value
  } else {
    sortBy.value = key
    sortAsc.value = false
  }
}

function sortIndicator(key: SortKey): string {
  if (sortBy.value !== key) return ''
  return sortAsc.value ? ' \u2191' : ' \u2193'
}

function formatAvgSize(a: AuthorStats): string {
  const total = Math.round(a.avg_additions + a.avg_deletions)
  return `+${Math.round(a.avg_additions)} / -${Math.round(a.avg_deletions)} (${total})`
}
</script>

<template>
  <div class="author-table-wrapper">
    <table class="author-table">
      <thead>
        <tr>
          <th class="sortable" @click="toggleSort('author')">Author{{ sortIndicator('author') }}</th>
          <th class="sortable" @click="toggleSort('pr_count')">PRs{{ sortIndicator('pr_count') }}</th>
          <th class="sortable" @click="toggleSort('avg_size')">Avg Size{{ sortIndicator('avg_size') }}</th>
          <th class="sortable" @click="toggleSort('merged_count')">Merged{{ sortIndicator('merged_count') }}</th>
          <th class="sortable" @click="toggleSort('reviewed_count')">Reviewed{{ sortIndicator('reviewed_count') }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="author in sortedAuthors" :key="author.author" class="author-row">
          <td class="col-author">
            <AuthorAvatar :username="author.author" />
            <span class="author-name">{{ author.author }}</span>
          </td>
          <td class="col-num">{{ author.pr_count }}</td>
          <td class="col-size">{{ formatAvgSize(author) }}</td>
          <td class="col-num">{{ author.merged_count }}</td>
          <td class="col-num">{{ author.reviewed_count }}</td>
        </tr>
      </tbody>
    </table>
    <div v-if="authors.length === 0" class="empty-state">
      No author data available. Sync some repositories first.
    </div>
  </div>
</template>

<style scoped>
.author-table-wrapper {
  overflow-x: auto;
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
}

.author-table {
  width: 100%;
  border-collapse: collapse;
}

.author-table thead {
  position: sticky;
  top: 0;
  z-index: 2;
}

.author-table th {
  text-align: left;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: var(--space-3);
  border-bottom: 1px solid var(--color-border-default);
  white-space: nowrap;
  user-select: none;
  background: var(--color-surface-raised);
}

.author-table th.sortable {
  cursor: pointer;
  transition: color var(--transition-fast);
}

.author-table th.sortable:hover {
  color: var(--color-accent);
}

.author-row {
  transition: background var(--transition-fast);
}

.author-row:hover {
  background: var(--color-surface-hover);
}

.author-row td {
  padding: var(--space-3);
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  font-size: 13px;
  vertical-align: middle;
}

.author-row:last-child td {
  border-bottom: none;
}

.col-author {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-weight: 500;
  color: var(--color-text-primary);
}

.author-name {
  color: var(--color-text-secondary);
}

.col-num {
  font-family: var(--font-mono);
  font-size: 13px;
  color: var(--color-text-secondary);
}

.col-size {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--color-text-secondary);
  white-space: nowrap;
}

.empty-state {
  padding: var(--space-8);
  text-align: center;
  color: var(--color-text-muted);
  font-size: 14px;
}
</style>
