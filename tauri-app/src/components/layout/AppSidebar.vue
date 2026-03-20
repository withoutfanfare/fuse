<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { LayoutDashboard, GitPullRequest, FolderGit2, Settings, ChevronLeft, ChevronRight, Users, Clock, BarChart3, Bookmark, LayoutGrid } from 'lucide-vue-next'
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
  <aside class="sidebar" :class="{ 'sidebar--collapsed': collapsed }">
    <nav class="sidebar-nav">
      <div v-for="(group, gi) in navGroups" :key="group.label" class="nav-group">
        <div v-if="!collapsed" class="nav-group-label">{{ group.label }}</div>
        <div v-else-if="gi > 0" class="nav-group-divider"></div>
        <router-link
          v-for="item in group.items"
          :key="item.path"
          :to="item.path"
          class="nav-item"
          :class="{ active: route.path.startsWith(item.path) }"
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
        </router-link>
      </div>
    </nav>

    <!-- Recently visited PRs -->
    <section v-if="recentPrs.length > 0" class="recent-prs">
      <div class="recent-prs-header">
        <Clock :size="14" class="recent-prs-icon" />
        <span v-if="!collapsed" class="recent-prs-label">Recent</span>
      </div>
      <router-link
        v-for="entry in recentPrs"
        :key="entry.id"
        :to="{ name: 'pr-detail', params: { id: entry.id } }"
        class="recent-pr-item"
        :title="collapsed ? `#${entry.number} ${entry.title}` : undefined"
      >
        <span class="recent-pr-number">#{{ entry.number }}</span>
        <span v-if="!collapsed" class="recent-pr-title">{{ entry.title }}</span>
      </router-link>
    </section>

    <div class="sidebar-footer">
      <button class="toggle-btn" :title="collapsed ? 'Expand sidebar' : 'Collapse sidebar'" @click="toggle">
        <component :is="collapsed ? ChevronRight : ChevronLeft" :size="16" />
      </button>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-width);
  background: var(--color-surface-chrome);
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  box-shadow: var(--shadow-panel), 0 0 0 1px var(--color-surface-hover);
  z-index: 20;
  transition: width var(--transition-normal);
  overflow: hidden;
  margin: var(--space-2) 0 var(--space-2) var(--space-2);
}

.sidebar--collapsed {
  width: var(--sidebar-width-collapsed);
}

.sidebar-nav {
  padding: var(--space-3) var(--space-2);
  flex: 1;
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

.nav-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  text-decoration: none;
  transition: background var(--transition-fast), color var(--transition-fast);
  font-size: 14px;
  white-space: nowrap;
}

.sidebar--collapsed .nav-item {
  justify-content: center;
  padding: var(--space-2);
  gap: 0;
}

.nav-item:hover {
  background: var(--color-surface-hover);
  color: var(--color-text-primary);
  text-decoration: none;
}

.nav-item.active {
  color: var(--color-accent);
  font-weight: 500;
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

.sidebar--collapsed .nav-item {
  position: relative;
}

.sidebar-footer {
  padding: var(--space-2);
  display: flex;
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
}

.toggle-btn:hover {
  background: var(--color-surface-hover);
  color: var(--color-text-primary);
  border-color: var(--color-border-default);
}

/* Recently visited PRs */
.recent-prs {
  padding: var(--space-2) var(--space-2);
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

.sidebar--collapsed .recent-prs-header {
  justify-content: center;
  padding: var(--space-1) var(--space-2);
}

.recent-prs-icon {
  flex-shrink: 0;
}

.recent-pr-item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-1) var(--space-3);
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  text-decoration: none;
  font-size: 12px;
  transition: background var(--transition-fast), color var(--transition-fast);
  overflow: hidden;
  white-space: nowrap;
}

.sidebar--collapsed .recent-pr-item {
  justify-content: center;
  padding: var(--space-1) var(--space-2);
}

.recent-pr-item:hover {
  background: var(--color-surface-hover);
  color: var(--color-text-primary);
  text-decoration: none;
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
