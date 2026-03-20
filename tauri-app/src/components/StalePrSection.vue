<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { usePullRequestsStore } from '../stores/pullRequests'
import { useToastStore } from '../stores/toast'
import { useConfirm } from '@stuntrocket/ui'
import type { PullRequest } from '../types'
import ContentLoader from './ContentLoader.vue'

const props = withDefaults(defineProps<{
  stalePrs: PullRequest[]
  loading?: boolean
}>(), {
  loading: false,
})

const emit = defineEmits<{
  'update:stalePrs': [prs: PullRequest[]]
}>()

const router = useRouter()
const prStore = usePullRequestsStore()
const toastStore = useToastStore()
const { confirm } = useConfirm()

const closingId = ref<number | null>(null)

const count = computed(() => props.stalePrs.length)

function ageInDays(updatedAt: string): number {
  const ms = Date.now() - Date.parse(updatedAt)
  return Math.floor(ms / 86_400_000)
}

async function handleClose(pr: PullRequest) {
  const confirmed = await confirm({
    title: 'Close Stale Pull Request',
    message: `Are you sure you want to close #${pr.number} "${pr.title}"?`,
    confirmLabel: 'Close PR',
  })
  if (!confirmed) return

  closingId.value = pr.id
  const success = await prStore.closePr(pr.id)
  if (success) {
    emit('update:stalePrs', props.stalePrs.filter(p => p.id !== pr.id))
    toastStore.addToast('success', 'PR Closed', `#${pr.number} has been closed on GitHub`)
  } else {
    toastStore.addToast('error', 'Close failed', prStore.error ?? 'Failed to close PR')
  }
  closingId.value = null
}

function openDetail(id: number) {
  router.push({ name: 'pr-detail', params: { id } })
}
</script>

<template>
  <section class="stale-section">
    <div class="stale-header">
      <h2 class="section-title">
        Stale Pull Requests
        <span v-if="!loading && count > 0" class="stale-count">{{ count }}</span>
      </h2>
    </div>

    <ContentLoader v-if="props.loading" variant="list" :count="2" />

    <div v-else-if="props.stalePrs.length === 0" class="stale-empty">
      No stale pull requests. Everything is up to date.
    </div>

    <div v-else class="stale-list">
      <div
        v-for="pr in props.stalePrs"
        :key="pr.id"
        class="stale-card"
      >
        <div class="stale-card-main" @click="openDetail(pr.id)">
          <div class="stale-card-top">
            <span class="pr-number">#{{ pr.number }}</span>
            <span class="stale-age">{{ ageInDays(pr.updated_at) }} days stale</span>
          </div>
          <div class="stale-title">{{ pr.title }}</div>
          <div class="stale-meta">
            <span class="stale-author">{{ pr.author }}</span>
            <span class="stale-separator">&middot;</span>
            <span class="stale-branch">{{ pr.head_branch }}</span>
          </div>
        </div>
        <button
          class="btn-close-pr"
          :disabled="closingId === pr.id"
          @click.stop="handleClose(pr)"
        >
          {{ closingId === pr.id ? 'Closing...' : 'Close' }}
        </button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.stale-section {
  margin-top: var(--space-8);
}

.stale-header {
  margin-bottom: var(--space-4);
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.stale-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 22px;
  height: 22px;
  padding: 0 var(--space-1-5);
  border-radius: var(--radius-full);
  background: rgba(234, 179, 8, 0.2);
  color: var(--color-status-warning);
  font-size: 12px;
  font-weight: 700;
}

.stale-loading {
  text-align: center;
  padding: var(--space-8);
  color: var(--color-text-muted);
  font-size: 14px;
}

.stale-empty {
  text-align: center;
  padding: var(--space-8);
  color: var(--color-text-muted);
  font-size: 14px;
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
}

.stale-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.stale-card {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-4);
  transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
}

.stale-card:hover {
  border-color: var(--color-border-hover);
  box-shadow: var(--shadow-panel);
}

.stale-card-main {
  flex: 1;
  cursor: pointer;
  min-width: 0;
}

.stale-card-top {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-bottom: var(--space-1);
}

.pr-number {
  font-family: var(--font-mono);
  font-size: 13px;
  color: var(--color-text-muted);
}

.stale-age {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-status-warning);
  background: rgba(234, 179, 8, 0.15);
  padding: 1px var(--space-2);
  border-radius: var(--radius-full);
}

.stale-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-primary);
  line-height: 1.4;
  margin-bottom: var(--space-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.stale-meta {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  font-size: 12px;
  color: var(--color-text-muted);
}

.stale-author {
  font-weight: 500;
  color: var(--color-text-secondary);
}

.stale-separator {
  color: var(--color-border-default);
}

.stale-branch {
  font-family: var(--font-mono);
  font-size: 11px;
}

.btn-close-pr {
  flex-shrink: 0;
  background: rgba(220, 38, 38, 0.15);
  color: var(--color-status-danger);
  font-weight: 600;
  font-size: 13px;
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  border: 1px solid rgba(220, 38, 38, 0.3);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-close-pr:hover:not(:disabled) {
  background: rgba(220, 38, 38, 0.25);
  border-color: rgba(220, 38, 38, 0.5);
}

.btn-close-pr:active:not(:disabled) {
  transform: scale(0.97);
}

.btn-close-pr:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-close-pr:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}
</style>
