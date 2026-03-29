<script setup lang="ts">
import { ref, computed } from 'vue'
import { GitCommitHorizontal, Loader2 } from 'lucide-vue-next'
import type { CommitInfo } from '../types'

const props = defineProps<{
  commits: CommitInfo[]
  selectedOid: string | null
  rangeStartOid: string | null
  loading?: boolean
}>()

const emit = defineEmits<{
  'select-commit': [oid: string | null]
  'select-range': [startOid: string, endOid: string]
}>()

const expanded = ref(false)

const selectedLabel = computed(() => {
  if (!props.selectedOid) return 'All changes'
  if (props.rangeStartOid) {
    const startIdx = props.commits.findIndex(c => c.oid === props.rangeStartOid)
    const endIdx = props.commits.findIndex(c => c.oid === props.selectedOid)
    if (startIdx !== -1 && endIdx !== -1) {
      const count = Math.abs(startIdx - endIdx) + 1
      return `${count} commits selected`
    }
  }
  const commit = props.commits.find(c => c.oid === props.selectedOid)
  return commit ? commit.messageHeadline : props.selectedOid.substring(0, 7)
})

function handleCommitClick(oid: string, event: MouseEvent) {
  if (event.shiftKey && props.selectedOid && props.selectedOid !== oid) {
    emit('select-range', props.selectedOid, oid)
  } else if (props.selectedOid === oid) {
    // Deselect — return to all changes
    emit('select-commit', null)
  } else {
    emit('select-commit', oid)
  }
  expanded.value = false
}

function showAllChanges() {
  emit('select-commit', null)
  expanded.value = false
}

function isInRange(oid: string): boolean {
  if (!props.selectedOid || !props.rangeStartOid) return false
  const commits = props.commits
  const startIdx = commits.findIndex(c => c.oid === props.rangeStartOid)
  const endIdx = commits.findIndex(c => c.oid === props.selectedOid)
  const commitIdx = commits.findIndex(c => c.oid === oid)
  if (startIdx === -1 || endIdx === -1 || commitIdx === -1) return false
  const [lo, hi] = startIdx < endIdx ? [startIdx, endIdx] : [endIdx, startIdx]
  return commitIdx >= lo && commitIdx <= hi
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleDateString('en-GB', { day: 'numeric', month: 'short' })
}
</script>

<template>
  <div class="commit-picker">
    <button class="picker-toggle" @click="expanded = !expanded" :class="{ 'has-selection': selectedOid }">
      <GitCommitHorizontal :size="14" />
      <span class="picker-label">{{ selectedLabel }}</span>
      <Loader2 v-if="loading" :size="12" class="picker-spinner" />
      <span class="picker-chevron">{{ expanded ? '\u25B2' : '\u25BC' }}</span>
    </button>

    <div v-if="expanded" class="picker-dropdown">
      <button
        class="picker-option picker-option-all"
        :class="{ 'is-selected': !selectedOid }"
        @click="showAllChanges"
      >
        All changes ({{ commits.length }} commits)
      </button>
      <div class="picker-divider" />
      <div class="picker-hint">Click to select, Shift+click for range</div>
      <div class="picker-list">
        <button
          v-for="commit in commits"
          :key="commit.oid"
          class="picker-option"
          :class="{
            'is-selected': selectedOid === commit.oid && !rangeStartOid,
            'is-in-range': isInRange(commit.oid),
          }"
          @click="handleCommitClick(commit.oid, $event)"
        >
          <span class="commit-oid">{{ commit.oid.substring(0, 7) }}</span>
          <span class="commit-message">{{ commit.messageHeadline }}</span>
          <span class="commit-date">{{ formatDate(commit.committedDate) }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.commit-picker {
  position: relative;
}

.picker-toggle {
  display: flex;
  align-items: center;
  gap: var(--space-1-5);
  padding: var(--space-1) var(--space-3);
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
  white-space: nowrap;
  max-width: 280px;
}

.picker-toggle:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-border-focus);
}

.picker-toggle.has-selection {
  border-color: rgba(20, 184, 166, 0.4);
  color: var(--color-accent);
}

.picker-label {
  overflow: hidden;
  text-overflow: ellipsis;
}

.picker-spinner {
  animation: spin 1s linear infinite;
  flex-shrink: 0;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.picker-chevron {
  font-size: 8px;
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.picker-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  z-index: 20;
  width: 420px;
  max-height: 320px;
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.picker-option {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  width: 100%;
  padding: var(--space-2) var(--space-3);
  background: none;
  border: none;
  color: var(--color-text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: background var(--transition-fast);
  text-align: left;
}

.picker-option:hover {
  background: var(--color-surface-hover);
}

.picker-option.is-selected {
  background: rgba(20, 184, 166, 0.1);
  color: var(--color-accent);
}

.picker-option.is-in-range {
  background: rgba(20, 184, 166, 0.06);
}

.picker-option-all {
  font-weight: 600;
  color: var(--color-text-primary);
  padding: var(--space-2-5) var(--space-3);
}

.picker-divider {
  height: 1px;
  background: var(--color-border-default);
}

.picker-hint {
  font-size: 10px;
  color: var(--color-text-muted);
  padding: var(--space-1-5) var(--space-3);
}

.picker-list {
  overflow-y: auto;
  flex: 1;
}

.commit-oid {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--color-text-muted);
  flex-shrink: 0;
  min-width: 56px;
}

.commit-message {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.commit-date {
  font-size: 10px;
  color: var(--color-text-muted);
  flex-shrink: 0;
}
</style>
