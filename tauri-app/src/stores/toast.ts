import { defineStore } from 'pinia'
import { useToastStack } from '@stuntrocket/ui'
import type { ToastType } from '../types'
import { useNotificationCentreStore } from './notificationCentre'

/** Map local ToastType to library variant names (identical in this case) */
const variantMap: Record<ToastType, 'success' | 'error' | 'warning' | 'info'> = {
  success: 'success',
  error: 'error',
  warning: 'warning',
  info: 'info',
}

export const useToastStore = defineStore('toast', () => {
  const stack = useToastStack()

  /**
   * Show a toast notification.
   *
   * The library's useToastStack uses a single `message` string, so
   * title and message are combined (title — message) when both are provided.
   */
  function addToast(
    type: ToastType,
    title: string,
    message?: string,
    duration = 4000,
  ) {
    const text = message ? `${title} — ${message}` : title
    stack.addToast(text, variantMap[type], duration)

    /* Also push to the notification centre for persistent history */
    const notifCentre = useNotificationCentreStore()
    notifCentre.push(type, title, message)
  }

  return { toasts: stack.toasts, addToast, removeToast: stack.removeToast }
})
