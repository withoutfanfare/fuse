<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import { RefreshCw, Bell } from 'lucide-vue-next'
import { useNotificationCentreStore } from '../../stores/notificationCentre'

const props = defineProps<{
  syncing: boolean
  lastSynced: string | null
  autoSyncActive: boolean
}>()

const emit = defineEmits<{
  'sync-requested': []
}>()

const route = useRoute()
const notifStore = useNotificationCentreStore()

const now = ref(Date.now())
let intervalId: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  intervalId = setInterval(() => {
    now.value = Date.now()
  }, 30_000)
})

onUnmounted(() => {
  if (intervalId !== null) {
    clearInterval(intervalId)
  }
})

const pageTitle = computed(() => {
  const titles: Record<string, string> = {
    '/dashboard': 'Dashboard',
    '/aggregate': 'Aggregate Dashboard',
    '/prs': 'Pull Requests',
    '/repositories': 'Repositories',
    '/settings': 'Settings',
  }
  for (const [path, title] of Object.entries(titles)) {
    if (route.path.startsWith(path)) return title
  }
  return 'Fuse'
})

const lastSyncedFormatted = computed(() => {
  if (!props.lastSynced) return null
  const date = new Date(props.lastSynced)
  const diffMs = now.value - date.getTime()
  const diffMin = Math.floor(diffMs / 60000)
  if (diffMin < 1) return 'just now'
  if (diffMin < 60) return `${diffMin}m ago`
  const diffHr = Math.floor(diffMin / 60)
  return `${diffHr}h ago`
})
</script>

<template>
  <header class="app-header">
    <div class="header-left">
      <h1 class="header-title">{{ pageTitle }}</h1>
    </div>
    <div class="header-actions">
      <span v-if="autoSyncActive" class="auto-sync-indicator" title="Auto-sync active">
        <span class="auto-sync-dot"></span>
        Auto-sync
      </span>
      <span v-if="lastSyncedFormatted" class="sync-status">
        Synced {{ lastSyncedFormatted }}
      </span>
      <button
        class="btn-bell"
        title="Notifications"
        @click="notifStore.toggleDrawer()"
      >
        <Bell :size="16" />
        <span v-if="notifStore.unreadCount > 0" class="bell-badge">
          {{ notifStore.unreadCount > 9 ? '9+' : notifStore.unreadCount }}
        </span>
      </button>
      <button
        class="btn-sync"
        :disabled="syncing"
        @click="emit('sync-requested')"
      >
        <RefreshCw :size="16" class="sync-icon" :class="{ spinning: syncing }" />
        {{ syncing ? 'Syncing…' : 'Sync' }}
      </button>
    </div>
  </header>
</template>

<style scoped>
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-3) var(--space-6) var(--space-4);
  background: transparent;
  flex-shrink: 0;
}

.header-title {
  font-size: 16px;
  font-weight: 600;
  letter-spacing: -0.02em;
  line-height: 1.2;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: var(--space-4);
}

.auto-sync-indicator {
  display: flex;
  align-items: center;
  gap: var(--space-1-5);
  font-size: 12px;
  color: var(--color-status-success);
  font-weight: 500;
}

.auto-sync-dot {
  width: 8px;
  height: 8px;
  border-radius: var(--radius-full);
  background: var(--color-status-success);
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.5; transform: scale(0.85); }
}

.sync-status {
  font-size: 12px;
  color: var(--color-text-muted);
}

/* Notification bell button */
.btn-bell {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: var(--color-surface-hover);
  border: none;
  border-radius: var(--radius-full);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-bell:hover {
  background: var(--color-border-default);
  color: var(--color-text-primary);
}

.bell-badge {
  position: absolute;
  top: -2px;
  right: -2px;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  background: var(--color-status-danger);
  color: white;
  font-size: 9px;
  font-weight: 700;
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
}

.btn-sync {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  background: var(--color-surface-hover);
  color: var(--color-text-primary);
  border: none;
  padding: var(--space-2) var(--space-4);
  font-size: 13px;
  font-weight: 500;
  border-radius: var(--radius-full);
}

.btn-sync:hover:not(:disabled) {
  background: var(--color-border-default);
}

.sync-icon {
  display: inline-block;
  font-size: 15px;
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
