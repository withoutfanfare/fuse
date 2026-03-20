import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { SyncResult, PullRequest, NotificationRule, PrChangeEvent } from '../types'
import { useSettingsStore } from '../stores/settings'
import { useNotificationCentreStore } from '../stores/notificationCentre'
import { computeRiskScore } from './useRiskScore'

/**
 * Composable providing native browser notifications for sync events and
 * configurable PR alert rules. Uses the Web Notification API which is available
 * inside the Tauri webview.
 */
export function useNotifications() {
  const permissionGranted = ref(false)
  const rules = ref<NotificationRule[]>([])

  async function requestPermission() {
    if (!('Notification' in window)) return

    if (Notification.permission === 'granted') {
      permissionGranted.value = true
      return
    }

    if (Notification.permission !== 'denied') {
      const result = await Notification.requestPermission()
      permissionGranted.value = result === 'granted'
    }
  }

  async function loadRules() {
    try {
      rules.value = await invoke<NotificationRule[]>('list_notification_rules')
    } catch {
      rules.value = []
    }
  }

  /**
   * Check if we're currently in quiet hours.
   */
  function isQuietHours(): boolean {
    const settings = useSettingsStore()
    const start = settings.settings.quiet_hours_start
    const end = settings.settings.quiet_hours_end
    if (!start || !end) return false

    const now = new Date()
    const currentMinutes = now.getHours() * 60 + now.getMinutes()
    const [startH, startM] = start.split(':').map(Number)
    const [endH, endM] = end.split(':').map(Number)
    const startMinutes = startH * 60 + startM
    const endMinutes = endH * 60 + endM

    if (startMinutes <= endMinutes) {
      return currentMinutes >= startMinutes && currentMinutes < endMinutes
    }
    // Wraps around midnight
    return currentMinutes >= startMinutes || currentMinutes < endMinutes
  }

  /**
   * Check if notifications are globally enabled.
   */
  function isEnabled(): boolean {
    const settings = useSettingsStore()
    return settings.settings.notifications_enabled !== 'false'
  }

  /**
   * Show a notification when a background sync finds new pull requests.
   */
  function notifyNewPrs(results: SyncResult[]) {
    if (!permissionGranted.value || !isEnabled() || isQuietHours()) return

    const totalNew = results.reduce((sum, r) => sum + (r.pr_count ?? 0), 0)
    if (totalNew === 0) return

    const repos = results
      .filter((r) => r.pr_count > 0 && !r.error)
      .map((r) => r.repo_name)

    if (repos.length === 0) return

    new Notification('Fuse — New pull requests', {
      body: `${totalNew} PR${totalNew === 1 ? '' : 's'} synced across ${repos.join(', ')}`,
    })
  }

  /**
   * Show a notification when any open PR has a risk score at or above the
   * given threshold (default 7).
   */
  function notifyHighRisk(prs: PullRequest[], threshold = 7) {
    if (!permissionGranted.value || !isEnabled() || isQuietHours()) return

    const settings = useSettingsStore()
    const configuredThreshold = Number(settings.settings.notification_risk_threshold) || threshold

    const risky = prs.filter((pr) => {
      const risk = computeRiskScore(pr)
      return risk >= configuredThreshold && pr.state === 'OPEN'
    })

    if (risky.length === 0) return

    new Notification('Fuse — High-risk PRs detected', {
      body: `${risky.length} open PR${risky.length === 1 ? '' : 's'} with elevated risk scores`,
    })
  }

  /**
   * Process sync change events and trigger notifications based on configured rules.
   */
  function notifyChanges(changes: PrChangeEvent[]) {
    if (!permissionGranted.value || !isEnabled() || isQuietHours()) return

    const notifStore = useNotificationCentreStore()

    for (const change of changes) {
      if (change.change_type === 'new') {
        notifStore.push('info', `New PR #${change.pr_number}`, `${change.pr_title} by ${change.author}`)
      } else if (change.change_type === 'merged') {
        notifStore.push('success', `PR #${change.pr_number} Merged`, change.pr_title)
      } else if (change.change_type === 'closed') {
        notifStore.push('warning', `PR #${change.pr_number} Closed`, change.pr_title)
      }
    }
  }

  return {
    permissionGranted,
    rules,
    requestPermission,
    loadRules,
    notifyNewPrs,
    notifyHighRisk,
    notifyChanges,
    isQuietHours,
    isEnabled,
  }
}
