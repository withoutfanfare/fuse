<script setup lang="ts">
import { computed } from 'vue'
import type { PrComment, PrReviewComment } from '../types'

const props = defineProps<{
  comments: PrComment[]
  reviews: PrReviewComment[]
}>()

interface TimelineEntry {
  kind: 'comment' | 'review'
  author: string
  body: string
  timestamp: string
  reviewState?: string
  inlineComments?: PrComment[]
}

const timeline = computed<TimelineEntry[]>(() => {
  const entries: TimelineEntry[] = []

  for (const c of props.comments) {
    entries.push({
      kind: 'comment',
      author: c.author.login,
      body: c.body,
      timestamp: c.createdAt,
    })
  }

  for (const r of props.reviews) {
    entries.push({
      kind: 'review',
      author: r.author.login,
      body: r.body,
      timestamp: r.submittedAt ?? '',
      reviewState: r.state,
      inlineComments: r.comments ?? [],
    })
  }

  entries.sort((a, b) => {
    if (!a.timestamp) return -1
    if (!b.timestamp) return 1
    return new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
  })

  return entries
})

function relativeTime(dateStr: string): string {
  if (!dateStr) return ''
  const now = Date.now()
  const then = new Date(dateStr).getTime()
  const diffMs = now - then
  const diffMins = Math.floor(diffMs / 60000)
  if (diffMins < 1) return 'just now'
  if (diffMins < 60) return `${diffMins}m ago`
  const diffHours = Math.floor(diffMins / 60)
  if (diffHours < 24) return `${diffHours}h ago`
  const diffDays = Math.floor(diffHours / 24)
  if (diffDays < 30) return `${diffDays}d ago`
  const diffMonths = Math.floor(diffDays / 30)
  return `${diffMonths}mo ago`
}

function authorInitial(name: string): string {
  return name.charAt(0).toUpperCase()
}

function stateBadgeClass(state: string): string {
  switch (state) {
    case 'APPROVED': return 'badge-approved'
    case 'CHANGES_REQUESTED': return 'badge-changes-requested'
    case 'COMMENTED': return 'badge-commented'
    case 'DISMISSED': return 'badge-dismissed'
    default: return 'badge-commented'
  }
}

function stateLabel(state: string): string {
  switch (state) {
    case 'APPROVED': return 'Approved'
    case 'CHANGES_REQUESTED': return 'Changes requested'
    case 'COMMENTED': return 'Commented'
    case 'DISMISSED': return 'Dismissed'
    default: return state
  }
}
</script>

<template>
  <div class="comment-thread">
    <div v-if="timeline.length === 0" class="empty-state">
      <p>No comments or reviews yet.</p>
    </div>

    <div v-for="(entry, idx) in timeline" :key="idx" class="comment-card">
      <div class="comment-header">
        <div class="avatar-circle">{{ authorInitial(entry.author) }}</div>
        <span class="comment-author">{{ entry.author }}</span>
        <span v-if="entry.kind === 'review' && entry.reviewState" class="review-badge" :class="stateBadgeClass(entry.reviewState)">
          {{ stateLabel(entry.reviewState) }}
        </span>
        <span class="comment-time">{{ relativeTime(entry.timestamp) }}</span>
      </div>

      <div v-if="entry.body" class="comment-body">{{ entry.body }}</div>

      <div v-if="entry.inlineComments && entry.inlineComments.length > 0" class="inline-comments">
        <div v-for="(ic, icIdx) in entry.inlineComments" :key="icIdx" class="inline-comment">
          <div class="inline-comment-header">
            <div class="avatar-circle avatar-small">{{ authorInitial(ic.author.login) }}</div>
            <span class="comment-author">{{ ic.author.login }}</span>
            <span class="comment-time">{{ relativeTime(ic.createdAt) }}</span>
          </div>
          <div class="comment-body inline-body">{{ ic.body }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.comment-thread {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.empty-state {
  text-align: centre;
  color: var(--color-text-muted);
  font-size: 13px;
  padding: var(--space-6);
}

.comment-card {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-4);
}

.comment-header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-bottom: var(--space-2);
}

.avatar-circle {
  width: 28px;
  height: 28px;
  border-radius: var(--radius-full);
  background: var(--color-accent-muted);
  color: var(--color-accent);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 600;
  flex-shrink: 0;
}

.avatar-small {
  width: 22px;
  height: 22px;
  font-size: 11px;
}

.comment-author {
  font-weight: 600;
  font-size: 13px;
  color: var(--color-text-primary);
}

.comment-time {
  font-size: 12px;
  color: var(--color-text-muted);
  margin-left: auto;
}

.review-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 1px var(--space-2);
  border-radius: var(--radius-full);
}

.badge-approved {
  background: rgba(34, 197, 94, 0.2);
  color: var(--color-status-success);
}

.badge-changes-requested {
  background: rgba(220, 38, 38, 0.2);
  color: var(--color-status-danger);
}

.badge-commented {
  background: rgba(96, 165, 250, 0.2);
  color: var(--color-status-info);
}

.badge-dismissed {
  background: rgba(100, 116, 139, 0.2);
  color: var(--color-text-muted);
}

.comment-body {
  font-size: 13px;
  line-height: 1.6;
  color: var(--color-text-secondary);
  white-space: pre-wrap;
  word-break: break-word;
}

.inline-comments {
  margin-top: var(--space-3);
  padding-top: var(--space-3);
  border-top: 1px solid var(--color-border-default);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.inline-comment {
  padding-left: var(--space-4);
  border-left: 2px solid var(--color-border-default);
}

.inline-comment-header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-bottom: var(--space-1);
}

.inline-body {
  padding-left: calc(22px + var(--space-2));
}
</style>
