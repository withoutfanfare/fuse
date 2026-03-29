<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, inject } from 'vue'
import type { ShallowRef } from 'vue'
import { useRouter } from 'vue-router'
import { usePullRequestsStore } from '../stores/pullRequests'
import { useRepositoriesStore } from '../stores/repositories'
import { useFiltersStore, type SortKey } from '../stores/filters'
import { useGroupsStore } from '../stores/groups'
import type { PrListContext } from '../composables/useKeyboardShortcuts'
import PRTable from '../components/PRTable.vue'
import BatchActionBar from '../components/BatchActionBar.vue'
import GroupFilter from '../components/GroupFilter.vue'
import FilterPresetsBar from '../components/FilterPresetsBar.vue'
import OfflineBanner from '../components/OfflineBanner.vue'
import { SSearchInput, SSelect } from '@stuntrocket/ui'
import { useOfflineMode } from '../composables/useOfflineMode'
import { useLabelFilter } from '../composables/useLabelFilter'
import { useSyncHealth } from '../composables/useSyncHealth'
import SyncHealthBanner from '../components/SyncHealthBanner.vue'

const router = useRouter()
const prStore = usePullRequestsStore()
const repoStore = useRepositoriesStore()
const filters = useFiltersStore()
const groupsStore = useGroupsStore()
const filterGroupId = ref<number | null>(null)
const { isOnline, timeSinceSync } = useOfflineMode()
const { labels: allLabels, selectedLabels, hasSelection: hasLabelSelection, fetchLabels, toggleLabel, clearSelection: clearLabelSelection } = useLabelFilter()
const filterCiStatus = ref<string>('')
const { unhealthyRepos, hasIssues: hasSyncIssues, fetchHealth } = useSyncHealth()

/**
 * SSelect works with string modelValue, so bridge between
 * the numeric repo ID and the string value used by SSelect.
 */
const repoSelectValue = computed({
  get() {
    return filters.filterRepoId != null ? String(filters.filterRepoId) : ''
  },
  set(val: string) {
    filters.filterRepoId = val ? Number(val) : null
  },
})

const repoSelectOptions = computed(() => [
  { value: '', label: 'All Repositories' },
  ...repoStore.repos.map(r => ({ value: String(r.id), label: `${r.owner}/${r.name}` })),
])

onMounted(async () => {
  if (repoStore.repos.length === 0) await repoStore.fetchAll()
  await groupsStore.fetchAll()
  await prStore.fetchAll(filters.filterRepoId ?? undefined, undefined)
  await fetchLabels(filters.filterRepoId ?? undefined)
  await fetchHealth()
})

watch(() => filters.filterRepoId, async () => {
  await prStore.fetchAll(filters.filterRepoId ?? undefined, undefined)
  await fetchLabels(filters.filterRepoId ?? undefined)
})

/** Client-side state + search + group filtering for instant, reliable results */
const filteredPrs = computed(() => {
  let result = prStore.prs

  // Group filter
  if (filterGroupId.value !== null) {
    const group = groupsStore.groups.find(g => g.id === filterGroupId.value)
    if (group) {
      const repoIds = new Set(group.repo_ids)
      result = result.filter(pr => repoIds.has(pr.repo_id))
    }
  }

  // State filter
  if (filters.filterState !== 'ALL') {
    result = result.filter(pr => {
      const state = filters.filterState
      if (state === 'OPEN') return !pr.merged_at && !pr.closed_at
      if (state === 'MERGED') return !!pr.merged_at
      if (state === 'CLOSED') return !!pr.closed_at && !pr.merged_at
      return true
    })
  }

  // Label filter — PR must carry ALL selected labels
  if (selectedLabels.value.size > 0) {
    result = result.filter(pr =>
      [...selectedLabels.value].every(l => pr.labels.includes(l))
    )
  }

  // CI status filter
  if (filterCiStatus.value) {
    result = result.filter(pr => pr.ci_status === filterCiStatus.value)
  }

  // Search filter
  const q = filters.searchQuery.toLowerCase().trim()
  if (q) {
    result = result.filter(pr => {
      const haystack = [
        pr.title,
        pr.author,
        pr.head_branch,
        `#${pr.number}`,
      ].join(' ').toLowerCase()
      return haystack.includes(q)
    })
  }

  return result
})

const hasFilters = computed(() => {
  return filters.filterRepoId !== null || filters.filterState !== 'ALL' || filters.searchQuery.trim() !== '' || filterGroupId.value !== null || hasLabelSelection.value || filterCiStatus.value !== ''
})

function openDetail(id: number) {
  router.push({ name: 'pr-detail', params: { id } })
}

const selectedIds = ref<Set<number>>(new Set())

function updateSelectedIds(ids: Set<number>) {
  selectedIds.value = ids
}

function clearSelection() {
  selectedIds.value = new Set()
}

async function handleBatchComplete() {
  await prStore.fetchAll(filters.filterRepoId ?? undefined, undefined)
  clearSelection()
}

/** Sort event handlers — propagate to the filters store for persistence */
function onSortByChange(key: SortKey) {
  filters.sortBy = key
}

function onSortAscChange(asc: boolean) {
  filters.sortAsc = asc
}

/** Keyboard-focused row index for j/k navigation */
const focusedIndex = ref(-1)

/**
 * Register this view's navigation context with the root-level keyboard
 * shortcuts composable via provide/inject. The context is set on mount
 * and cleared on unmount so shortcuts only fire on the PR list route.
 */
const prListContext = inject<ShallowRef<PrListContext | null>>('prListContext')

onMounted(() => {
  if (prListContext) {
    prListContext.value = {
      focusedIndex,
      listLength: () => filteredPrs.value.length,
      openDetail,
      getIdAtIndex: (idx: number) => filteredPrs.value[idx]?.id,
      toggleSelection: (id: number) => {
        const next = new Set(selectedIds.value)
        if (next.has(id)) {
          next.delete(id)
        } else {
          next.add(id)
        }
        selectedIds.value = next
      },
    }
  }
})

onUnmounted(() => {
  if (prListContext) {
    prListContext.value = null
  }
})
</script>

<template>
  <div class="pull-requests-view">
    <OfflineBanner
      :is-online="isOnline"
      :time-since-sync="timeSinceSync"
      :syncing="prStore.syncing"
      @retry="prStore.syncAll()"
    />
    <SyncHealthBanner
      v-if="hasSyncIssues"
      :unhealthy-repos="unhealthyRepos"
    />
    <FilterPresetsBar class="presets-row" />
    <div class="filters-bar">
      <div class="filter-group">
        <label class="filter-label">Repository</label>
        <SSelect v-model="repoSelectValue" size="sm">
          <option v-for="opt in repoSelectOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </option>
        </SSelect>
      </div>
      <GroupFilter v-model="filterGroupId" />
      <div class="filter-group">
        <label class="filter-label">State</label>
        <div class="filter-buttons">
          <button
            v-for="state in ['OPEN', 'MERGED', 'CLOSED', 'ALL']"
            :key="state"
            class="filter-btn"
            :class="{ active: filters.filterState === state }"
            @click="filters.filterState = state"
          >
            {{ state.charAt(0) + state.slice(1).toLowerCase() }}
          </button>
        </div>
      </div>
      <div v-if="allLabels.length > 0" class="filter-group">
        <label class="filter-label">Labels</label>
        <div class="label-filter-chips">
          <button
            v-for="lbl in allLabels.slice(0, 12)"
            :key="lbl.name"
            class="label-chip"
            :class="{ active: selectedLabels.has(lbl.name) }"
            :style="lbl.color ? { '--chip-color': `#${lbl.color}` } : {}"
            @click="toggleLabel(lbl.name)"
          >
            {{ lbl.name }}
            <span class="label-count">{{ lbl.count }}</span>
          </button>
          <button
            v-if="hasLabelSelection"
            class="label-chip label-chip-clear"
            @click="clearLabelSelection"
          >Clear</button>
        </div>
      </div>
      <div class="filter-group">
        <label class="filter-label">CI</label>
        <div class="filter-buttons">
          <button
            v-for="ci in ['', 'passing', 'failing', 'pending']"
            :key="ci"
            class="filter-btn"
            :class="{ active: filterCiStatus === ci }"
            @click="filterCiStatus = ci"
          >
            {{ ci === '' ? 'All' : ci.charAt(0).toUpperCase() + ci.slice(1) }}
          </button>
        </div>
      </div>
      <div class="filter-group search-group">
        <label class="filter-label">Search</label>
        <SSearchInput
          :model-value="filters.searchQuery"
          placeholder="Search PRs..."
          clearable
          size="sm"
          class="search-input"
          @update:model-value="filters.searchQuery = $event"
        />
      </div>
    </div>

    <PRTable
      :prs="filteredPrs"
      :loading="prStore.loading"
      :has-filters="hasFilters"
      :selectable="true"
      :selected-ids="selectedIds"
      :sort-by="filters.sortBy"
      :sort-asc="filters.sortAsc"
      :focused-index="focusedIndex"
      @open-detail="openDetail"
      @update:selected-ids="updateSelectedIds"
      @update:sort-by="onSortByChange"
      @update:sort-asc="onSortAscChange"
    />

    <BatchActionBar
      :selected-count="selectedIds.size"
      :selected-ids="[...selectedIds]"
      @clear-selection="clearSelection"
      @batch-complete="handleBatchComplete"
    />
  </div>
</template>

<style scoped>
.pull-requests-view {
  padding-bottom: 80px;
}

.presets-row {
  margin-bottom: var(--space-3);
}

.filters-bar {
  display: flex;
  gap: var(--space-6);
  margin-bottom: var(--space-6);
  align-items: flex-end;
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-4) var(--space-5);
  flex-wrap: wrap;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.filter-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}


.filter-buttons {
  display: flex;
  gap: var(--space-1);
}

.filter-btn {
  background: var(--color-surface-raised);
  color: var(--color-text-secondary);
  font-size: 13px;
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-full);
  border: 1px solid transparent;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.filter-btn:hover {
  background: var(--color-surface-hover);
  color: var(--color-text-primary);
  border-color: var(--color-border-hover);
}

.filter-btn:active {
  transform: scale(0.97);
}

.filter-btn:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.filter-btn.active {
  background: var(--color-accent-muted);
  color: var(--color-accent);
  border-color: rgba(20, 184, 166, 0.3);
  font-weight: 600;
}

.search-group {
  margin-left: auto;
}

.search-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 10px;
  color: var(--color-text-muted);
  pointer-events: none;
}

.search-input {
  max-width: 240px;
}

.search-clear {
  position: absolute;
  right: 4px;
  background: none;
  border: none;
  color: var(--color-text-muted);
  font-size: 16px;
  line-height: 1;
  padding: var(--space-1);
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.search-clear:hover {
  color: var(--color-text-primary);
  background: var(--color-surface-hover);
}

/* Label filter chips */
.label-filter-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.label-chip {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: var(--radius-full);
  border: 1px solid var(--color-border-default);
  background: var(--color-surface-raised);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
  white-space: nowrap;
}

.label-chip:hover {
  border-color: var(--color-border-hover);
  background: var(--color-surface-hover);
}

.label-chip.active {
  background: color-mix(in srgb, var(--chip-color, var(--color-accent)) 15%, transparent);
  color: var(--chip-color, var(--color-accent));
  border-color: color-mix(in srgb, var(--chip-color, var(--color-accent)) 40%, transparent);
}

.label-chip-clear {
  color: var(--color-text-muted);
  font-style: italic;
}

.label-count {
  margin-left: 3px;
  opacity: 0.6;
  font-size: 10px;
}
</style>
