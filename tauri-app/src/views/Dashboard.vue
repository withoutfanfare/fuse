<script setup lang="ts">
import { defineAsyncComponent, onMounted, computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { usePullRequestsStore } from '../stores/pullRequests'
import { computeRiskScore } from '../composables/useRiskScore'
import { useDependencyGraph, type DependencyNode, type DependencyEdge } from '../composables/useDependencyGraph'
import { CheckCircle } from 'lucide-vue-next'
import StatsCard from '../components/StatsCard.vue'
import SkeletonStatsCard from '../components/skeletons/SkeletonStatsCard.vue'
import PRCard from '../components/PRCard.vue'
import { SEmptyState } from '@stuntrocket/ui'
import ReviewProgress from '../components/ReviewProgress.vue'
import StalePrSection from '../components/StalePrSection.vue'
import AgeHeatmap from '../components/AgeHeatmap.vue'
import VelocityChart from '../components/VelocityChart.vue'
import PriorityQueue from '../components/PriorityQueue.vue'
import WorkloadDashboard from '../components/WorkloadDashboard.vue'
import DependencyGraph from '../components/DependencyGraph.vue'
import ContentLoader from '../components/ContentLoader.vue'
const ReviewTimeDashboard = defineAsyncComponent(() => import('../components/ReviewTimeDashboard.vue'))
import type { AgeBucket, VelocityPoint, DailyPrCounts, PullRequest, PriorityQueueItem, ReviewerWorkloadStats } from '../types'

const router = useRouter()
const prStore = usePullRequestsStore()
const { fetchDependencies, computeDependencies, buildGraph, loading: depLoading, error: depError } = useDependencyGraph()

const initialLoad = ref(true)
const ageBuckets = ref<AgeBucket[]>([])
const velocityData = ref<VelocityPoint[]>([])
const dailyCounts = ref<DailyPrCounts>({ open_counts: [], pending_counts: [] })

/* Dashboard-level state for child component data (hoisted from child onMounted calls) */
const priorityQueue = ref<PriorityQueueItem[]>([])
const priorityLoading = ref(false)
const priorityError = ref<string | null>(null)

const workloadData = ref<ReviewerWorkloadStats[]>([])
const workloadLoading = ref(false)
const workloadError = ref<string | null>(null)

const stalePrs = ref<PullRequest[]>([])
const staleLoading = ref(true)

/* Dependency graph node/edge state */
const depNodes = ref<DependencyNode[]>([])
const depEdges = ref<DependencyEdge[]>([])

onMounted(async () => {
  // Critical — gate the skeleton
  await Promise.all([
    prStore.fetchStats(),
    prStore.prs.length === 0 ? prStore.fetchAll() : Promise.resolve(),
  ])
  initialLoad.value = false

  // All secondary IPC calls in a single coordinated batch
  priorityLoading.value = true
  workloadLoading.value = true

  const [buckets, velocity, counts, queueResult, workloadResult, staleResult] = await Promise.all([
    prStore.fetchAgeDistribution(),
    prStore.fetchReviewVelocity(),
    prStore.fetchDailyPrCounts(),
    invoke<PriorityQueueItem[]>('get_priority_queue').catch((e: unknown) => { priorityError.value = String(e); return [] as PriorityQueueItem[] }),
    invoke<ReviewerWorkloadStats[]>('get_reviewer_workload').catch((e: unknown) => { workloadError.value = String(e); return [] as ReviewerWorkloadStats[] }),
    prStore.fetchStalePrs(),
    fetchDependencies(),
  ])

  ageBuckets.value = buckets
  velocityData.value = velocity
  dailyCounts.value = counts
  priorityQueue.value = queueResult
  priorityLoading.value = false
  workloadData.value = workloadResult
  workloadLoading.value = false
  stalePrs.value = staleResult
  staleLoading.value = false

  // Build dependency graph from fetched data
  const graph = buildGraph(prStore.openPrs)
  depNodes.value = graph.nodes
  depEdges.value = graph.edges
})

/** Refresh priority queue on explicit user action */
async function refreshPriorityQueue() {
  priorityLoading.value = true
  priorityError.value = null
  try {
    priorityQueue.value = await invoke<PriorityQueueItem[]>('get_priority_queue')
  } catch (e) {
    priorityError.value = String(e)
  } finally {
    priorityLoading.value = false
  }
}

/** Refresh workload on explicit user action */
async function refreshWorkload() {
  workloadLoading.value = true
  workloadError.value = null
  try {
    workloadData.value = await invoke<ReviewerWorkloadStats[]>('get_reviewer_workload')
  } catch (e) {
    workloadError.value = String(e)
  } finally {
    workloadLoading.value = false
  }
}

/** Refresh dependency graph — recompute on explicit user action (write operation) */
async function refreshDependencies() {
  await computeDependencies()
  const graph = buildGraph(prStore.openPrs)
  depNodes.value = graph.nodes
  depEdges.value = graph.edges
}

/** 7-day trend for the "Open PRs" sparkline */
const openTrend = computed(() => dailyCounts.value.open_counts)

/** 7-day trend for the "Pending Review" sparkline */
const pendingTrend = computed(() => dailyCounts.value.pending_counts)

/** Derive a 7-day "reviewed per day" trend from velocity data (last 7 points) */
const reviewedTrend = computed(() => {
  if (velocityData.value.length < 2) return []
  return velocityData.value.slice(-7).map(p => p.reviewed)
})

/** Derive a 7-day "merged per day" trend from velocity data (last 7 points) */
const mergedTrend = computed(() => {
  if (velocityData.value.length < 2) return []
  return velocityData.value.slice(-7).map(p => p.merged)
})

const urgentPrs = computed(() => {
  return [...prStore.openPrs]
    .sort((a, b) => computeRiskScore(b) - computeRiskScore(a))
    .slice(0, 5)
})

const totalOpenPrs = computed(() => prStore.openPrs.length)
const reviewedPrs = computed(() => {
  return prStore.openPrs.filter(pr =>
    pr.review_status === 'reviewed' || pr.review_status === 'approved'
  ).length
})

function openDetail(id: number) {
  router.push({ name: 'pr-detail', params: { id } })
}
</script>

<template>
  <div class="dashboard">
    <section class="stats-row">
      <template v-if="initialLoad || prStore.loading">
        <SkeletonStatsCard v-for="n in 5" :key="`skeleton-${n}`" />
      </template>
      <TransitionGroup v-else name="card-cascade">
        <StatsCard
          key="open-prs"
          :value="prStore.stats?.total_open_prs ?? 0"
          label="Open PRs"
          variant="info"
          :history="openTrend"
          :style="{ transitionDelay: '0ms' }"
        />
        <StatsCard
          key="pending-review"
          :value="prStore.stats?.pending_reviews ?? 0"
          label="Pending Review"
          variant="warning"
          :history="pendingTrend"
          :style="{ transitionDelay: '50ms' }"
        />
        <StatsCard
          key="in-progress"
          :value="prStore.stats?.in_progress ?? 0"
          label="In Progress"
          variant="info"
          :history="reviewedTrend"
          :style="{ transitionDelay: '100ms' }"
        />
        <StatsCard
          key="approved"
          :value="prStore.stats?.approved ?? 0"
          label="Approved"
          variant="success"
          :history="mergedTrend"
          :style="{ transitionDelay: '150ms' }"
        />
        <ReviewProgress key="progress" :reviewed="reviewedPrs" :total="totalOpenPrs" />
      </TransitionGroup>
    </section>

    <template v-if="initialLoad">
      <ContentLoader variant="list" :count="3" />
      <ContentLoader variant="cards" :count="4" />
    </template>

    <template v-else>
      <section class="priority-section">
        <PriorityQueue
          :queue="priorityQueue"
          :loading="priorityLoading"
          :error="priorityError"
          @refresh="refreshPriorityQueue"
        />
      </section>

      <section class="urgent-section">
        <h2 class="section-title">Needs Attention</h2>
        <div v-if="urgentPrs.length > 0" class="urgent-grid">
          <PRCard
            v-for="pr in urgentPrs"
            :key="pr.id"
            :pr="pr"
            @open-detail="openDetail"
          />
        </div>
        <SEmptyState
          v-else
          title="All caught up"
          description="No pull requests need your attention right now. Time to ship."
        >
          <template #icon><CheckCircle :size="36" /></template>
        </SEmptyState>
      </section>

      <section class="dependency-section">
        <DependencyGraph
          :nodes="depNodes"
          :edges="depEdges"
          :loading="depLoading"
          :error="depError"
          @refresh="refreshDependencies"
        />
      </section>

      <section class="workload-section">
        <div class="analytics-card">
          <WorkloadDashboard
            :workload="workloadData"
            :loading="workloadLoading"
            :error="workloadError"
            @refresh="refreshWorkload"
          />
        </div>
      </section>

      <section class="analytics-grid">
        <div class="analytics-card">
          <h2 class="section-title">PR Age Distribution</h2>
          <AgeHeatmap :buckets="ageBuckets" />
        </div>
        <div class="analytics-card">
          <h2 class="section-title">Review Velocity</h2>
          <VelocityChart :data="velocityData" />
        </div>
      </section>

      <StalePrSection :stale-prs="stalePrs" :loading="staleLoading" @update:stale-prs="stalePrs = $event" />

      <section class="review-time-section">
        <ReviewTimeDashboard />
      </section>
    </template>
  </div>
</template>

<style scoped>
.dashboard {
  width: 100%;
}

.stats-row {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: var(--space-3);
  margin-bottom: var(--space-5);
}

.section-title {
  font-size: var(--text-heading-size);
  font-weight: var(--text-heading-weight);
  letter-spacing: var(--text-heading-tracking);
  line-height: var(--text-heading-leading);
  margin-bottom: var(--space-3);
  color: var(--color-text-primary);
}

.priority-section {
  margin-bottom: var(--space-5);
}

.urgent-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: var(--space-3);
}

.dependency-section {
  margin-top: var(--space-5);
  margin-bottom: var(--space-5);
}

.workload-section {
  margin-top: var(--space-5);
  margin-bottom: var(--space-5);
}

.analytics-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-3);
  margin-top: var(--space-5);
  margin-bottom: var(--space-5);
}

.analytics-card {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-4);
}

.review-time-section {
  margin-top: var(--space-5);
  margin-bottom: var(--space-5);
}

</style>
