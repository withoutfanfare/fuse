<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { LayoutDashboard, GitPullRequest, FolderGit2, Settings, ChevronLeft, ChevronRight, Users, Clock, BarChart3, Bookmark, LayoutGrid } from 'lucide-vue-next'
import { SSidebar, SSidebarLink, SIconButton, SBadge, SDivider, useSidebarCollapse } from '@stuntrocket/ui'
import { usePullRequestsStore } from '../../stores/pullRequests'
import { useRecentPrs } from '../../composables/useRecentPrs'
import { useGlobalBookmarks } from '../../composables/useBookmarks'

const route = useRoute()
const prStore = usePullRequestsStore()
const { collapsed, toggle } = useSidebarCollapse('sidebar-collapsed')
const { recentPrs } = useRecentPrs()
const { bookmarkCount, fetchBookmarkCount } = useGlobalBookmarks()

const pendingCount = computed(() => prStore.pendingReview.length)

const sidebarWidth = computed(() => collapsed.value ? '56px' : '224px')

onMounted(() => {
  fetchBookmarkCount()
})

const navGroups = [
  {
    label: 'Review',
    items: [
      { path: '/dashboard', label: 'Dashboard', icon: LayoutDashboard },
      { path: '/aggregate', label: 'Aggregate', icon: LayoutGrid },
      { path: '/prs', label: 'Pull Requests', icon: GitPullRequest, badge: true },
      { path: '/bookmarks', label: 'Bookmarks', icon: Bookmark, bookmarkBadge: true },
    ],
  },
  {
    label: 'Insights',
    items: [
      { path: '/digest', label: 'Digest', icon: BarChart3 },
      { path: '/authors', label: 'Authors', icon: Users },
    ],
  },
  {
    label: 'Manage',
    items: [
      { path: '/repositories', label: 'Repositories', icon: FolderGit2 },
      { path: '/settings', label: 'Settings', icon: Settings },
    ],
  },
]
</script>

<template>
  <SSidebar :width="sidebarWidth" class="sidebar-shell">
    <template #header>
      <div v-for="(group, gi) in navGroups" :key="group.label" class="nav-group">
        <div v-if="!collapsed" class="nav-group-label">{{ group.label }}</div>
        <SDivider v-else-if="gi > 0" class="my-1" />
        <SSidebarLink
          v-for="item in group.items"
          :key="item.path"
          :to="item.path"
          :active="route.path.startsWith(item.path)"
        >
          <div class="sidebar-link-content" :class="{ 'sidebar-link-content--collapsed': collapsed }">
            <component :is="item.icon" :size="18" class="sidebar-link-icon" />
            <span v-if="!collapsed" class="sidebar-link-label">{{ item.label }}</span>
            <SBadge
              v-if="item.badge && pendingCount > 0 && !collapsed"
              variant="count"
            >
              {{ pendingCount }}
            </SBadge>
            <SBadge
              v-if="item.bookmarkBadge && bookmarkCount > 0 && !collapsed"
              variant="count"
            >
              {{ bookmarkCount }}
            </SBadge>
          </div>
        </SSidebarLink>
      </div>
    </template>

    <!-- Recent PRs -->
    <template #default>
      <section v-if="recentPrs.length > 0" class="recent-prs">
        <div class="recent-prs-header">
          <Clock :size="14" class="shrink-0 text-[var(--color-text-secondary)]" />
          <span v-if="!collapsed" class="recent-prs-label">Recent</span>
        </div>
        <SSidebarLink
          v-for="entry in recentPrs"
          :key="entry.id"
          :to="{ name: 'pr-detail', params: { id: entry.id } } as any"
          :active="false"
        >
          <div class="sidebar-link-content" :class="{ 'sidebar-link-content--collapsed': collapsed }">
            <span class="recent-pr-number">#{{ entry.number }}</span>
            <span v-if="!collapsed" class="recent-pr-title">{{ entry.title }}</span>
          </div>
        </SSidebarLink>
      </section>
    </template>

    <template #footer>
      <div class="sidebar-footer">
        <SIconButton
          variant="ghost"
          size="sm"
          :tooltip="collapsed ? 'Expand sidebar' : 'Collapse sidebar'"
          @click="toggle"
        >
          <component :is="collapsed ? ChevronRight : ChevronLeft" :size="16" />
        </SIconButton>
      </div>
    </template>
  </SSidebar>
</template>

<style scoped>
.sidebar-shell {
  margin: var(--space-2) 0 var(--space-2) var(--space-2);
  border-radius: var(--radius-lg);
  transition: width var(--duration-normal, 250ms) cubic-bezier(0.4, 0, 0.2, 1);
}

.nav-group {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.nav-group + .nav-group {
  margin-top: var(--space-3);
}

.nav-group-label {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--color-text-tertiary, var(--color-text-muted));
  padding: var(--space-1) var(--space-1);
  margin-bottom: 2px;
}

.sidebar-link-content {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  width: 100%;
  min-width: 0;
}

.sidebar-link-content--collapsed {
  justify-content: center;
  gap: 0;
}

.sidebar-link-icon {
  flex-shrink: 0;
}

.sidebar-link-label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Recent PRs section */
.recent-prs {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.recent-prs-header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-1);
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--color-text-tertiary, var(--color-text-muted));
  margin-bottom: 2px;
}

.recent-pr-number {
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 600;
  color: var(--color-accent);
  flex-shrink: 0;
}

.recent-pr-title {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
  font-size: 12px;
}

.sidebar-footer {
  display: flex;
  justify-content: center;
}
</style>
