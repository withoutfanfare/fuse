<script setup lang="ts">
import { WifiOff, RefreshCw } from 'lucide-vue-next'

defineProps<{
  isOnline: boolean
  timeSinceSync: string | null
  syncing: boolean
}>()

const emit = defineEmits<{
  retry: []
}>()
</script>

<template>
  <Transition name="offline-banner">
    <div v-if="!isOnline" class="offline-banner">
      <WifiOff :size="16" class="offline-icon" />
      <span class="offline-text">
        You're offline — showing cached data
        <span v-if="timeSinceSync" class="offline-sync-age">
          (last synced {{ timeSinceSync }})
        </span>
      </span>
      <button
        class="offline-retry"
        :disabled="syncing"
        @click="emit('retry')"
      >
        <RefreshCw :size="12" :class="{ spinning: syncing }" />
        {{ syncing ? 'Retrying…' : 'Retry' }}
      </button>
    </div>
  </Transition>
</template>

<style scoped>
.offline-banner {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-4);
  background: rgba(234, 179, 8, 0.08);
  border: 1px solid rgba(234, 179, 8, 0.2);
  border-radius: var(--radius-md);
  margin-bottom: var(--space-4);
}

.offline-icon {
  color: var(--color-status-warning);
  flex-shrink: 0;
}

.offline-text {
  flex: 1;
  font-size: 12px;
  color: var(--color-text-secondary);
}

.offline-sync-age {
  color: var(--color-text-muted);
}

.offline-retry {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: var(--space-1) var(--space-2);
  background: rgba(234, 179, 8, 0.15);
  border: 1px solid rgba(234, 179, 8, 0.3);
  border-radius: var(--radius-md);
  font-size: 11px;
  font-weight: 600;
  color: var(--color-status-warning);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.offline-retry:hover:not(:disabled) {
  background: rgba(234, 179, 8, 0.25);
}

.offline-retry:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.offline-banner-enter-active {
  transition: all 0.3s ease;
}

.offline-banner-leave-active {
  transition: all 0.2s ease;
}

.offline-banner-enter-from,
.offline-banner-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
