<script setup lang="ts">
import { AlertTriangle } from 'lucide-vue-next'
import { SNoticeBanner } from '@stuntrocket/ui'
import type { SyncHealthStatus } from '../types'

defineProps<{
  unhealthyRepos: SyncHealthStatus[]
}>()
</script>

<template>
  <Transition name="sync-health-banner">
    <SNoticeBanner v-if="unhealthyRepos.length > 0" variant="danger">
      <div class="sync-health-content">
        <AlertTriangle :size="16" class="sync-health-icon" />
        <div class="sync-health-text">
          <span class="sync-health-title">
            Sync issues detected for {{ unhealthyRepos.length }}
            {{ unhealthyRepos.length === 1 ? 'repository' : 'repositories' }}
          </span>
          <div class="sync-health-repos">
            <span
              v-for="repo in unhealthyRepos.slice(0, 3)"
              :key="repo.repo_id"
              class="sync-health-repo"
            >
              <strong>{{ repo.repo_name }}</strong>
              — {{ repo.consecutive_failures }} consecutive
              {{ repo.consecutive_failures === 1 ? 'failure' : 'failures' }}
              <template v-if="repo.last_error">
                ({{ repo.last_error.slice(0, 80) }}{{ repo.last_error.length > 80 ? '…' : '' }})
              </template>
            </span>
            <span v-if="unhealthyRepos.length > 3" class="sync-health-more">
              and {{ unhealthyRepos.length - 3 }} more…
            </span>
          </div>
        </div>
      </div>
    </SNoticeBanner>
  </Transition>
</template>

<style scoped>
.sync-health-content {
  display: flex;
  align-items: flex-start;
  gap: var(--space-2);
  width: 100%;
}

.sync-health-icon {
  flex-shrink: 0;
  margin-top: 1px;
}

.sync-health-text {
  flex: 1;
  font-size: 12px;
}

.sync-health-title {
  font-weight: 600;
}

.sync-health-repos {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-top: 4px;
}

.sync-health-repo {
  color: var(--color-text-secondary);
  font-size: 11px;
}

.sync-health-more {
  color: var(--color-text-muted);
  font-size: 11px;
  font-style: italic;
}

.sync-health-banner-enter-active {
  transition: all 0.3s ease;
}

.sync-health-banner-leave-active {
  transition: all 0.2s ease;
}

.sync-health-banner-enter-from,
.sync-health-banner-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
