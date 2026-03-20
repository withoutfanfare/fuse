import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { PrDependency, PullRequest } from '../types'

export interface DependencyNode {
  id: number
  prNumber: number
  title: string
  author: string
  /** Number of PRs this one blocks (dependants) */
  blocksCount: number
  /** Number of PRs this one depends on */
  dependsOnCount: number
  /** Whether this PR is a merge-order risk (has unmerged dependencies) */
  isRisk: boolean
}

export interface DependencyEdge {
  from: number  // pr_id that depends
  to: number    // pr_id that is depended upon
  type: 'body_reference' | 'branch_ancestry'
}

export function useDependencyGraph() {
  const dependencies = ref<PrDependency[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  /** Compute dependencies from all open PRs and persist to SQLite. */
  async function computeDependencies() {
    loading.value = true
    error.value = null
    try {
      dependencies.value = await invoke<PrDependency[]>('compute_dependencies')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  /** Fetch previously computed dependencies without re-computing. */
  async function fetchDependencies() {
    loading.value = true
    error.value = null
    try {
      dependencies.value = await invoke<PrDependency[]>('get_pr_dependencies')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  /** Build graph nodes and edges from dependencies and a list of open PRs. */
  function buildGraph(prs: PullRequest[]) {
    const prMap = new Map(prs.map(pr => [pr.id, pr]))

    const edges: DependencyEdge[] = dependencies.value.map(d => ({
      from: d.pr_id,
      to: d.depends_on_pr_id,
      type: d.dependency_type,
    }))

    // Gather all PR IDs involved in dependencies
    const involvedIds = new Set<number>()
    for (const d of dependencies.value) {
      involvedIds.add(d.pr_id)
      involvedIds.add(d.depends_on_pr_id)
    }

    // Count how many PRs each PR blocks and depends on
    const blocksMap = new Map<number, number>()
    const dependsMap = new Map<number, number>()
    for (const d of dependencies.value) {
      blocksMap.set(d.depends_on_pr_id, (blocksMap.get(d.depends_on_pr_id) ?? 0) + 1)
      dependsMap.set(d.pr_id, (dependsMap.get(d.pr_id) ?? 0) + 1)
    }

    const nodes: DependencyNode[] = []
    for (const id of involvedIds) {
      const pr = prMap.get(id)
      if (!pr) continue
      const dependsOnCount = dependsMap.get(id) ?? 0
      nodes.push({
        id,
        prNumber: pr.number,
        title: pr.title,
        author: pr.author,
        blocksCount: blocksMap.get(id) ?? 0,
        dependsOnCount,
        // A risk if this PR has unmerged dependencies
        isRisk: dependsOnCount > 0,
      })
    }

    return { nodes, edges }
  }

  const hasDependencies = computed(() => dependencies.value.length > 0)

  return {
    dependencies,
    loading,
    error,
    hasDependencies,
    computeDependencies,
    fetchDependencies,
    buildGraph,
  }
}
