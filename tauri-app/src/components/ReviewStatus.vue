<script setup lang="ts">
import { ref, watch } from 'vue'
import { SBadge, SButton, STextarea } from '@stuntrocket/ui'
import type { ReviewStatus } from '../types'
import ReviewPipeline from './ReviewPipeline.vue'

const props = defineProps<{
  status: ReviewStatus | null
  notes: string | null
}>()

const emit = defineEmits<{
  'status-changed': [status: ReviewStatus, notes: string]
}>()

const localNotes = ref(props.notes ?? '')
const localStatus = ref<ReviewStatus>(props.status ?? 'pending')

watch(() => props.status, (v) => { localStatus.value = v ?? 'pending' })
watch(() => props.notes, (v) => { localNotes.value = v ?? '' })

function save() {
  emit('status-changed', localStatus.value, localNotes.value)
}

/** Map review status values to SBadge variants. */
const statusVariantMap: Record<ReviewStatus, 'default' | 'info' | 'warning' | 'success' | 'error'> = {
  pending: 'default',
  in_progress: 'info',
  reviewed: 'warning',
  approved: 'success',
  changes_requested: 'error',
}

const statusOptions: { value: ReviewStatus; label: string }[] = [
  { value: 'pending', label: 'Pending' },
  { value: 'in_progress', label: 'In Progress' },
  { value: 'reviewed', label: 'Reviewed' },
  { value: 'approved', label: 'Approved' },
  { value: 'changes_requested', label: 'Changes Requested' },
]
</script>

<template>
  <div class="review-status">
    <ReviewPipeline :status="localStatus" />
    <div class="status-row">
      <label class="status-label">Review Status</label>
      <div class="status-buttons">
        <SBadge
          v-for="opt in statusOptions"
          :key="opt.value"
          :variant="localStatus === opt.value ? statusVariantMap[opt.value] : 'default'"
          role="button"
          tabindex="0"
          class="status-btn"
          :class="{ active: localStatus === opt.value }"
          @click="localStatus = opt.value"
          @keydown.enter="localStatus = opt.value"
          @keydown.space.prevent="localStatus = opt.value"
        >
          {{ opt.label }}
        </SBadge>
      </div>
    </div>
    <div class="notes-row">
      <STextarea
        v-model="localNotes"
        placeholder="Add review notes..."
        :rows="3"
      />
    </div>
    <SButton variant="primary" class="save-btn" @click="save">Save Review</SButton>
  </div>
</template>

<style scoped>
.review-status {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-5);
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.status-row {
  display: flex;
  flex-direction: column;
}

.notes-row {
  display: flex;
  flex-direction: column;
}

.status-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-bottom: var(--space-2);
  display: block;
}

.status-buttons {
  display: flex;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.status-btn {
  cursor: pointer;
  transition: all var(--transition-fast);
}

.status-btn:hover {
  opacity: 0.85;
}

.status-btn:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.status-btn:active {
  transform: scale(0.97);
}

.save-btn {
  align-self: flex-end;
}
</style>
