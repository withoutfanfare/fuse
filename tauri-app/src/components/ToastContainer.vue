<script setup lang="ts">
import { useToastStore } from '../stores/toast'
import type { ToastType } from '../types'

const toastStore = useToastStore()

function borderColour(type: ToastType): string {
  const map: Record<ToastType, string> = {
    success: 'var(--color-status-success)',
    error: 'var(--color-status-danger)',
    warning: 'var(--color-status-warning)',
    info: 'var(--color-status-info)',
  }
  return map[type]
}
</script>

<template>
  <div class="toast-container" aria-live="polite">
    <TransitionGroup name="toast">
      <div
        v-for="toast in toastStore.toasts"
        :key="toast.id"
        class="toast-card"
        :style="{ borderColor: borderColour(toast.type) }"
        role="alert"
      >
        <div class="toast-body">
          <span class="toast-title">{{ toast.title }}</span>
          <span v-if="toast.message" class="toast-message">{{ toast.message }}</span>
        </div>
        <button
          class="toast-close"
          aria-label="Dismiss notification"
          @click="toastStore.removeToast(toast.id)"
        >
          &times;
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-container {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 50;
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: 360px;
  pointer-events: none;
}

.toast-card {
  pointer-events: auto;
  display: flex;
  align-items: flex-start;
  gap: var(--space-3);
  background: var(--color-surface-panel);
  backdrop-filter: blur(24px) saturate(1.4);
  border: 1px solid;
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-3) var(--space-4);
  min-width: 280px;
  max-width: 360px;
}

.toast-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  min-width: 0;
}

.toast-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
  line-height: 1.4;
}

.toast-message {
  font-size: 12px;
  color: var(--color-text-secondary);
  line-height: 1.4;
}

.toast-close {
  flex-shrink: 0;
  background: none;
  border: none;
  color: var(--color-text-muted);
  font-size: 18px;
  line-height: 1;
  padding: 0;
  cursor: pointer;
  transition: color var(--transition-fast);
}

.toast-close:hover {
  color: var(--color-text-primary);
}

/* Transition: slide in from right, fade out */
.toast-enter-active {
  transition: all 0.3s ease-out;
}

.toast-leave-active {
  transition: all 0.25s ease-in;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(80px);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(40px);
}

.toast-move {
  transition: transform 0.25s ease;
}
</style>
