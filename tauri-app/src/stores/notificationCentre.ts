import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ToastType } from '../types'

/**
 * Notification Centre store.
 *
 * Accumulates all application notifications (capped at 100) so the
 * user can review them later via the notification drawer.  Existing
 * toasts still fire ephemerally — this store provides the persistent
 * history layer.
 */

export interface NotificationEntry {
  id: number
  type: ToastType
  title: string
  message?: string
  timestamp: number
  read: boolean
}

const MAX_ENTRIES = 100

export const useNotificationCentreStore = defineStore('notificationCentre', () => {
  const entries = ref<NotificationEntry[]>([])
  const drawerOpen = ref(false)

  let nextId = 1

  /** Number of unread notifications */
  const unreadCount = computed(() =>
    entries.value.filter(e => !e.read).length,
  )

  /** Push a new notification entry. Caps at MAX_ENTRIES (oldest removed first). */
  function push(type: ToastType, title: string, message?: string) {
    const entry: NotificationEntry = {
      id: nextId++,
      type,
      title,
      message,
      timestamp: Date.now(),
      read: false,
    }
    entries.value.unshift(entry)
    if (entries.value.length > MAX_ENTRIES) {
      entries.value = entries.value.slice(0, MAX_ENTRIES)
    }
  }

  /** Mark a single notification as read */
  function markRead(id: number) {
    const entry = entries.value.find(e => e.id === id)
    if (entry) entry.read = true
  }

  /** Mark all notifications as read */
  function markAllRead() {
    entries.value.forEach(e => { e.read = true })
  }

  /** Remove a single notification */
  function remove(id: number) {
    const idx = entries.value.findIndex(e => e.id === id)
    if (idx !== -1) entries.value.splice(idx, 1)
  }

  /** Clear all notifications */
  function clearAll() {
    entries.value = []
  }

  /** Toggle the drawer open/closed */
  function toggleDrawer() {
    drawerOpen.value = !drawerOpen.value
    /* Mark all as read when opening */
    if (drawerOpen.value) {
      markAllRead()
    }
  }

  function openDrawer() {
    drawerOpen.value = true
    markAllRead()
  }

  function closeDrawer() {
    drawerOpen.value = false
  }

  return {
    entries,
    drawerOpen,
    unreadCount,
    push,
    markRead,
    markAllRead,
    remove,
    clearAll,
    toggleDrawer,
    openDrawer,
    closeDrawer,
  }
})
