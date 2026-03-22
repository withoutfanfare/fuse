<script setup lang="ts">
import { computed } from 'vue'
import { SCard } from '@stuntrocket/ui'
import type { PullRequest } from '../types'
import { computeRiskScore, riskColour } from '../composables/useRiskScore'
import RiskGauge from './RiskGauge.vue'
import AuthorAvatar from './AuthorAvatar.vue'

const props = defineProps<{
  pr: PullRequest
}>()

const emit = defineEmits<{
  'open-detail': [id: number]
}>()

const riskScore = computed(() => computeRiskScore(props.pr))

/** CSS custom property bound to the card element for risk-level glow */
const cardRiskColor = computed(() => riskColour(riskScore.value))

const ageLabel = computed(() => {
  const hours = Math.floor((Date.now() - Date.parse(props.pr.created_at)) / 3_600_000)
  if (hours < 1) return '<1h'
  if (hours < 24) return `${hours}h`
  const days = Math.floor(hours / 24)
  return `${days}d`
})

const ageColourClass = computed(() => {
  const hours = (Date.now() - Date.parse(props.pr.created_at)) / 3_600_000
  if (hours < 24) return 'age-fresh'
  if (hours < 72) return 'age-normal'
  if (hours < 168) return 'age-aging'
  if (hours < 336) return 'age-old'
  return 'age-stale'
})

const repoDisplay = computed(() => {
  // We don't have repo info on the PR directly, so show branch info
  return props.pr.head_branch
})
</script>

<template>
  <SCard
    variant="glass"
    hoverable
    :style="{ '--card-risk-color': cardRiskColor }"
    class="pr-card"
    @click="emit('open-detail', pr.id)"
  >
    <div class="pr-card-header">
      <RiskGauge :score="riskScore" :size="28" />
      <span class="pr-number">#{{ pr.number }}</span>
      <span v-if="pr.is_draft" class="draft-badge">Draft</span>
      <span
        v-if="pr.mergeable === 'CONFLICTING'"
        class="conflict-indicator"
        title="This PR has merge conflicts"
      >Conflicts</span>
    </div>
    <div class="pr-title">{{ pr.title }}</div>
    <div class="pr-meta">
      <AuthorAvatar :username="pr.author" />
      <span class="pr-author">{{ pr.author }}</span>
      <span class="pr-separator">&middot;</span>
      <span class="pr-branch">{{ repoDisplay }}</span>
      <span class="pr-separator">&middot;</span>
      <span class="pr-age" :class="ageColourClass">{{ ageLabel }}</span>
      <span class="pr-separator">&middot;</span>
      <span class="pr-changes">
        <span class="additions">+{{ pr.additions }}</span>
        <span class="deletions">-{{ pr.deletions }}</span>
      </span>
    </div>
    <div v-if="pr.review_status" class="pr-review-badge" :class="[`review-${pr.review_status}`]">
      {{ pr.review_status.replace(/_/g, ' ') }}
    </div>
  </SCard>
</template>

<style scoped>
:deep(.py-12) {
  padding-top: var(--space-4) !important;
  padding-bottom: var(--space-4) !important;
}

.pr-card {
  cursor: pointer;
}

.pr-card-header {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  margin-bottom: var(--space-1);
}

.pr-number {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--color-text-muted);
}

.draft-badge {
  font-size: 11px;
  color: var(--color-text-muted);
  background: rgba(100, 116, 139, 0.2);
  padding: 1px var(--space-2);
  border-radius: var(--radius-full);
}

.pr-title {
  font-size: 13px;
  font-weight: var(--text-subheading-weight);
  letter-spacing: var(--text-body-tracking);
  line-height: 1.35;
  margin-bottom: var(--space-1);
  color: var(--color-text-primary);
}

.pr-meta {
  font-size: 11px;
  font-weight: var(--text-caption-weight);
  letter-spacing: var(--text-caption-tracking);
  color: var(--color-text-muted);
  display: flex;
  align-items: center;
  gap: var(--space-1);
  flex-wrap: wrap;
}

.pr-separator {
  color: var(--color-border-default);
}

.pr-author {
  font-weight: 500;
  color: var(--color-text-secondary);
}

.pr-branch {
  font-family: var(--font-mono);
  font-size: 11px;
}

.additions {
  color: var(--color-status-success);
}

.deletions {
  color: var(--color-status-danger);
  margin-left: 2px;
}

/* PR age colour coding */
.age-fresh { color: var(--color-status-success); }
.age-normal { color: var(--color-text-secondary); }
.age-aging { color: var(--color-status-warning); }
.age-old { color: var(--color-risk-high); }
.age-stale { color: var(--color-status-danger); }

.pr-review-badge {
  display: inline-block;
  margin-top: var(--space-1);
  font-size: 10px;
  font-weight: 600;
  padding: 2px var(--space-2);
  border-radius: var(--radius-full);
  text-transform: capitalize;
}

.review-pending { background: rgba(100, 116, 139, 0.2); color: var(--color-text-muted); }
.review-in_progress { background: rgba(59, 130, 246, 0.2); color: var(--color-status-info); }
.review-reviewed { background: rgba(234, 179, 8, 0.2); color: var(--color-status-warning); }
.review-approved { background: rgba(34, 197, 94, 0.2); color: var(--color-status-success); }
.review-changes_requested { background: rgba(220, 38, 38, 0.2); color: var(--color-status-danger); }

.conflict-indicator {
  font-size: 11px;
  font-weight: 600;
  color: var(--color-status-danger);
  background: rgba(220, 38, 38, 0.2);
  padding: 1px var(--space-2);
  border-radius: var(--radius-full);
}
</style>
