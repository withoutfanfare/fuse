<script setup lang="ts">
import { computed } from 'vue'
import type { CommitInfo } from '../types'
import AuthorAvatar from './AuthorAvatar.vue'

const props = defineProps<{
  commits: CommitInfo[]
  loading: boolean
  error: string | null
}>()

/**
 * Format a commit hash to a short 7-character abbreviation.
 */
function shortHash(oid: string): string {
  return oid.slice(0, 7)
}

/**
 * Format an ISO date string into a human-readable form.
 */
function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-GB', {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

/**
 * Resolve the display name for a commit author.
 * Prefers the GitHub login if available, otherwise falls back to the name.
 */
function authorDisplayName(author: { name: string; login: string | null }): string {
  return author.login ?? author.name
}

const commitCount = computed(() => props.commits.length)
</script>

<template>
  <div class="commit-timeline">
    <div v-if="loading" class="timeline-loading">
      Loading commit history...
    </div>

    <div v-else-if="error" class="timeline-error">
      <span class="error-icon">!</span>
      <span>{{ error }}</span>
    </div>

    <div v-else-if="commits.length === 0" class="timeline-empty">
      No commits found for this pull request.
    </div>

    <template v-else>
      <div class="timeline-header">
        <span class="commit-count">{{ commitCount }} commit{{ commitCount === 1 ? '' : 's' }}</span>
      </div>

      <ol class="timeline-list">
        <li
          v-for="(commit, index) in commits"
          :key="commit.oid"
          class="timeline-entry"
          :class="{ 'timeline-entry--last': index === commits.length - 1 }"
        >
          <div class="timeline-connector">
            <div class="timeline-dot" />
            <div v-if="index < commits.length - 1" class="timeline-line" />
          </div>

          <div class="timeline-content">
            <div class="commit-header">
              <div class="commit-author-row">
                <AuthorAvatar
                  v-if="commit.authors.length > 0 && commit.authors[0].login"
                  :username="commit.authors[0].login ?? ''"
                  :size="22"
                />
                <span
                  v-for="(author, aIdx) in commit.authors"
                  :key="aIdx"
                  class="commit-author"
                >
                  {{ authorDisplayName(author) }}<span v-if="aIdx < commit.authors.length - 1" class="author-separator">,&nbsp;</span>
                </span>
              </div>
              <code class="commit-hash" :title="commit.oid">{{ shortHash(commit.oid) }}</code>
            </div>

            <p class="commit-message">{{ commit.messageHeadline }}</p>
            <p v-if="commit.messageBody" class="commit-body">{{ commit.messageBody }}</p>

            <time class="commit-date" :datetime="commit.committedDate">
              {{ formatDate(commit.committedDate) }}
            </time>
          </div>
        </li>
      </ol>
    </template>
  </div>
</template>

<style scoped>
.commit-timeline {
  width: 100%;
}

.timeline-loading,
.timeline-empty {
  text-align: center;
  color: var(--color-text-muted);
  font-size: 13px;
  padding: var(--space-4);
}

.timeline-error {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  color: var(--color-status-danger);
  font-size: 13px;
  padding: var(--space-3);
  background: rgba(220, 38, 38, 0.08);
  border: 1px solid rgba(220, 38, 38, 0.2);
  border-radius: var(--radius-md);
}

.error-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border-radius: var(--radius-full);
  background: rgba(220, 38, 38, 0.2);
  font-size: 11px;
  font-weight: 700;
  flex-shrink: 0;
}

.timeline-header {
  margin-bottom: var(--space-3);
}

.commit-count {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.timeline-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.timeline-entry {
  display: flex;
  gap: var(--space-3);
  position: relative;
}

.timeline-connector {
  display: flex;
  flex-direction: column;
  align-items: center;
  flex-shrink: 0;
  width: 16px;
  padding-top: 4px;
}

.timeline-dot {
  width: 10px;
  height: 10px;
  border-radius: var(--radius-full);
  background: var(--color-accent);
  border: 2px solid rgba(20, 184, 166, 0.3);
  flex-shrink: 0;
  z-index: 1;
}

.timeline-line {
  width: 2px;
  flex: 1;
  background: linear-gradient(
    to bottom,
    rgba(20, 184, 166, 0.3),
    rgba(20, 184, 166, 0.08)
  );
  margin-top: var(--space-1);
}

.timeline-content {
  flex: 1;
  min-width: 0;
  padding-bottom: var(--space-4);
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.timeline-entry--last .timeline-content {
  border-bottom: none;
  padding-bottom: 0;
}

.commit-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-2);
  margin-bottom: var(--space-1);
}

.commit-author-row {
  display: flex;
  align-items: center;
  gap: var(--space-1-5);
  min-width: 0;
}

.commit-author {
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.author-separator {
  color: var(--color-text-muted);
}

.commit-hash {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--color-accent);
  background: var(--color-accent-muted);
  padding: 1px var(--space-2);
  border-radius: var(--radius-sm);
  flex-shrink: 0;
  cursor: default;
  user-select: all;
}

.commit-message {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
  line-height: 1.4;
  margin: 0 0 var(--space-0-5) 0;
  word-break: break-word;
}

.commit-body {
  font-size: 12px;
  color: var(--color-text-muted);
  line-height: 1.5;
  margin: 0 0 var(--space-1) 0;
  white-space: pre-wrap;
  word-break: break-word;
}

.commit-date {
  font-size: 11px;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
}
</style>
