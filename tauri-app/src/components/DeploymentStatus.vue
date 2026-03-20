<script setup lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener'
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

/** Map environment name to a display colour class. */
function envClass(environment: string): string {
  const lower = environment.toLowerCase()
  if (lower.includes('prod')) return 'env-production'
  if (lower.includes('staging') || lower.includes('stage')) return 'env-staging'
  if (lower.includes('preview') || lower.includes('dev')) return 'env-preview'
  return 'env-default'
}

/** Map deployment status to a status colour class. */
function statusClass(status: string): string {
  if (status === 'success' || status === 'active') return 'status-success'
  if (status === 'pending' || status === 'queued' || status === 'in_progress') return 'status-pending'
  if (status === 'failure' || status === 'error') return 'status-failure'
  return 'status-neutral'
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
    <div
      v-for="dep in deployments"
      :key="dep.environment"
      class="deployment-badge"
      :class="[envClass(dep.environment), statusClass(dep.status)]"
      :title="`${dep.environment}: ${statusLabel(dep.status)} — ${formatDate(dep.updated_at)}`"
    >
      <span class="deployment-status-dot" :class="statusClass(dep.status)" />
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
    </div>
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
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 2px var(--space-2) 2px 6px;
  border-radius: var(--radius-full);
  font-size: 11px;
  font-weight: 500;
  border: 1px solid transparent;
  transition: all var(--transition-fast);
}

/* Environment colour schemes */
.deployment-badge.env-production {
  background: rgba(220, 38, 38, 0.12);
  border-color: rgba(220, 38, 38, 0.25);
  color: #f87171;
}

.deployment-badge.env-staging {
  background: rgba(234, 179, 8, 0.12);
  border-color: rgba(234, 179, 8, 0.25);
  color: #fbbf24;
}

.deployment-badge.env-preview {
  background: rgba(59, 130, 246, 0.12);
  border-color: rgba(59, 130, 246, 0.25);
  color: #60a5fa;
}

.deployment-badge.env-default {
  background: rgba(100, 116, 139, 0.12);
  border-color: rgba(100, 116, 139, 0.25);
  color: var(--color-text-muted);
}

/* Status indicator dot */
.deployment-status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.deployment-status-dot.status-success {
  background: var(--color-status-success);
  box-shadow: 0 0 4px rgba(34, 197, 94, 0.5);
}

.deployment-status-dot.status-pending {
  background: var(--color-status-warning);
  animation: pulse-dot 2s ease-in-out infinite;
}

.deployment-status-dot.status-failure {
  background: var(--color-status-danger);
}

.deployment-status-dot.status-neutral {
  background: var(--color-text-muted);
}

@keyframes pulse-dot {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 1; }
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
