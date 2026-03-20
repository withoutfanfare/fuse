<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import type { ReviewStatus } from '../types'

const props = defineProps<{
  currentStatus: ReviewStatus | null
  anchorRect: DOMRect
}>()

const emit = defineEmits<{
  'select': [status: ReviewStatus]
  'close': []
}>()

const popoverRef = ref<HTMLElement | null>(null)

/** All available review statuses with display labels */
const statusOptions: { value: ReviewStatus; label: string; colourClass: string }[] = [
  { value: 'pending', label: 'Pending', colourClass: 'opt-pending' },
  { value: 'in_progress', label: 'In Progress', colourClass: 'opt-in_progress' },
  { value: 'reviewed', label: 'Reviewed', colourClass: 'opt-reviewed' },
  { value: 'approved', label: 'Approved', colourClass: 'opt-approved' },
  { value: 'changes_requested', label: 'Changes Requested', colourClass: 'opt-changes_requested' },
]

/** Position the popover relative to the anchor badge */
const popoverStyle = computed(() => {
  const margin = 4
  const rect = props.anchorRect

  /* Centre horizontally below the badge */
  let left = rect.left + rect.width / 2 - 100
  let top = rect.bottom + margin

  /* Clamp within the viewport */
  const vw = typeof window !== 'undefined' ? window.innerWidth : 1200
  const vh = typeof window !== 'undefined' ? window.innerHeight : 800

  if (left + 200 > vw - 8) left = vw - 208
  if (left < 8) left = 8
  if (top + 220 > vh - 8) {
    /* Flip above the badge */
    top = rect.top - 220 - margin
  }

  return {
    left: `${left}px`,
    top: `${top}px`,
  }
})

function selectStatus(status: ReviewStatus) {
  emit('select', status)
  emit('close')
}

/** Close on outside click */
function onDocumentClick(event: MouseEvent) {
  if (popoverRef.value && !popoverRef.value.contains(event.target as Node)) {
    emit('close')
  }
}

/** Close on Escape */
function onKeyDown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    emit('close')
  }
}

onMounted(() => {
  nextTick(() => {
    document.addEventListener('click', onDocumentClick, true)
    document.addEventListener('keydown', onKeyDown)
  })
})

onUnmounted(() => {
  document.removeEventListener('click', onDocumentClick, true)
  document.removeEventListener('keydown', onKeyDown)
})
</script>

<template>
  <Teleport to="body">
    <div
      ref="popoverRef"
      class="quick-status-popover"
      :style="popoverStyle"
      role="listbox"
      aria-label="Change review status"
    >
      <div class="popover-header">Set Status</div>
      <button
        v-for="opt in statusOptions"
        :key="opt.value"
        class="status-option"
        :class="[opt.colourClass, { active: currentStatus === opt.value }]"
        role="option"
        :aria-selected="currentStatus === opt.value"
        @click.stop="selectStatus(opt.value)"
      >
        <span class="status-dot" />
        <span class="status-label">{{ opt.label }}</span>
        <span v-if="currentStatus === opt.value" class="check-mark">&#10003;</span>
      </button>
    </div>
  </Teleport>
</template>

<style scoped>
.quick-status-popover {
  position: fixed;
  z-index: 9998;
  width: 200px;
  background: var(--color-surface-panel);
  backdrop-filter: blur(24px) saturate(1.4);
  -webkit-backdrop-filter: blur(24px) saturate(1.4);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card), 0 8px 32px rgba(0, 0, 0, 0.4);
  padding: var(--space-2);
  animation: popover-enter 0.12s ease-out;
}

@keyframes popover-enter {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}

@media (prefers-reduced-motion: reduce) {
  .quick-status-popover {
    animation: none;
  }
}

.popover-header {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-text-muted);
  padding: var(--space-1) var(--space-2) var(--space-2);
  border-bottom: 1px solid var(--color-border-default);
  margin-bottom: var(--space-1);
}

.status-option {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  width: 100%;
  padding: var(--space-2) var(--space-2);
  background: none;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: 13px;
  color: var(--color-text-secondary);
  transition: background var(--transition-fast);
  text-align: left;
}

.status-option:hover {
  background: var(--color-surface-hover);
}

.status-option.active {
  color: var(--color-text-primary);
  font-weight: 500;
}

.status-option:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

/* Status dot colours matching the table badges */
.opt-pending .status-dot { background: var(--color-text-muted); }
.opt-in_progress .status-dot { background: var(--color-status-info); }
.opt-reviewed .status-dot { background: var(--color-status-warning); }
.opt-approved .status-dot { background: var(--color-status-success); }
.opt-changes_requested .status-dot { background: var(--color-status-danger); }

.status-label {
  flex: 1;
}

.check-mark {
  font-size: 12px;
  color: var(--color-accent);
}
</style>
