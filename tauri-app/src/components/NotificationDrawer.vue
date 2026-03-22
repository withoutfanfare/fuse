<script setup lang="ts">
import { computed } from 'vue'
import { CheckCheck, Trash2, Info, AlertTriangle, AlertCircle, CheckCircle, X } from 'lucide-vue-next'
import { SDrawer, SIconButton } from '@stuntrocket/ui'
import { useNotificationCentreStore } from '../stores/notificationCentre'
import type { ToastType } from '../types'

const store = useNotificationCentreStore()

/**
 * Icon component and colour for each notification type.
 */
const typeConfig: Record<ToastType, { icon: typeof Info; colour: string }> = {
  info: { icon: Info, colour: 'var(--color-status-info)' },
  success: { icon: CheckCircle, colour: 'var(--color-status-success)' },
  warning: { icon: AlertTriangle, colour: 'var(--color-status-warning)' },
  error: { icon: AlertCircle, colour: 'var(--color-status-danger)' },
}

function formatTime(ts: number): string {
  const diff = Date.now() - ts
  const secs = Math.floor(diff / 1000)
  if (secs < 60) return 'just now'
  const mins = Math.floor(secs / 60)
  if (mins < 60) return `${mins}m ago`
  const hrs = Math.floor(mins / 60)
  if (hrs < 24) return `${hrs}h ago`
  return new Date(ts).toLocaleDateString('en-GB', {
    day: 'numeric',
    month: 'short',
    hour: '2-digit',
    minute: '2-digit',
  })
}

const isEmpty = computed(() => store.entries.length === 0)
</script>

<template>
  <SDrawer
    :open="store.drawerOpen"
    title="Notifications"
    size="sm"
    side="right"
    closable
    overlay
    @close="store.closeDrawer()"
  >
    <template #header>
      <div class="drawer-header-content">
        <span class="drawer-title">Notifications</span>
        <div class="drawer-actions">
          <SIconButton
            v-if="!isEmpty"
            variant="ghost"
            size="sm"
            tooltip="Mark all as read"
            @click="store.markAllRead()"
          >
            <CheckCheck :size="16" />
          </SIconButton>
          <SIconButton
            v-if="!isEmpty"
            variant="ghost"
            size="sm"
            tooltip="Clear all"
            @click="store.clearAll()"
          >
            <Trash2 :size="16" />
          </SIconButton>
        </div>
      </div>
    </template>

    <div class="drawer-body">
      <div v-if="isEmpty" class="drawer-empty">
        <Info :size="28" class="drawer-empty-icon" />
        <p>No notifications yet</p>
      </div>
      <TransitionGroup v-else name="notif" tag="div" class="drawer-list">
        <div
          v-for="entry in store.entries"
          :key="entry.id"
          class="notif-item"
          :class="{ 'notif-item--unread': !entry.read }"
        >
          <div
            class="notif-icon"
            :style="{ color: typeConfig[entry.type].colour }"
          >
            <component :is="typeConfig[entry.type].icon" :size="16" />
          </div>
          <div class="notif-content">
            <span class="notif-title">{{ entry.title }}</span>
            <span v-if="entry.message" class="notif-message">{{ entry.message }}</span>
            <span class="notif-time">{{ formatTime(entry.timestamp) }}</span>
          </div>
          <SIconButton
            variant="ghost"
            size="sm"
            tooltip="Dismiss"
            class="notif-dismiss"
            @click="store.remove(entry.id)"
          >
            <X :size="14" />
          </SIconButton>
        </div>
      </TransitionGroup>
    </div>
  </SDrawer>
</template>

<style scoped>
.drawer-header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.drawer-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.drawer-actions {
  display: flex;
  align-items: center;
  gap: var(--space-1);
}

/* Body — scrollable list */
.drawer-body {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.drawer-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-3);
  padding: var(--space-12) var(--space-6);
  color: var(--color-text-muted);
  font-size: 13px;
}

.drawer-empty-icon {
  opacity: 0.5;
}

.drawer-list {
  display: flex;
  flex-direction: column;
}

/* Individual notification item */
.notif-item {
  display: flex;
  align-items: flex-start;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-5);
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  transition: background var(--transition-fast);
}

.notif-item:hover {
  background: var(--color-surface-hover);
}

.notif-item--unread {
  background: rgba(20, 184, 166, 0.04);
}

.notif-item--unread:hover {
  background: rgba(20, 184, 166, 0.08);
}

.notif-icon {
  flex-shrink: 0;
  margin-top: 2px;
}

.notif-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--space-0-5);
  min-width: 0;
}

.notif-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
  line-height: 1.4;
}

.notif-message {
  font-size: 12px;
  color: var(--color-text-secondary);
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.notif-time {
  font-size: 11px;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
}

.notif-dismiss {
  flex-shrink: 0;
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.notif-item:hover .notif-dismiss {
  opacity: 1;
}

/* List item transitions */
.notif-enter-active {
  transition: all 0.2s ease-out;
}

.notif-leave-active {
  transition: all 0.15s ease-in;
}

.notif-enter-from {
  opacity: 0;
  transform: translateX(20px);
}

.notif-leave-to {
  opacity: 0;
  transform: translateX(-10px);
}
</style>
