<script setup lang="ts">
import { computed } from 'vue'
import { SBadge } from '@stuntrocket/ui'

const props = defineProps<{
  status: string
}>()

const steps = ['pending', 'in_progress', 'reviewed', 'approved'] as const
const stepLabels: Record<string, string> = {
  pending: 'Pending',
  in_progress: 'In Progress',
  reviewed: 'Reviewed',
  approved: 'Approved',
}

const isChangesRequested = computed(() => props.status === 'changes_requested')

/**
 * Map current status to a zero-based index in the pipeline.
 * "changes_requested" maps to the same position as "reviewed" (step 2).
 */
const currentIndex = computed(() => {
  if (isChangesRequested.value) return 2
  const idx = steps.indexOf(props.status as typeof steps[number])
  return idx >= 0 ? idx : 0
})

function stepState(idx: number): 'complete' | 'current' | 'future' {
  if (idx < currentIndex.value) return 'complete'
  if (idx === currentIndex.value) return 'current'
  return 'future'
}

/** Map step state to an SBadge variant for the label */
function stepBadgeVariant(idx: number): 'success' | 'accent' | 'warning' | 'default' {
  const state = stepState(idx)
  if (state === 'complete') return 'success'
  if (state === 'current' && isChangesRequested.value && idx === currentIndex.value) return 'warning'
  if (state === 'current') return 'accent'
  return 'default'
}
</script>

<template>
  <div class="review-pipeline">
    <template v-for="(step, idx) in steps" :key="step">
      <!-- Connecting line before every step except the first -->
      <div
        v-if="idx > 0"
        class="pipeline-line"
        :class="{ complete: idx <= currentIndex }"
      />
      <!-- Step circle -->
      <div class="pipeline-step" :class="[stepState(idx)]">
        <div
          class="step-circle"
          :class="{ 'changes-requested': isChangesRequested && idx === currentIndex }"
        />
        <SBadge :variant="stepBadgeVariant(idx)" class="step-badge">
          {{ isChangesRequested && idx === currentIndex ? 'Changes Req.' : stepLabels[step] }}
        </SBadge>
      </div>
    </template>
  </div>
</template>

<style scoped>
.review-pipeline {
  display: flex;
  align-items: flex-start;
  gap: 0;
  width: 100%;
  padding: var(--space-3) 0;
}

/* Step container */
.pipeline-step {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-1-5);
  flex: 0 0 auto;
}

/* Circle */
.step-circle {
  width: 16px;
  height: 16px;
  border-radius: var(--radius-full);
  border: 2px solid var(--color-pipeline-track);
  background: transparent;
  transition: background 0.3s ease, border-color 0.3s ease, box-shadow 0.3s ease;
}

/* Completed step */
.pipeline-step.complete .step-circle {
  background: var(--color-pipeline-complete);
  border-color: var(--color-pipeline-complete);
}

/* Current step — pulse glow */
.pipeline-step.current .step-circle {
  background: var(--color-pipeline-complete);
  border-color: var(--color-pipeline-complete);
  animation: pipeline-pulse 2s ease-in-out infinite;
}

/* Changes requested — red indicator */
.step-circle.changes-requested {
  background: var(--color-status-danger) !important;
  border-color: var(--color-status-danger) !important;
  animation: pipeline-pulse-red 2s ease-in-out infinite !important;
}

/* Future step */
.pipeline-step.future .step-circle {
  background: transparent;
  border-color: var(--color-pipeline-track);
}

/* Connecting line */
.pipeline-line {
  flex: 1;
  height: 2px;
  margin-top: 7px; /* vertically centre with 16px circle */
  background: var(--color-pipeline-track);
  transition: background 0.3s ease;
}

.pipeline-line.complete {
  background: var(--color-pipeline-complete);
}

/* Badge label styling */
.step-badge {
  font-size: 10px;
  white-space: nowrap;
  text-align: center;
}

/* Animations */
@keyframes pipeline-pulse {
  0%, 100% { box-shadow: 0 0 0 0 rgba(249, 115, 22, 0.4); }
  50% { box-shadow: 0 0 0 6px rgba(249, 115, 22, 0); }
}

@keyframes pipeline-pulse-red {
  0%, 100% { box-shadow: 0 0 0 0 rgba(248, 113, 113, 0.4); }
  50% { box-shadow: 0 0 0 6px rgba(248, 113, 113, 0); }
}
</style>
