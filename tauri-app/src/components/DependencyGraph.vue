<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { SButton, SCard, SEmptyState, SSectionHeader } from '@stuntrocket/ui'
import type { DependencyNode, DependencyEdge } from '../composables/useDependencyGraph'
import ContentLoader from './ContentLoader.vue'

const props = withDefaults(defineProps<{
  nodes: DependencyNode[]
  edges: DependencyEdge[]
  loading?: boolean
  error?: string | null
}>(), {
  loading: false,
  error: null,
})

const emit = defineEmits<{
  refresh: []
}>()

const router = useRouter()

// SVG dimensions
const svgWidth = 800
const svgHeight = 200
const nodeRadius = 18
const padding = 40

interface PositionedNode extends DependencyNode {
  x: number
  y: number
}

/** Layout nodes in a simple left-to-right topological arrangement. */
const positionedNodes = computed<PositionedNode[]>(() => {
  if (props.nodes.length === 0) return []

  // Compute topological levels via BFS from root nodes (those with no dependencies)
  const levels = new Map<number, number>()
  const adjList = new Map<number, number[]>()
  const inDegree = new Map<number, number>()

  for (const node of props.nodes) {
    adjList.set(node.id, [])
    inDegree.set(node.id, 0)
  }

  for (const edge of props.edges) {
    const dependants = adjList.get(edge.from) ?? []
    dependants.push(edge.to)
    adjList.set(edge.from, dependants)
    inDegree.set(edge.to, (inDegree.get(edge.to) ?? 0) + 1)
  }

  // BFS to assign levels (root = level 0, dependants go right)
  const queue: number[] = []
  for (const node of props.nodes) {
    if ((inDegree.get(node.id) ?? 0) === 0) {
      queue.push(node.id)
      levels.set(node.id, 0)
    }
  }

  while (queue.length > 0) {
    const current = queue.shift()!
    const currentLevel = levels.get(current) ?? 0
    const dependants = adjList.get(current) ?? []
    for (const dep of dependants) {
      const existingLevel = levels.get(dep) ?? -1
      const newLevel = currentLevel + 1
      if (newLevel > existingLevel) {
        levels.set(dep, newLevel)
      }
      // Decrease virtual in-degree to process in topological order
      const remaining = (inDegree.get(dep) ?? 1) - 1
      inDegree.set(dep, remaining)
      if (remaining <= 0) {
        queue.push(dep)
      }
    }
  }

  // Group nodes by level
  const levelGroups = new Map<number, DependencyNode[]>()
  let maxLevel = 0
  for (const node of props.nodes) {
    const level = levels.get(node.id) ?? 0
    maxLevel = Math.max(maxLevel, level)
    const group = levelGroups.get(level) ?? []
    group.push(node)
    levelGroups.set(level, group)
  }

  // Position nodes
  const usableWidth = svgWidth - 2 * padding
  const usableHeight = svgHeight - 2 * padding
  const colSpacing = maxLevel > 0 ? usableWidth / maxLevel : usableWidth

  const positioned: PositionedNode[] = []
  for (const [level, group] of levelGroups) {
    const rowSpacing = group.length > 1 ? usableHeight / (group.length - 1) : 0
    const startY = group.length > 1 ? padding : svgHeight / 2

    for (let i = 0; i < group.length; i++) {
      positioned.push({
        ...group[i],
        x: padding + level * colSpacing,
        y: startY + i * rowSpacing,
      })
    }
  }

  return positioned
})

/** Get positioned node by ID for drawing edges. */
function getNode(id: number): PositionedNode | undefined {
  return positionedNodes.value.find(n => n.id === id)
}

/** Compute an SVG path for a curved edge between two nodes. */
function edgePath(edge: DependencyEdge): string {
  const fromNode = getNode(edge.from)
  const toNode = getNode(edge.to)
  if (!fromNode || !toNode) return ''

  const dx = toNode.x - fromNode.x
  const cpx = dx * 0.5

  return `M ${fromNode.x} ${fromNode.y} C ${fromNode.x + cpx} ${fromNode.y}, ${toNode.x - cpx} ${toNode.y}, ${toNode.x} ${toNode.y}`
}

function navigateToPr(nodeId: number) {
  router.push({ name: 'pr-detail', params: { id: nodeId } })
}
</script>

<template>
  <SCard variant="content" class="dependency-graph-panel">
    <div class="graph-header">
      <SSectionHeader title="PR Dependency Graph" />
      <SButton variant="secondary" size="sm" :disabled="loading" :loading="loading" @click="emit('refresh')">
        Refresh
      </SButton>
    </div>

    <div v-if="error" class="graph-error">{{ error }}</div>

    <ContentLoader v-if="loading" variant="cards" :count="3" />

    <SEmptyState
      v-else-if="positionedNodes.length === 0"
      title="No dependencies"
      description="No cross-PR dependencies detected."
    />

    <div v-else class="graph-container">
      <svg :viewBox="`0 0 ${svgWidth} ${svgHeight}`" class="graph-svg">
        <defs>
          <marker
            id="arrow"
            viewBox="0 0 10 10"
            refX="10"
            refY="5"
            markerWidth="6"
            markerHeight="6"
            orient="auto"
          >
            <path d="M 0 0 L 10 5 L 0 10 z" fill="var(--color-text-muted)" />
          </marker>
          <marker
            id="arrow-risk"
            viewBox="0 0 10 10"
            refX="10"
            refY="5"
            markerWidth="6"
            markerHeight="6"
            orient="auto"
          >
            <path d="M 0 0 L 10 5 L 0 10 z" fill="var(--color-status-warning)" />
          </marker>
        </defs>

        <!-- Edges -->
        <path
          v-for="(edge, idx) in props.edges"
          :key="`edge-${idx}`"
          :d="edgePath(edge)"
          fill="none"
          :stroke="edge.type === 'branch_ancestry' ? 'rgba(139, 92, 246, 0.5)' : 'rgba(100, 116, 139, 0.4)'"
          stroke-width="2"
          marker-end="url(#arrow)"
        />

        <!-- Nodes -->
        <g
          v-for="node in positionedNodes"
          :key="node.id"
          class="graph-node"
          :class="{ 'graph-node-risk': node.isRisk }"
          @click="navigateToPr(node.id)"
        >
          <circle
            :cx="node.x"
            :cy="node.y"
            :r="nodeRadius"
            :class="node.isRisk ? 'node-circle-risk' : 'node-circle'"
          />
          <text
            :x="node.x"
            :y="node.y - 4"
            text-anchor="middle"
            class="node-number"
          >
            #{{ node.prNumber }}
          </text>
          <text
            :x="node.x"
            :y="node.y + 10"
            text-anchor="middle"
            class="node-label"
          >
            {{ node.author }}
          </text>

          <!-- Risk indicator -->
          <circle
            v-if="node.isRisk"
            :cx="node.x + nodeRadius - 4"
            :cy="node.y - nodeRadius + 4"
            r="6"
            class="risk-dot"
          />

          <!-- Tooltip title -->
          <title>{{ node.title }} ({{ node.blocksCount }} blocked, {{ node.dependsOnCount }} dependencies)</title>
        </g>
      </svg>

      <div class="graph-legend">
        <div class="legend-item">
          <span class="legend-swatch legend-branch" />
          <span class="legend-label">Branch ancestry (stacked PRs)</span>
        </div>
        <div class="legend-item">
          <span class="legend-swatch legend-body" />
          <span class="legend-label">Body reference (Depends on / Blocked by)</span>
        </div>
        <div class="legend-item">
          <span class="legend-dot legend-risk-dot" />
          <span class="legend-label">Merge-order risk</span>
        </div>
      </div>
    </div>
  </SCard>
</template>

<style scoped>
:deep(.py-12) {
  padding-top: var(--space-4) !important;
  padding-bottom: var(--space-4) !important;
}

.graph-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-2);
}

.graph-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.graph-error {
  color: var(--color-status-danger);
  font-size: 13px;
  padding: var(--space-3);
}

.graph-container {
  overflow-x: auto;
}

.graph-svg {
  width: 100%;
  height: auto;
  min-height: 140px;
}

.graph-node {
  cursor: pointer;
}

.node-circle {
  fill: rgba(30, 41, 59, 0.8);
  stroke: var(--color-border-default);
  stroke-width: 2;
  transition: all var(--transition-fast);
}

.graph-node:hover .node-circle {
  stroke: var(--color-accent);
  fill: rgba(20, 184, 166, 0.1);
}

.node-circle-risk {
  fill: rgba(234, 179, 8, 0.1);
  stroke: var(--color-status-warning);
  stroke-width: 2;
  transition: all var(--transition-fast);
}

.graph-node:hover .node-circle-risk {
  fill: rgba(234, 179, 8, 0.2);
}

.node-number {
  fill: var(--color-text-primary);
  font-size: 10px;
  font-weight: 700;
  font-family: var(--font-mono);
  pointer-events: none;
}

.node-label {
  fill: var(--color-text-muted);
  font-size: 8px;
  pointer-events: none;
}

.risk-dot {
  fill: var(--color-status-warning);
  stroke: rgba(234, 179, 8, 0.3);
  stroke-width: 2;
}

.graph-legend {
  display: flex;
  gap: var(--space-3);
  margin-top: var(--space-2);
  padding-top: var(--space-2);
  border-top: 1px solid var(--color-border-default);
}

.legend-item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: 11px;
  color: var(--color-text-muted);
}

.legend-swatch {
  display: inline-block;
  width: 16px;
  height: 3px;
  border-radius: 2px;
}

.legend-branch {
  background: rgba(139, 92, 246, 0.5);
}

.legend-body {
  background: rgba(100, 116, 139, 0.4);
}

.legend-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.legend-risk-dot {
  background: var(--color-status-warning);
}

.legend-label {
  white-space: nowrap;
}
</style>
