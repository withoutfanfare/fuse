<script setup lang="ts">
import { ref } from 'vue'
import { CheckCircle, GitMerge, X } from 'lucide-vue-next'
import { SButton, SBadge } from '@stuntrocket/ui'
import { usePullRequestsStore } from '../stores/pullRequests'
import { useToastStore } from '../stores/toast'
import type { BatchResult } from '../types'

const props = defineProps<{
  selectedCount: number
  selectedIds: number[]
}>()

const emit = defineEmits<{
  'clear-selection': []
  'batch-complete': []
}>()

const prStore = usePullRequestsStore()
const toastStore = useToastStore()
const processing = ref(false)
const results = ref<BatchResult[]>([])
const showResults = ref(false)

async function handleBatchApprove() {
  processing.value = true
  results.value = []
  showResults.value = false

  const batchResults = await prStore.batchApprove(props.selectedIds)
  results.value = batchResults
  showResults.value = true
  processing.value = false

  const succeeded = batchResults.filter(r => r.success).length
  const failed = batchResults.filter(r => !r.success).length

  if (failed === 0) {
    toastStore.addToast('success', 'Batch approve complete', `${succeeded} PR(s) approved successfully`)
  } else {
    toastStore.addToast('warning', 'Batch approve complete', `${succeeded} succeeded, ${failed} failed`)
  }

  emit('batch-complete')
}

async function handleBatchMerge() {
  processing.value = true
  results.value = []
  showResults.value = false

  const batchResults = await prStore.batchMerge(props.selectedIds)
  results.value = batchResults
  showResults.value = true
  processing.value = false

  const succeeded = batchResults.filter(r => r.success).length
  const failed = batchResults.filter(r => !r.success).length

  if (failed === 0) {
    toastStore.addToast('success', 'Batch merge complete', `${succeeded} PR(s) merged successfully`)
  } else {
    toastStore.addToast('warning', 'Batch merge complete', `${succeeded} succeeded, ${failed} failed`)
  }

  emit('batch-complete')
}

function dismissResults() {
  showResults.value = false
  results.value = []
}
</script>

<template>
  <Transition name="slide-up">
    <div v-if="selectedCount > 0" class="batch-action-bar">
      <div class="batch-bar-inner">
        <div class="batch-info">
          <span class="batch-count">{{ selectedCount }} PR{{ selectedCount !== 1 ? 's' : '' }} selected</span>
          <SButton variant="ghost" size="sm" @click="emit('clear-selection')" title="Clear selection">
            <X :size="14" />
            Clear
          </SButton>
        </div>

        <div class="batch-actions">
          <SButton
            variant="primary"
            size="sm"
            :disabled="processing"
            :loading="processing"
            @click="handleBatchApprove"
          >
            <CheckCircle :size="16" />
            Approve All
          </SButton>
          <SButton
            variant="secondary"
            size="sm"
            :disabled="processing"
            :loading="processing"
            @click="handleBatchMerge"
          >
            <GitMerge :size="16" />
            Merge All
          </SButton>
        </div>
      </div>

      <Transition name="results-fade">
        <div v-if="showResults && results.length > 0" class="batch-results">
          <div class="results-header">
            <span class="results-title">Results</span>
            <SButton variant="ghost" size="sm" @click="dismissResults">Dismiss</SButton>
          </div>
          <div class="results-list">
            <div
              v-for="result in results"
              :key="result.pr_id"
              class="result-item"
            >
              <SBadge :variant="result.success ? 'success' : 'error'">
                {{ result.success ? 'OK' : 'FAIL' }}
              </SBadge>
              <span class="result-message">{{ result.message }}</span>
            </div>
          </div>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<style scoped>
.batch-action-bar {
  position: fixed;
  bottom: 0;
  left: var(--sidebar-width);
  right: 0;
  z-index: 100;
  background: rgba(18, 17, 16, 0.85);
  border-top: 1px solid var(--color-border-default);
  box-shadow: var(--shadow-overlay);
  padding: var(--space-4) var(--space-6);
}

.batch-bar-inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  max-width: 1200px;
}

.batch-info {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.batch-count {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.batch-actions {
  display: flex;
  gap: var(--space-3);
}

.batch-results {
  margin-top: var(--space-3);
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-3);
  max-height: 200px;
  overflow-y: auto;
}

.results-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-2);
}

.results-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.results-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.result-item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: 12px;
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-sm);
}

.result-message {
  color: var(--color-text-secondary);
}

/* Slide-up transition */
.slide-up-enter-active {
  transition: transform 250ms cubic-bezier(0.4, 0, 0.2, 1), opacity 250ms ease;
}

.slide-up-leave-active {
  transition: transform 200ms cubic-bezier(0.4, 0, 0.2, 1), opacity 200ms ease;
}

.slide-up-enter-from {
  transform: translateY(100%);
  opacity: 0;
}

.slide-up-leave-to {
  transform: translateY(100%);
  opacity: 0;
}

/* Results fade transition */
.results-fade-enter-active {
  transition: opacity 200ms ease, max-height 250ms ease;
}

.results-fade-leave-active {
  transition: opacity 150ms ease, max-height 200ms ease;
}

.results-fade-enter-from,
.results-fade-leave-to {
  opacity: 0;
}
</style>
