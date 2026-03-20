<script setup lang="ts">
import { ref, watch } from 'vue'
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
        <button
          v-for="opt in statusOptions"
          :key="opt.value"
          class="status-btn"
          :class="{ active: localStatus === opt.value, [`status-${opt.value}`]: true }"
          @click="localStatus = opt.value"
        >
          {{ opt.label }}
        </button>
      </div>
    </div>
    <div class="notes-row">
      <textarea
        v-model="localNotes"
        class="notes-input"
        placeholder="Add review notes…"
        rows="3"
      />
    </div>
    <button class="save-btn" @click="save">Save Review</button>
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
  background: var(--color-surface-hover);
  color: var(--color-text-secondary);
  font-size: 12px;
  padding: var(--space-1) var(--space-3);
  border-radius: var(--radius-full);
  border: 1px solid transparent;
  transition: all var(--transition-fast);
  cursor: pointer;
}

.status-btn:hover {
  background: var(--color-surface-hover);
  color: var(--color-text-primary);
  border-color: var(--color-border-hover);
}

.status-btn:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.status-btn:active {
  transform: scale(0.97);
}

.status-btn.active {
  color: var(--color-text-primary);
}

.status-btn.active.status-pending {
  background: rgba(100, 116, 139, 0.2);
  color: var(--color-text-muted);
  border-color: rgba(100, 116, 139, 0.3);
}
.status-btn.active.status-in_progress {
  background: rgba(59, 130, 246, 0.2);
  color: var(--color-status-info);
  border-color: rgba(59, 130, 246, 0.3);
}
.status-btn.active.status-reviewed {
  background: rgba(234, 179, 8, 0.2);
  color: var(--color-status-warning);
  border-color: rgba(234, 179, 8, 0.3);
}
.status-btn.active.status-approved {
  background: rgba(34, 197, 94, 0.2);
  color: var(--color-status-success);
  border-color: rgba(34, 197, 94, 0.3);
}
.status-btn.active.status-changes_requested {
  background: rgba(220, 38, 38, 0.2);
  color: var(--color-status-danger);
  border-color: rgba(220, 38, 38, 0.3);
}

.notes-input {
  width: 100%;
  resize: vertical;
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-3);
  color: var(--color-text-primary);
  font-size: 13px;
  transition: border-color var(--transition-fast);
}

.notes-input:focus {
  border-color: var(--color-border-focus);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.notes-input::placeholder {
  color: var(--color-text-muted);
}

.save-btn {
  align-self: flex-end;
  background: var(--color-accent);
  color: var(--color-text-inverse);
  font-weight: 600;
  padding: var(--space-2) var(--space-5);
  border-radius: var(--radius-md);
  border: none;
  cursor: pointer;
  transition: background var(--transition-fast), transform var(--transition-fast);
}

.save-btn:hover {
  background: var(--color-accent-hover);
}

.save-btn:active {
  transform: scale(0.97);
}

.save-btn:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}
</style>
