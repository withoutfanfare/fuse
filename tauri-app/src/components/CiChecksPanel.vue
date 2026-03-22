<script setup lang="ts">
import { ref, computed } from 'vue'
import { openUrl } from '@tauri-apps/plugin-opener'
import { SButton, SBadge, SListRow, SSectionHeader, SEmptyState } from '@stuntrocket/ui'
import type { CiCheck } from '../types'

const props = defineProps<{
  checks: CiCheck[]
  /** Timestamp (ms since epoch) of the last successful fetch, or null. */
  lastFetchedAt?: number | null
}>()

const emit = defineEmits<{
  refresh: []
}>()

const expanded = ref(true)

/** Human-readable "last fetched X s ago" text. */
const lastFetchedLabel = computed(() => {
  if (!props.lastFetchedAt) return null
  const seconds = Math.round((Date.now() - props.lastFetchedAt) / 1000)
  if (seconds < 5) return 'just now'
  if (seconds < 60) return `${seconds}s ago`
  const mins = Math.floor(seconds / 60)
  return `${mins}m ago`
})

function statusBadgeVariant(check: CiCheck): 'success' | 'error' | 'warning' {
  if (check.conclusion === 'SUCCESS' || check.conclusion === 'success') return 'success'
  if (check.conclusion === 'FAILURE' || check.conclusion === 'failure') return 'error'
  return 'warning'
}

function statusIcon(check: CiCheck): string {
  if (check.conclusion === 'SUCCESS' || check.conclusion === 'success') return '\u2713'
  if (check.conclusion === 'FAILURE' || check.conclusion === 'failure') return '\u2717'
  return '\u23F3'
}

async function openDetails(url: string | null) {
  if (url) await openUrl(url)
}
</script>

<template>
  <div class="ci-panel">
    <button class="ci-panel-header" @click="expanded = !expanded">
      <SSectionHeader title="CI/CD Status" />
      <div class="header-right">
        <span v-if="lastFetchedLabel" class="last-fetched">{{ lastFetchedLabel }}</span>
        <SButton
          variant="icon"
          size="sm"
          title="Refresh CI checks"
          aria-label="Refresh CI checks"
          @click.stop="emit('refresh')"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="23 4 23 10 17 10" />
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
          </svg>
        </SButton>
        <span class="toggle-icon">{{ expanded ? '\u25B2' : '\u25BC' }}</span>
      </div>
    </button>

    <div v-if="expanded" class="ci-panel-body">
      <SEmptyState
        v-if="checks.length === 0"
        title="No checks found"
        description="No checks found for this pull request."
      />

      <SListRow v-for="check in checks" :key="check.name">
        <template #default>
          <div class="check-row-content">
            <SBadge :variant="statusBadgeVariant(check)">{{ statusIcon(check) }}</SBadge>
            <span class="check-name">{{ check.name }}</span>
          </div>
        </template>
        <template #actions>
          <SButton
            v-if="check.detailsUrl"
            variant="ghost"
            size="sm"
            @click="openDetails(check.detailsUrl)"
          >
            Details &rarr;
          </SButton>
        </template>
      </SListRow>
    </div>
  </div>
</template>

<style scoped>
.ci-panel {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  overflow: hidden;
}

.ci-panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
  background: none;
  border: none;
  padding: var(--space-4) var(--space-5);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.ci-panel-header:hover {
  background: var(--color-surface-hover);
}

.header-right {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.last-fetched {
  font-size: 11px;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
}

.toggle-icon {
  font-size: 11px;
  color: var(--color-text-muted);
}

.ci-panel-body {
  padding: 0 var(--space-5) var(--space-4);
}

.check-row-content {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.check-name {
  flex: 1;
  font-size: 13px;
  color: var(--color-text-primary);
  font-weight: 500;
}
</style>
