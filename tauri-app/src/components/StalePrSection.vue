<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { SButton, SCard, SBadge, SEmptyState, SSectionHeader } from '@stuntrocket/ui'
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
  <SCard variant="content" class="stale-section">
    <div class="stale-header">
      <SSectionHeader title="Stale Pull Requests" />
      <SBadge v-if="!loading && count > 0" variant="warning">{{ count }}</SBadge>
    </div>

    <ContentLoader v-if="props.loading" variant="list" :count="2" />

    <SEmptyState
      v-else-if="props.stalePrs.length === 0"
      title="All clear"
      description="No stale pull requests. Everything is up to date."
    />

    <div v-else class="stale-list">
      <SCard
        v-for="pr in props.stalePrs"
        :key="pr.id"
        variant="list"
        hoverable
        class="stale-card"
      >
        <div class="stale-card-main" @click="openDetail(pr.id)">
          <div class="stale-card-top">
            <span class="pr-number">#{{ pr.number }}</span>
            <SBadge variant="warning">{{ ageInDays(pr.updated_at) }} days stale</SBadge>
          </div>
          <div class="stale-title">{{ pr.title }}</div>
          <div class="stale-meta">
            <span class="stale-author">{{ pr.author }}</span>
            <span class="stale-separator">&middot;</span>
            <span class="stale-branch">{{ pr.head_branch }}</span>
          </div>
        </div>
        <SButton
          variant="danger"
          size="sm"
          :disabled="closingId === pr.id"
          :loading="closingId === pr.id"
          @click.stop="handleClose(pr)"
        >
          Close
        </SButton>
      </SCard>
    </div>
  </SCard>
</template>

<style scoped>
:deep(.py-12) {
  padding-top: var(--space-4) !important;
  padding-bottom: var(--space-4) !important;
}

.stale-header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-bottom: var(--space-2);
}

.stale-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.stale-card {
  display: flex;
  align-items: center;
  gap: var(--space-2);
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

.stale-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
  line-height: 1.35;
  margin-bottom: var(--space-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.stale-meta {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  font-size: 11px;
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
</style>
