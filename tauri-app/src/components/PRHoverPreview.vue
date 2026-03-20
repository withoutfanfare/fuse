<script setup lang="ts">
import { computed } from 'vue'
import type { PullRequest } from '../types'
import { computeRiskScore, computeRiskBreakdown, riskLevel, riskColour } from '../composables/useRiskScore'

const props = defineProps<{
  pr: PullRequest
  position: { x: number; y: number }
}>()

const score = computed(() => computeRiskScore(props.pr))
const level = computed(() => riskLevel(score.value))
const colour = computed(() => riskColour(score.value))
const breakdown = computed(() => computeRiskBreakdown(props.pr))

/** Truncate description to a sensible excerpt length */
const descriptionExcerpt = computed(() => {
  const body = props.pr.body ?? ''
  if (body.length <= 180) return body
  return body.slice(0, 180).trimEnd() + '…'
})

/** Derive a basic CI summary from the PR's review decision field */
const ciSummary = computed(() => {
  if (props.pr.mergeable === 'CONFLICTING') return 'Conflicts detected'
  if (props.pr.review_decision === 'APPROVED') return 'Approved'
  if (props.pr.review_decision === 'CHANGES_REQUESTED') return 'Changes requested'
  if (props.pr.review_decision === 'REVIEW_REQUIRED') return 'Review required'
  return null
})

/**
 * Position the card near the cursor but ensure it stays within the viewport.
 * Card appears to the right of the cursor with a small offset.
 */
const cardStyle = computed(() => {
  const cardWidth = 340
  const cardHeight = 320
  const margin = 16
  const vw = typeof window !== 'undefined' ? window.innerWidth : 1200
  const vh = typeof window !== 'undefined' ? window.innerHeight : 800

  let left = props.position.x + margin
  let top = props.position.y - 40

  /* Flip to the left if it would overflow the right edge */
  if (left + cardWidth > vw - margin) {
    left = props.position.x - cardWidth - margin
  }

  /* Clamp vertically so the card stays on screen */
  if (top + cardHeight > vh - margin) {
    top = vh - cardHeight - margin
  }
  if (top < margin) {
    top = margin
  }

  return {
    left: `${left}px`,
    top: `${top}px`,
  }
})
</script>

<template>
  <Teleport to="body">
    <div class="hover-preview" :style="cardStyle" role="tooltip">
      <!-- Header: PR number + title -->
      <div class="preview-header">
        <span class="preview-number">#{{ pr.number }}</span>
        <span class="preview-title">{{ pr.title }}</span>
      </div>

      <!-- Description excerpt -->
      <p v-if="descriptionExcerpt" class="preview-description">{{ descriptionExcerpt }}</p>

      <!-- Labels -->
      <div v-if="pr.labels.length > 0" class="preview-labels">
        <span v-for="label in pr.labels" :key="label" class="preview-label">{{ label }}</span>
      </div>

      <!-- CI / review status summary -->
      <div v-if="ciSummary" class="preview-ci">
        <span class="ci-dot" :class="[
          pr.review_decision === 'APPROVED' ? 'ci-pass' :
          pr.review_decision === 'CHANGES_REQUESTED' || pr.mergeable === 'CONFLICTING' ? 'ci-fail' :
          'ci-pending'
        ]" />
        {{ ciSummary }}
      </div>

      <!-- Risk breakdown -->
      <div class="preview-risk">
        <div class="risk-header">
          <span class="risk-score" :style="{ color: colour }">Risk {{ score }}/10</span>
          <span class="risk-level" :class="`risk-${level}`">{{ level }}</span>
        </div>
        <div class="risk-factors">
          <div v-for="factor in breakdown" :key="factor.label" class="risk-factor">
            <span class="factor-label">{{ factor.label }}</span>
            <span class="factor-points" :class="factor.points < 0 ? 'factor-neg' : 'factor-pos'">
              {{ factor.points > 0 ? '+' : '' }}{{ factor.points }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.hover-preview {
  position: fixed;
  z-index: 9999;
  width: 340px;
  max-height: 380px;
  overflow-y: auto;
  background: var(--color-surface-panel);
  backdrop-filter: blur(24px) saturate(1.4);
  -webkit-backdrop-filter: blur(24px) saturate(1.4);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card), 0 8px 32px rgba(0, 0, 0, 0.4);
  padding: var(--space-4);
  pointer-events: none;
  animation: preview-fade-in 0.15s ease-out;
}

@keyframes preview-fade-in {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}

/* Respect reduced motion preference */
@media (prefers-reduced-motion: reduce) {
  .hover-preview {
    animation: none;
  }
}

.preview-header {
  display: flex;
  align-items: baseline;
  gap: var(--space-2);
  margin-bottom: var(--space-2);
}

.preview-number {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.preview-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.preview-description {
  font-size: 12px;
  color: var(--color-text-secondary);
  line-height: 1.5;
  margin: 0 0 var(--space-3) 0;
  /* Allow wrapping for multi-line excerpts */
  white-space: pre-wrap;
  word-break: break-word;
}

.preview-labels {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-1);
  margin-bottom: var(--space-3);
}

.preview-label {
  background: rgba(20, 184, 166, 0.15);
  color: var(--color-accent);
  font-size: 11px;
  padding: 1px var(--space-2);
  border-radius: var(--radius-full);
  font-weight: 500;
}

.preview-ci {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-bottom: var(--space-3);
}

.ci-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.ci-pass { background: var(--color-status-success); }
.ci-fail { background: var(--color-status-danger); }
.ci-pending { background: var(--color-status-warning); }

.preview-risk {
  border-top: 1px solid var(--color-border-default);
  padding-top: var(--space-3);
}

.risk-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-2);
}

.risk-score {
  font-size: 13px;
  font-weight: 600;
  font-family: var(--font-mono);
}

.risk-level {
  font-size: 11px;
  font-weight: 600;
  padding: 1px var(--space-2);
  border-radius: var(--radius-full);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.risk-low { background: rgba(34, 197, 94, 0.2); color: var(--color-status-success); }
.risk-medium { background: rgba(234, 179, 8, 0.2); color: var(--color-status-warning); }
.risk-high { background: rgba(249, 115, 22, 0.2); color: var(--color-risk-high); }
.risk-critical { background: rgba(220, 38, 38, 0.2); color: var(--color-status-danger); }

.risk-factors {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.risk-factor {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
}

.factor-label { color: var(--color-text-muted); }
.factor-pos { color: var(--color-status-danger); font-family: var(--font-mono); }
.factor-neg { color: var(--color-status-success); font-family: var(--font-mono); }
</style>
