import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { PullRequest, SyncResult, ReviewRule, DashboardStats, BatchResult, AgeBucket, VelocityPoint, AuthorStats, DailyPrCounts } from '../types'
import { useToastStore } from './toast'

export const usePullRequestsStore = defineStore('pullRequests', () => {
  const prs = ref<PullRequest[]>([])
  const loading = ref(false)
  const syncing = ref(false)
  const error = ref<string | null>(null)
  const lastSynced = ref<string | null>(null)
  const stats = ref<DashboardStats | null>(null)

  const openPrs = computed(() => prs.value.filter(pr => pr.state === 'OPEN'))
  const pendingReview = computed(() => prs.value.filter(pr => pr.state === 'OPEN' && (!pr.review_status || pr.review_status === 'pending')))

  async function fetchAll(repoId?: number, statusFilter?: string) {
    loading.value = true
    error.value = null
    try {
      const params: Record<string, unknown> = {}
      if (repoId !== undefined) params.repoId = repoId
      if (statusFilter !== undefined) params.statusFilter = statusFilter
      prs.value = await invoke<PullRequest[]>('get_pull_requests', params)
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function fetchOne(id: number): Promise<PullRequest | null> {
    try {
      return await invoke<PullRequest>('get_pull_request', { id })
    } catch (e) {
      error.value = String(e)
      return null
    }
  }

  async function syncIncremental(repoId?: number): Promise<SyncResult[]> {
    syncing.value = true
    error.value = null
    const toast = useToastStore()
    try {
      const syncParams: Record<string, unknown> = {}
      if (repoId !== undefined) syncParams.repoId = repoId
      const results = await invoke<SyncResult[]>('sync_pull_requests_incremental', syncParams)
      lastSynced.value = new Date().toISOString()
      await Promise.all([fetchAll(), fetchStats()])
      invoke('refresh_menu')
      const failures = results.filter(r => r.error)
      if (failures.length > 0) {
        for (const f of failures) {
          toast.addToast('error', `Sync failed: ${f.repo_name}`, f.error ?? undefined, 8000)
        }
      }
      return results
    } catch (e) {
      const msg = String(e)
      error.value = msg
      toast.addToast('error', 'Incremental sync failed', msg, 8000)
      return []
    } finally {
      syncing.value = false
    }
  }

  async function syncAll(repoId?: number): Promise<SyncResult[]> {
    syncing.value = true
    error.value = null
    const toast = useToastStore()
    try {
      const syncParams: Record<string, unknown> = {}
      if (repoId !== undefined) syncParams.repoId = repoId
      const results = await invoke<SyncResult[]>('sync_pull_requests', syncParams)
      lastSynced.value = new Date().toISOString()
      await Promise.all([fetchAll(), fetchStats()])
      // Refresh the menu bar PR list
      invoke('refresh_menu')

      // Surface per-repo sync errors so the user can investigate
      const failures = results.filter(r => r.error)
      if (failures.length > 0) {
        for (const f of failures) {
          toast.addToast('error', `Sync failed: ${f.repo_name}`, f.error ?? undefined, 8000)
        }
      }

      return results
    } catch (e) {
      const msg = String(e)
      error.value = msg
      toast.addToast('error', 'Sync failed', msg, 8000)
      return []
    } finally {
      syncing.value = false
    }
  }

  async function updateReviewStatus(prId: number, status: string, notes?: string) {
    try {
      const params: Record<string, unknown> = { prId, status }
      if (notes !== undefined) params.notes = notes
      await invoke('update_review_status', params)
    } catch (e) {
      error.value = String(e)
    }
  }

  async function fetchStats() {
    try {
      stats.value = await invoke<DashboardStats>('get_dashboard_stats')
    } catch (e) {
      error.value = String(e)
    }
  }

  async function fetchRules(repoId: number): Promise<ReviewRule[]> {
    try {
      return await invoke<ReviewRule[]>('get_review_rules', { repoId })
    } catch (e) {
      error.value = String(e)
      return []
    }
  }

  async function setRules(repoId: number, rules: string[]) {
    try {
      await invoke('set_review_rules', { repoId, rules })
    } catch (e) {
      error.value = String(e)
    }
  }

  async function approvePr(prId: number, body?: string): Promise<string | null> {
    error.value = null
    try {
      const params: Record<string, unknown> = { prId }
      if (body !== undefined) params.body = body
      const result = await invoke<string>('approve_pull_request', params)
      invoke('refresh_menu')
      return result
    } catch (e) {
      error.value = String(e)
      return null
    }
  }

  async function mergePr(prId: number, mergeMethod?: string): Promise<string | null> {
    error.value = null
    try {
      const params: Record<string, unknown> = { prId }
      if (mergeMethod !== undefined) params.mergeMethod = mergeMethod
      const result = await invoke<string>('merge_pull_request', params)
      invoke('refresh_menu')
      return result
    } catch (e) {
      error.value = String(e)
      return null
    }
  }

  async function fetchStalePrs(): Promise<PullRequest[]> {
    try {
      return await invoke<PullRequest[]>('get_stale_prs')
    } catch (e) {
      error.value = String(e)
      return []
    }
  }

  async function batchApprove(prIds: number[], body?: string): Promise<BatchResult[]> {
    error.value = null
    try {
      const params: Record<string, unknown> = { prIds }
      if (body !== undefined) params.body = body
      const results = await invoke<BatchResult[]>('batch_approve', params)
      invoke('refresh_menu')
      return results
    } catch (e) {
      error.value = String(e)
      return []
    }
  }

  async function batchMerge(prIds: number[], method?: string): Promise<BatchResult[]> {
    error.value = null
    try {
      const params: Record<string, unknown> = { prIds }
      if (method !== undefined) params.method = method
      const results = await invoke<BatchResult[]>('batch_merge', params)
      invoke('refresh_menu')
      return results
    } catch (e) {
      error.value = String(e)
      return []
    }
  }

  async function closePr(prId: number): Promise<boolean> {
    error.value = null
    try {
      await invoke('close_pull_request', { prId })
      invoke('refresh_menu')
      return true
    } catch (e) {
      error.value = String(e)
      return false
    }
  }

  async function fetchAgeDistribution(): Promise<AgeBucket[]> {
    try {
      return await invoke<AgeBucket[]>('get_age_distribution')
    } catch (e) {
      error.value = String(e)
      return []
    }
  }

  async function fetchReviewVelocity(days?: number): Promise<VelocityPoint[]> {
    try {
      const params: Record<string, unknown> = {}
      if (days !== undefined) params.days = days
      return await invoke<VelocityPoint[]>('get_review_velocity', params)
    } catch (e) {
      error.value = String(e)
      return []
    }
  }

  async function fetchAuthorStats(): Promise<AuthorStats[]> {
    try {
      return await invoke<AuthorStats[]>('get_author_stats')
    } catch (e) {
      error.value = String(e)
      return []
    }
  }

  async function fetchDailyPrCounts(): Promise<DailyPrCounts> {
    try {
      return await invoke<DailyPrCounts>('get_daily_pr_counts')
    } catch (e) {
      error.value = String(e)
      return { open_counts: [], pending_counts: [] }
    }
  }

  return {
    prs, loading, syncing, error, lastSynced, stats,
    openPrs, pendingReview,
    fetchAll, fetchOne, syncAll, syncIncremental, updateReviewStatus, fetchStats, fetchRules, setRules,
    approvePr, mergePr, batchApprove, batchMerge, fetchStalePrs, closePr,
    fetchAgeDistribution, fetchReviewVelocity, fetchAuthorStats, fetchDailyPrCounts,
  }
})
