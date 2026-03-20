<script setup lang="ts">
import { ref, computed } from 'vue'
import { openUrl } from '@tauri-apps/plugin-opener'
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

function statusIcon(check: CiCheck): string {
  if (check.conclusion === 'SUCCESS' || check.conclusion === 'success') return '\u2713'
  if (check.conclusion === 'FAILURE' || check.conclusion === 'failure') return '\u2717'
  return '\u23F3'
}

function statusClass(check: CiCheck): string {
  if (check.conclusion === 'SUCCESS' || check.conclusion === 'success') return 'check-pass'
  if (check.conclusion === 'FAILURE' || check.conclusion === 'failure') return 'check-fail'
  return 'check-pending'
}

async function openDetails(url: string | null) {
  if (url) await openUrl(url)
}
</script>

<template>
  <div class="ci-panel">
    <button class="ci-panel-header" @click="expanded = !expanded">
      <h2 class="section-title">CI/CD Status</h2>
      <div class="header-right">
        <span v-if="lastFetchedLabel" class="last-fetched">{{ lastFetchedLabel }}</span>
        <button
          class="refresh-btn"
          title="Refresh CI checks"
          aria-label="Refresh CI checks"
          @click.stop="emit('refresh')"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="23 4 23 10 17 10" />
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
          </svg>
        </button>
        <span class="toggle-icon">{{ expanded ? '\u25B2' : '\u25BC' }}</span>
      </div>
    </button>

    <div v-if="expanded" class="ci-panel-body">
      <div v-if="checks.length === 0" class="ci-empty">
        No checks found for this pull request.
      </div>

      <div v-for="check in checks" :key="check.name" class="ci-check-row">
        <span class="check-icon" :class="statusClass(check)">{{ statusIcon(check) }}</span>
        <span class="check-name">{{ check.name }}</span>
        <button
          v-if="check.detailsUrl"
          class="check-link"
          @click="openDetails(check.detailsUrl)"
        >
          Details &rarr;
        </button>
      </div>
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

.refresh-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  padding: 0;
  background: none;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.refresh-btn:hover {
  color: var(--color-text-primary);
  background: var(--color-surface-hover);
  border-color: var(--color-border-default);
}

.refresh-btn:focus-visible {
  outline: 2px solid var(--color-border-focus);
  outline-offset: 2px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin: 0;
}

.toggle-icon {
  font-size: 11px;
  color: var(--color-text-muted);
}

.ci-panel-body {
  padding: 0 var(--space-5) var(--space-4);
}

.ci-empty {
  font-size: 13px;
  color: var(--color-text-muted);
  padding: var(--space-3) 0;
}

.ci-check-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-2) 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.ci-check-row:last-child {
  border-bottom: none;
}

.check-icon {
  font-size: 13px;
  font-weight: 700;
  min-width: 18px;
  text-align: center;
}

.check-pass { color: var(--color-status-success); }
.check-fail { color: var(--color-status-danger); }
.check-pending {
  color: var(--color-status-warning);
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.check-name {
  flex: 1;
  font-size: 13px;
  color: var(--color-text-primary);
  font-weight: 500;
}

.check-link {
  background: none;
  border: none;
  color: var(--color-accent);
  font-size: 12px;
  cursor: pointer;
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.check-link:hover {
  background: var(--color-accent-muted);
  color: var(--color-accent-hover);
}
</style>
