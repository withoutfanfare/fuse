import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Toast, ToastType } from '../types'
import { useNotificationCentreStore } from './notificationCentre'

export const useToastStore = defineStore('toast', () => {
  const toasts = ref<Toast[]>([])

  let nextId = 1

  function addToast(
    type: ToastType,
    title: string,
    message?: string,
    duration = 4000,
  ) {
    const id = nextId++
    const toast: Toast = { id, type, title, message, duration }
    toasts.value.push(toast)

    /* Also push to the notification centre for persistent history */
    const notifCentre = useNotificationCentreStore()
    notifCentre.push(type, title, message)

    if (duration > 0) {
      setTimeout(() => removeToast(id), duration)
    }

    return id
  }

  function removeToast(id: number) {
    const idx = toasts.value.findIndex(t => t.id === id)
    if (idx !== -1) {
      toasts.value.splice(idx, 1)
    }
  }

  return { toasts, addToast, removeToast }
})
