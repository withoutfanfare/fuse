<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import StatsCard from '../components/StatsCard.vue'
import SkeletonStatsCard from '../components/skeletons/SkeletonStatsCard.vue'
import ContentLoader from '../components/ContentLoader.vue'
import { SSectionHeader, SEmptyState, SCard } from '@stuntrocket/ui'
import type { AggregateDashboard } from '../types'

const router = useRouter()

const loading = ref(true)
const error = ref<string | null>(null)
const dashboard = ref<AggregateDashboard | null>(null)

onMounted(async () => {
  try {
    dashboard.value = await invoke<AggregateDashboard>('get_aggregate_dashboard')
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
})

function navigateToPr(prId: number) {
  router.push({ name: 'pr-detail', params: { id: prId } })
}

function navigateToRepo(repoId: number) {
  router.push({ name: 'pull-requests', query: { repoId: String(repoId) } })
}

function formatAge(hours: number): string {
  if (hours < 1) return '<1h'
  if (hours < 24) return `${Math.round(hours)}h`
  const days = Math.floor(hours / 24)
  return `${days}d`
}

function formatSyncTime(syncAt: string | null): string {
  if (!syncAt) return 'Never'
  const diffMs = Date.now() - new Date(syncAt).getTime()
  const diffMin = Math.floor(diffMs / 60000)
  if (diffMin < 1) return 'just now'
  if (diffMin < 60) return `${diffMin}m ago`
  const diffHr = Math.floor(diffMin / 60)
  if (diffHr < 24) return `${diffHr}h ago`
  return `${Math.floor(diffHr / 24)}d ago`
}

function riskBadgeClass(score: number): string {
  if (score <= 3) return 'risk-low'
  if (score <= 5) return 'risk-medium'
  if (score <= 7) return 'risk-high'
  return 'risk-critical'
}
</script>

<template>
  <div class="aggregate-dashboard">
    <h1 class="page-title">Aggregate Dashboard</h1>
    <p class="page-subtitle">Cross-repository overview of your review workload</p>

    <template v-if="loading">
      <section class="stats-row">
        <SkeletonStatsCard v-for="n in 4" :key="`skeleton-${n}`" />
      </section>
      <ContentLoader variant="list" :count="3" />
    </template>

    <template v-else-if="error">
      <div class="error-banner">{{ error }}</div>
    </template>

    <template v-else-if="dashboard">
      <section class="stats-row">
        <StatsCard
          :value="dashboard.total_open_prs"
          label="Total Open PRs"
          variant="info"
        />
        <StatsCard
          :value="dashboard.review_requested_count"
          label="Reviews Requested"
          variant="warning"
        />
        <StatsCard
          :value="dashboard.high_risk_count"
          label="High Risk"
          variant="danger"
        />
        <StatsCard
          :value="dashboard.stale_count"
          label="Stale (>3d)"
          variant="neutral"
        />
      </section>

      <!-- Top risk PRs -->
      <section class="section">
        <SSectionHeader title="Highest Risk PRs" />
        <SEmptyState
          v-if="dashboard.top_risk_prs.length === 0"
          title="No pull requests"
          description="No open pull requests across any repository."
        />
        <div v-else class="risk-table">
          <div
            v-for="pr in dashboard.top_risk_prs"
            :key="pr.pr_id"
            class="risk-row"
            @click="navigateToPr(pr.pr_id)"
          >
            <span class="risk-badge" :class="riskBadgeClass(pr.risk_score)">
              {{ pr.risk_score.toFixed(1) }}
            </span>
            <span class="risk-pr-title">
              <span class="risk-pr-number">#{{ pr.number }}</span>
              {{ pr.title }}
            </span>
            <span class="risk-repo-badge">{{ pr.repo_name }}</span>
            <span class="risk-author">{{ pr.author }}</span>
            <span class="risk-stats">+{{ pr.additions }}/-{{ pr.deletions }}</span>
          </div>
        </div>
      </section>

      <!-- Per-repository summary -->
      <section class="section">
        <SSectionHeader title="Repositories" />
        <SEmptyState
          v-if="dashboard.repo_summaries.length === 0"
          title="No repositories"
          description="No repositories added yet."
        />
        <div v-else class="repo-grid">
          <SCard
            v-for="repo in dashboard.repo_summaries"
            :key="repo.repo_id"
            variant="content"
            hoverable
            class="repo-card"
            @click="navigateToRepo(repo.repo_id)"
          >
            <div class="repo-card-header">
              <span class="repo-card-name">{{ repo.repo_name }}</span>
              <span class="repo-card-count">{{ repo.open_pr_count }} open</span>
            </div>
            <div class="repo-card-details">
              <span v-if="repo.open_pr_count > 0" class="repo-card-age">
                Oldest: {{ formatAge(repo.oldest_pr_age_hours) }}
              </span>
              <span class="repo-card-sync">
                Synced: {{ formatSyncTime(repo.last_sync_at) }}
              </span>
            </div>
          </SCard>
        </div>
      </section>
    </template>
  </div>
</template>

<style scoped>
.aggregate-dashboard {
  width: 100%;
}

.page-title {
  font-size: var(--text-heading-size);
  font-weight: var(--text-heading-weight);
  letter-spacing: var(--text-heading-tracking);
  color: var(--color-text-primary);
  margin-bottom: var(--space-1);
}

.page-subtitle {
  font-size: 13px;
  color: var(--color-text-muted);
  margin-bottom: var(--space-5);
}

.stats-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: var(--space-3);
  margin-bottom: var(--space-5);
}

.section {
  margin-bottom: var(--space-5);
}


.error-banner {
  background: rgba(220, 38, 38, 0.1);
  border: 1px solid rgba(220, 38, 38, 0.3);
  color: var(--color-status-danger);
  padding: var(--space-4);
  border-radius: var(--radius-lg);
  font-size: 13px;
}

/* Risk table */
.risk-table {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  overflow: hidden;
}

.risk-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--color-border-default);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.risk-row:last-child {
  border-bottom: none;
}

.risk-row:hover {
  background: var(--color-surface-hover);
}

.risk-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 36px;
  height: 24px;
  border-radius: var(--radius-full);
  font-size: 11px;
  font-weight: 700;
  font-family: var(--font-mono);
  flex-shrink: 0;
}

.risk-low { background: rgba(34, 197, 94, 0.2); color: var(--color-status-success); }
.risk-medium { background: rgba(234, 179, 8, 0.2); color: var(--color-status-warning); }
.risk-high { background: rgba(249, 115, 22, 0.2); color: #f97316; }
.risk-critical { background: rgba(220, 38, 38, 0.2); color: var(--color-status-danger); }

.risk-pr-title {
  flex: 1;
  font-size: 13px;
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.risk-pr-number {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--color-text-muted);
  margin-right: var(--space-1);
}

.risk-repo-badge {
  background: rgba(20, 184, 166, 0.12);
  color: var(--color-accent);
  font-size: 11px;
  padding: 2px var(--space-2);
  border-radius: var(--radius-full);
  font-weight: 500;
  flex-shrink: 0;
  white-space: nowrap;
}

.risk-author {
  font-size: 12px;
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.risk-stats {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--color-text-muted);
  flex-shrink: 0;
}

/* Repository grid */
.repo-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--space-3);
}

.repo-card {
  cursor: pointer;
}

.repo-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-2);
}

.repo-card-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.repo-card-count {
  font-size: 12px;
  font-family: var(--font-mono);
  font-weight: 600;
  color: var(--color-accent);
}

.repo-card-details {
  display: flex;
  gap: var(--space-4);
  font-size: 12px;
  color: var(--color-text-muted);
}
</style>
