<script setup lang="ts">
import { ref, computed } from 'vue'
import { useFocusTrap } from '@stuntrocket/ui'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

const panelRef = ref<HTMLElement | null>(null)
const isVisible = computed(() => props.visible)

/* Focus trap keeps Tab / Shift+Tab within the overlay */
useFocusTrap(panelRef, isVisible)

const navigationShortcuts = [
  { key: '1', description: 'Go to Dashboard' },
  { key: '2', description: 'Go to Pull Requests' },
  { key: '3', description: 'Go to Repositories' },
  { key: '4', description: 'Go to Settings' },
  { key: '5', description: 'Go to Aggregate view' },
]

const actionShortcuts = [
  { key: '/', description: 'Focus search' },
  { key: 'r', description: 'Sync all repositories' },
  { key: '\u2318K', description: 'Open command palette' },
  { key: '?', description: 'Toggle this overlay' },
  { key: 'Esc', description: 'Close overlay / blur input' },
]

const reviewShortcuts = [
  { key: '] / n', description: 'Next file in diff' },
  { key: '[ / p', description: 'Previous file in diff' },
  { key: 'c', description: 'Toggle checklist item' },
  { key: 'a', description: 'Jump to next annotation' },
  { key: '\u21E7\u23CE', description: 'Complete review & advance' },
]

function onBackdropClick(e: MouseEvent) {
  if ((e.target as HTMLElement).classList.contains('shortcut-overlay')) {
    emit('close')
  }
}
</script>

<template>
  <Transition name="overlay-fade">
    <div v-if="visible" class="shortcut-overlay" @click="onBackdropClick">
      <div
        ref="panelRef"
        class="shortcut-panel"
        role="dialog"
        aria-modal="true"
        aria-labelledby="shortcut-overlay-title"
      >
        <div class="shortcut-header">
          <h2 id="shortcut-overlay-title" class="shortcut-title">Keyboard Shortcuts</h2>
          <button class="shortcut-close" aria-label="Close shortcuts overlay" @click="emit('close')">&times;</button>
        </div>

        <div class="shortcut-columns">
          <div class="shortcut-group">
            <h3 class="group-title">Navigation</h3>
            <ul class="shortcut-list">
              <li v-for="s in navigationShortcuts" :key="s.key" class="shortcut-item">
                <kbd class="key-cap">{{ s.key }}</kbd>
                <span class="key-desc">{{ s.description }}</span>
              </li>
            </ul>
          </div>

          <div class="shortcut-group">
            <h3 class="group-title">Actions</h3>
            <ul class="shortcut-list">
              <li v-for="s in actionShortcuts" :key="s.key" class="shortcut-item">
                <kbd class="key-cap">{{ s.key }}</kbd>
                <span class="key-desc">{{ s.description }}</span>
              </li>
            </ul>
          </div>

          <div class="shortcut-group">
            <h3 class="group-title">Review Workflow</h3>
            <ul class="shortcut-list">
              <li v-for="s in reviewShortcuts" :key="s.key" class="shortcut-item">
                <kbd class="key-cap">{{ s.key }}</kbd>
                <span class="key-desc">{{ s.description }}</span>
              </li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.shortcut-overlay {
  position: fixed;
  inset: 0;
  z-index: 90;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
}

.shortcut-panel {
  background: var(--color-surface-panel);
  backdrop-filter: blur(24px) saturate(1.4);
  -webkit-backdrop-filter: blur(24px) saturate(1.4);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-overlay);
  padding: var(--space-6);
  min-width: 540px;
  max-width: 640px;
}

.shortcut-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-5);
}

.shortcut-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.shortcut-close {
  background: none;
  color: var(--color-text-muted);
  font-size: 20px;
  line-height: 1;
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-sm);
}

.shortcut-close:hover {
  color: var(--color-text-primary);
  background: var(--color-surface-hover);
}

.shortcut-close:focus-visible {
  outline: 2px solid var(--color-border-focus);
  outline-offset: 2px;
}

.shortcut-columns {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-6);
}

.group-title {
  font-size: 11px;
  font-weight: 700;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  margin-bottom: var(--space-3);
}

.shortcut-list {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.shortcut-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.key-cap {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 28px;
  height: 26px;
  padding: 0 var(--space-2);
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-sm);
  font-family: var(--font-mono);
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-primary);
  box-shadow: 0 1px 0 var(--color-border-default);
}

.key-desc {
  font-size: 13px;
  color: var(--color-text-secondary);
}

/* Transition */
.overlay-fade-enter-active,
.overlay-fade-leave-active {
  transition: opacity var(--transition-fast);
}

.overlay-fade-enter-active .shortcut-panel,
.overlay-fade-leave-active .shortcut-panel {
  transition: transform var(--transition-fast), opacity var(--transition-fast);
}

.overlay-fade-enter-from,
.overlay-fade-leave-to {
  opacity: 0;
}

.overlay-fade-enter-from .shortcut-panel,
.overlay-fade-leave-to .shortcut-panel {
  transform: scale(0.95);
  opacity: 0;
}
</style>
