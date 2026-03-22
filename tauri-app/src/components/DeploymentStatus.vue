<script setup lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener'
import { SBadge, SStatusDot } from '@stuntrocket/ui'
import type { Deployment } from '../types'

defineProps<{
  deployments: Deployment[]
  loading: boolean
}>()

/** Map deployment status to a human-readable label. */
function statusLabel(status: string): string {
  const labels: Record<string, string> = {
    success: 'Active',
    active: 'Active',
    pending: 'Pending',
    in_progress: 'Deploying',
    queued: 'Queued',
    failure: 'Failed',
    error: 'Error',
    inactive: 'Inactive',
    unknown: 'Unknown',
  }
  return labels[status] || status
}

/** Map deployment status to SBadge variant. */
function badgeVariant(status: string): 'success' | 'warning' | 'error' | 'default' {
  if (status === 'success' || status === 'active') return 'success'
  if (status === 'pending' || status === 'queued' || status === 'in_progress') return 'warning'
  if (status === 'failure' || status === 'error') return 'error'
  return 'default'
}

/** Map deployment status to SStatusDot variant. */
function dotVariant(status: string): 'success' | 'warning' | 'error' | 'neutral' {
  if (status === 'success' || status === 'active') return 'success'
  if (status === 'pending' || status === 'queued' || status === 'in_progress') return 'warning'
  if (status === 'failure' || status === 'error') return 'error'
  return 'neutral'
}

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-GB', {
    day: 'numeric', month: 'short',
    hour: '2-digit', minute: '2-digit',
  })
}

async function openDeploymentUrl(url: string | null) {
  if (url) await openUrl(url)
}
</script>

<template>
  <div v-if="loading" class="deployment-loading">
    <span class="loading-text">Checking deployments...</span>
  </div>
  <div v-else-if="deployments.length > 0" class="deployment-badges">
    <SBadge
      v-for="dep in deployments"
      :key="dep.environment"
      :variant="badgeVariant(dep.status)"
      :title="`${dep.environment}: ${statusLabel(dep.status)} — ${formatDate(dep.updated_at)}`"
      class="deployment-badge"
    >
      <SStatusDot :variant="dotVariant(dep.status)" />
      <span class="deployment-env">{{ dep.environment }}</span>
      <span class="deployment-state">{{ statusLabel(dep.status) }}</span>
      <button
        v-if="dep.url"
        class="deployment-link"
        title="Open deployment"
        @click.stop="openDeploymentUrl(dep.url)"
      >
        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
          <polyline points="15 3 21 3 21 9" />
          <line x1="10" y1="14" x2="21" y2="3" />
        </svg>
      </button>
    </SBadge>
  </div>
</template>

<style scoped>
.deployment-loading {
  display: inline-flex;
  align-items: center;
}

.loading-text {
  font-size: 11px;
  color: var(--color-text-muted);
}

.deployment-badges {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.deployment-badge {
  gap: 5px;
}

.deployment-env {
  font-weight: 600;
  text-transform: capitalize;
}

.deployment-state {
  opacity: 0.8;
  font-weight: 400;
}

.deployment-link {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  padding: 0;
  background: none;
  border: none;
  color: inherit;
  opacity: 0.6;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.deployment-link:hover {
  opacity: 1;
  background: rgba(255, 255, 255, 0.1);
}

.deployment-link:focus-visible {
  outline: 2px solid var(--color-border-focus);
  outline-offset: 1px;
}
</style>
