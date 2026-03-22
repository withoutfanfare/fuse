<script setup lang="ts">
import { useRouter } from 'vue-router'
import { SButton, SCard, SSectionHeader, SEmptyState } from '@stuntrocket/ui'
import type { PriorityQueueItem } from '../types'
import AuthorAvatar from './AuthorAvatar.vue'
import ContentLoader from './ContentLoader.vue'

const props = withDefaults(defineProps<{
  queue: PriorityQueueItem[]
  loading?: boolean
  error?: string | null
}>(), {
  loading: false,
  error: null,
})

const emit = defineEmits<{
  refresh: []
}>()

const router = useRouter()

function openDetail(id: number) {
  router.push({ name: 'pr-detail', params: { id } })
}

/** Format priority score for display */
function formatScore(score: number): string {
  return score.toFixed(1)
}

/** Colour for priority score badge based on value */
function scoreColour(score: number): string {
  if (score >= 8) return 'var(--color-risk-critical)'
  if (score >= 5) return 'var(--color-risk-high)'
  if (score >= 3) return 'var(--color-status-warning)'
  return 'var(--color-text-muted)'
}
</script>

<template>
  <div class="priority-queue">
    <div class="queue-header">
      <SSectionHeader title="Review Queue" />
      <SButton variant="secondary" size="sm" @click="emit('refresh')" :disabled="loading" :loading="loading">
        Refresh
      </SButton>
    </div>

    <p v-if="error" class="queue-error">{{ error }}</p>

    <ContentLoader v-if="loading" variant="list" :count="3" />

    <!-- "Next to review" suggestion -->
    <SCard
      v-else-if="queue.length > 0"
      variant="content"
      hoverable
      class="next-review-card"
      @click="openDetail(queue[0].pr.id)"
    >
      <div class="next-label">Next to review</div>
      <div class="next-body">
        <div class="next-title">{{ queue[0].pr.title }}</div>
        <div class="next-meta">
          <AuthorAvatar :username="queue[0].pr.author" />
          <span class="next-author">{{ queue[0].pr.author }}</span>
          <span class="next-separator">&middot;</span>
          <span class="next-number">#{{ queue[0].pr.number }}</span>
        </div>
        <div class="next-factors">
          <span
            v-for="(factor, i) in queue[0].factors"
            :key="i"
            class="factor-chip"
            :class="{ negative: factor.points < 0 }"
          >
            {{ factor.label }} ({{ factor.points > 0 ? '+' : '' }}{{ factor.points.toFixed(1) }})
          </span>
        </div>
      </div>
      <div class="next-score" :style="{ color: scoreColour(queue[0].priority_score) }">
        {{ formatScore(queue[0].priority_score) }}
      </div>
    </SCard>

    <!-- Remaining queue items -->
    <div v-if="queue.length > 1" class="queue-list">
      <SCard
        v-for="item in queue.slice(1, 10)"
        :key="item.pr.id"
        variant="list"
        hoverable
        class="queue-item"
        @click="openDetail(item.pr.id)"
      >
        <div class="queue-item-score" :style="{ color: scoreColour(item.priority_score) }">
          {{ formatScore(item.priority_score) }}
        </div>
        <div class="queue-item-body">
          <div class="queue-item-title">{{ item.pr.title }}</div>
          <div class="queue-item-meta">
            <span class="queue-item-number">#{{ item.pr.number }}</span>
            <span class="meta-sep">&middot;</span>
            <span>{{ item.pr.author }}</span>
            <span class="meta-sep">&middot;</span>
            <span class="queue-item-files">{{ item.pr.changed_files }} files</span>
          </div>
        </div>
        <div class="queue-item-factors" :title="item.factors.map(f => `${f.label}: ${f.points.toFixed(1)}`).join('\n')">
          <span class="factor-count">{{ item.factors.length }} factors</span>
        </div>
      </SCard>
    </div>

    <SEmptyState
      v-else-if="queue.length === 0"
      title="Queue empty"
      description="No open pull requests in the queue."
    />
  </div>
</template>

<style scoped>
.priority-queue {
  width: 100%;
}

.queue-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-4);
}

.queue-error {
  color: var(--color-status-danger);
  font-size: 13px;
  margin-bottom: var(--space-3);
}

.next-review-card {
  display: flex;
  align-items: flex-start;
  gap: var(--space-4);
  cursor: pointer;
  margin-bottom: var(--space-4);
}

.next-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--color-accent);
  background: var(--color-accent-muted);
  padding: 2px var(--space-2);
  border-radius: var(--radius-full);
  white-space: nowrap;
  align-self: flex-start;
}

.next-body {
  flex: 1;
  min-width: 0;
}

.next-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: var(--space-1);
}

.next-meta {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  font-size: 12px;
  color: var(--color-text-muted);
  margin-bottom: var(--space-2);
}

.next-author {
  font-weight: 500;
  color: var(--color-text-secondary);
}

.next-separator {
  color: var(--color-border-default);
}

.next-number {
  font-family: var(--font-mono);
}

.next-factors {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-1);
}

.factor-chip {
  font-size: 10px;
  padding: 1px var(--space-2);
  border-radius: var(--radius-full);
  background: rgba(20, 184, 166, 0.1);
  color: var(--color-text-secondary);
}

.factor-chip.negative {
  background: rgba(100, 116, 139, 0.15);
  color: var(--color-text-muted);
}

.next-score {
  font-size: 22px;
  font-weight: 700;
  font-family: var(--font-mono);
  white-space: nowrap;
  min-width: 48px;
  text-align: right;
}

.queue-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.queue-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  cursor: pointer;
}

.queue-item-score {
  font-size: 16px;
  font-weight: 700;
  font-family: var(--font-mono);
  min-width: 40px;
  text-align: center;
}

.queue-item-body {
  flex: 1;
  min-width: 0;
}

.queue-item-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.queue-item-meta {
  font-size: 11px;
  color: var(--color-text-muted);
  display: flex;
  align-items: center;
  gap: var(--space-1);
}

.queue-item-number {
  font-family: var(--font-mono);
}

.meta-sep {
  color: var(--color-border-default);
}

.queue-item-files {
  color: var(--color-text-muted);
}

.queue-item-factors {
  font-size: 11px;
  color: var(--color-text-muted);
}

.factor-count {
  font-style: italic;
}
</style>
