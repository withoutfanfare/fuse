<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { LayoutDashboard, GitPullRequest, FolderGit2, Settings, ChevronLeft, ChevronRight, Users, Clock, BarChart3, Bookmark, LayoutGrid } from 'lucide-vue-next'
import { SSidebar, SSidebarLink } from '@stuntrocket/ui'
import { usePullRequestsStore } from '../../stores/pullRequests'
import { useSidebarState } from '../../composables/useSidebarState'
import { useRecentPrs } from '../../composables/useRecentPrs'
import { useGlobalBookmarks } from '../../composables/useBookmarks'
const route = useRoute()
const prStore = usePullRequestsStore()
const { collapsed, toggle } = useSidebarState()
const { recentPrs } = useRecentPrs()
const { bookmarkCount, fetchBookmarkCount } = useGlobalBookmarks()

const pendingCount = computed(() => prStore.pendingReview.length)

const sidebarWidth = computed(() =>
  collapsed.value ? 'var(--sidebar-width-collapsed)' : 'var(--sidebar-width)'
)

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
  <SSidebar :width="sidebarWidth" class="app-sidebar" :class="{ 'app-sidebar--collapsed': collapsed }">
    <template #header>
      <!-- Navigation groups -->
      <nav class="sidebar-nav">
        <div v-for="(group, gi) in navGroups" :key="group.label" class="nav-group">
          <div v-if="!collapsed" class="nav-group-label">{{ group.label }}</div>
          <div v-else-if="gi > 0" class="nav-group-divider"></div>
          <SSidebarLink
            v-for="item in group.items"
            :key="item.path"
            :to="item.path"
            :active="route.path.startsWith(item.path)"
            :title="collapsed ? item.label : undefined"
          >
            <span class="nav-icon"><component :is="item.icon" :size="18" /></span>
            <span v-if="!collapsed" class="nav-label">{{ item.label }}</span>
            <span
              v-if="item.badge && pendingCount > 0"
              class="nav-badge"
              :class="{ 'nav-badge--small': collapsed }"
            >
              {{ pendingCount }}
            </span>
            <span
              v-if="item.bookmarkBadge && bookmarkCount > 0"
              class="nav-badge"
              :class="{ 'nav-badge--small': collapsed }"
            >
              {{ bookmarkCount }}
            </span>
          </SSidebarLink>
        </div>
      </nav>
    </template>

    <!-- Recently visited PRs -->
    <section v-if="recentPrs.length > 0" class="recent-prs">
      <div class="recent-prs-header">
        <Clock :size="14" class="recent-prs-icon" />
        <span v-if="!collapsed" class="recent-prs-label">Recent</span>
      </div>
      <SSidebarLink
        v-for="entry in recentPrs"
        :key="entry.id"
        :to="{ name: 'pr-detail', params: { id: entry.id } } as any"
        :title="collapsed ? `#${entry.number} ${entry.title}` : undefined"
      >
        <span class="recent-pr-number">#{{ entry.number }}</span>
        <span v-if="!collapsed" class="recent-pr-title">{{ entry.title }}</span>
      </SSidebarLink>
    </section>

    <template #footer>
      <button class="toggle-btn" :title="collapsed ? 'Expand sidebar' : 'Collapse sidebar'" @click="toggle">
        <component :is="collapsed ? ChevronRight : ChevronLeft" :size="16" />
      </button>
    </template>
  </SSidebar>
</template>

<style scoped>
.app-sidebar {
  flex-shrink: 0;
  z-index: 20;
  transition: width var(--transition-normal);
  margin: var(--space-2) 0 var(--space-2) var(--space-2);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-panel), 0 0 0 1px var(--color-surface-hover);
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.nav-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-0-5);
}

.nav-group-label {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--color-text-muted);
  padding: var(--space-1) var(--space-3);
}

.nav-group-divider {
  height: 1px;
  background: var(--color-border-default);
  margin: var(--space-1) var(--space-2);
}

.nav-icon {
  font-size: 16px;
  width: 20px;
  text-align: center;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.nav-label {
  flex: 1;
}

.nav-badge {
  background: var(--color-accent);
  color: white;
  font-size: 11px;
  font-weight: 700;
  min-width: 20px;
  height: 20px;
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 var(--space-1);
  transition: all var(--transition-fast);
}

.nav-badge--small {
  position: absolute;
  top: 2px;
  right: 2px;
  min-width: 16px;
  height: 16px;
  font-size: 9px;
  padding: 0 3px;
}

.app-sidebar--collapsed :deep(.s-sidebar-link) {
  position: relative;
  justify-content: center;
}

.toggle-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  background: none;
  border: 1px solid transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  transition: background var(--transition-fast), color var(--transition-fast), border-color var(--transition-fast);
  margin: 0 auto;
}

.toggle-btn:hover {
  background: var(--color-surface-hover);
  color: var(--color-text-primary);
  border-color: var(--color-border-default);
}

/* Recently visited PRs */
.recent-prs {
  display: flex;
  flex-direction: column;
  gap: var(--space-0-5);
  flex-shrink: 0;
}

.recent-prs-header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-1) var(--space-3);
  color: var(--color-text-muted);
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.app-sidebar--collapsed .recent-prs-header {
  justify-content: center;
  padding: var(--space-1) var(--space-2);
}

.recent-prs-icon {
  flex-shrink: 0;
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
}
</style>
