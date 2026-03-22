<script setup lang="ts">
import { computed, defineAsyncComponent, onMounted, onUnmounted, ref, shallowRef, provide, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { RefreshCw, Bell } from 'lucide-vue-next'
import { useRepositoriesStore } from './stores/repositories'
import { usePullRequestsStore } from './stores/pullRequests'
import { useNotificationCentreStore } from './stores/notificationCentre'
import { useKeyboardShortcuts, type PrListContext } from './composables/useKeyboardShortcuts'
import { useCommandPalette } from './composables/useCommandPalette'
import { useAutoSync } from './composables/useAutoSync'
import { useFocusMode } from './composables/useFocusMode'
import { useTheme } from './composables/useTheme'
import {
  SAmbientBlobs,
  STopbar,
  SRouteProgressBar,
  SCommandPalette,
  SButton,
  SIconButton,
  SStatusDot,
  SSpinner,
  SKbd,
  useRelativeTime,
} from '@stuntrocket/ui'
import AppSidebar from './components/layout/AppSidebar.vue'
import ToastContainer from './components/ToastContainer.vue'
const NotificationDrawer = defineAsyncComponent(() => import('./components/NotificationDrawer.vue'))
import ConfirmDialog from './components/ConfirmDialog.vue'
const ShortcutOverlay = defineAsyncComponent(() => import('./components/ShortcutOverlay.vue'))
const OnboardingWizard = defineAsyncComponent(() => import('./components/OnboardingWizard.vue'))
import { useOnboarding } from './composables/useOnboarding'

const router = useRouter()
const route = useRoute()
const repoStore = useRepositoriesStore()
const prStore = usePullRequestsStore()
const notifStore = useNotificationCentreStore()

/** PR list navigation context — set by PullRequests.vue, consumed by keyboard shortcuts */
const prListContext = shallowRef<PrListContext | null>(null)
provide('prListContext', prListContext)

const { showShortcutOverlay } = useKeyboardShortcuts(router, prStore, prListContext)
const commandPalette = useCommandPalette(router, prStore, repoStore)
const { isPolling } = useAutoSync()
const { focusActive } = useFocusMode()
/* Initialise theme (mode + accent) so it applies on app load */
useTheme()
const { showOnboarding } = useOnboarding()

/* Page title derived from route */
const pageTitle = computed(() => {
  const titles: Record<string, string> = {
    '/dashboard': 'Dashboard',
    '/aggregate': 'Aggregate Dashboard',
    '/prs': 'Pull Requests',
    '/repositories': 'Repositories',
    '/settings': 'Settings',
  }
  for (const [path, title] of Object.entries(titles)) {
    if (route.path.startsWith(path)) return title
  }
  return 'Fuse'
})

/* Relative time for last synced — uses library composable */
const lastSyncedDate = computed(() => prStore.lastSynced ? new Date(prStore.lastSynced) : null)
const { relative: lastSyncedRelative } = useRelativeTime(lastSyncedDate, 30_000)

/* Route progress bar — driven by router navigation guards */
const routeLoading = ref(false)
router.beforeEach(() => { routeLoading.value = true })
router.afterEach(() => { routeLoading.value = false })

function handleCmdK(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault()
    commandPalette.toggle()
  }
}

/** Shows a loading overlay while route transitions complete (e.g. tray PR clicks) */
const navigating = ref(false)
provide('dismissNavLoading', () => { navigating.value = false })

/* Safety: if navigating stays true for more than 3 s (e.g. component fails to mount), auto-dismiss */
let navSafetyTimer: ReturnType<typeof setTimeout> | null = null
watch(navigating, (val) => {
  if (navSafetyTimer) { clearTimeout(navSafetyTimer); navSafetyTimer = null }
  if (val) {
    navSafetyTimer = setTimeout(() => { navigating.value = false }, 3000)
  }
})

let unlistenNavPr: (() => void) | null = null
let unlistenMenuSync: (() => void) | null = null

onMounted(async () => {
  window.addEventListener('keydown', handleCmdK)
  // Only await repoStore (needed for sidebar); let PR sync run without blocking the shell.
  // syncAll fetches fresh PRs from GitHub and then refreshes the store + menu automatically.
  await repoStore.fetchAll()
  prStore.syncAll().then(() => invoke('refresh_menu'))
  prStore.fetchStats()

  // Listen for tray menu navigation events — show loading overlay immediately.
  // A unique query param forces Vue Router to treat every click as a fresh
  // navigation, even when clicking the same PR twice (Vue Router 4 silently
  // ignores duplicate navigations otherwise).
  unlistenNavPr = await listen<number>('menu-navigate-pr', (event) => {
    navigating.value = true
    router.push({
      name: 'pr-detail',
      params: { id: String(event.payload) },
      query: { t: Date.now() },
    })
  })

  unlistenMenuSync = await listen('menu-sync-requested', () => {
    prStore.syncAll()
  })
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleCmdK)
  unlistenNavPr?.()
  unlistenMenuSync?.()
})

function onCommandSelect(cmd: { action: () => void }) {
  commandPalette.close()
  cmd.action()
}

function onCommandSelectByIndex(index: number) {
  const cmd = commandPalette.commands.value[index]
  if (cmd) onCommandSelect(cmd)
}
</script>

<template>
  <div class="app-root">
    <SAmbientBlobs />

    <!-- Topbar — replaces both TitleBar and AppHeader -->
    <STopbar traffic-light-padding="78px">
      <template #left>
        <h1 class="topbar-title">{{ pageTitle }}</h1>
      </template>
      <template #right>
        <span v-if="isPolling" class="auto-sync-indicator">
          <SStatusDot variant="success" pulse />
          <span class="auto-sync-label">Auto-sync</span>
        </span>
        <span v-if="lastSyncedRelative" class="sync-status">
          Synced {{ lastSyncedRelative }}
        </span>
        <SIconButton
          variant="ghost"
          size="sm"
          tooltip="Notifications"
          @click="notifStore.toggleDrawer()"
        >
          <Bell :size="16" />
        </SIconButton>
        <span v-if="notifStore.unreadCount > 0" class="bell-badge">
          {{ notifStore.unreadCount > 9 ? '9+' : notifStore.unreadCount }}
        </span>
        <SButton
          variant="secondary"
          size="sm"
          :loading="prStore.syncing"
          @click="prStore.syncAll()"
        >
          <RefreshCw :size="14" />
          {{ prStore.syncing ? 'Syncing…' : 'Sync' }}
        </SButton>
      </template>
    </STopbar>

    <!-- App content — below fixed topbar -->
    <div class="app-layout">
      <AppSidebar />
      <main class="app-content">
        <SRouteProgressBar :loading="routeLoading" />
        <!-- Navigation loading overlay — shows immediately on tray PR clicks -->
        <Transition name="nav-loading">
          <div v-if="navigating" class="nav-loading-overlay">
            <SSpinner />
            <span class="nav-loading-text">Loading pull request…</span>
          </div>
        </Transition>
        <router-view :key="$route.path" />
      </main>
    </div>

    <!-- Focus mode: floating exit pill -->
    <Transition name="focus-pill-fade">
      <SButton
        v-if="focusActive"
        variant="secondary"
        size="sm"
        class="focus-exit-pill"
        @click="focusActive = false"
      >
        Exit Focus
        <SKbd>⌘⇧F</SKbd>
      </SButton>
    </Transition>

    <OnboardingWizard v-if="showOnboarding" />
    <ToastContainer />
    <NotificationDrawer />
    <ConfirmDialog />
    <ShortcutOverlay
      :visible="showShortcutOverlay"
      @close="showShortcutOverlay = false"
    />
    <SCommandPalette
      :open="commandPalette.isOpen.value"
      :result-count="commandPalette.commands.value.length"
      placeholder="Search commands, PRs, pages..."
      @update:query="commandPalette.searchQuery.value = $event"
      @close="commandPalette.close()"
      @select="onCommandSelectByIndex"
    >
      <template #default="{ selectedIndex }">
        <div
          v-for="(cmd, i) in commandPalette.commands.value"
          :key="cmd.id"
          :data-index="i"
          class="palette-item"
          :class="{ 'palette-item--active': i === selectedIndex }"
          @click="onCommandSelect(cmd)"
        >
          <span class="palette-item-icon">{{ cmd.icon }}</span>
          <div class="palette-item-text">
            <span class="palette-item-title">{{ cmd.title }}</span>
            <span class="palette-item-subtitle">{{ cmd.subtitle }}</span>
          </div>
        </div>
        <div v-if="commandPalette.commands.value.length === 0" class="palette-empty">
          No results found
        </div>
      </template>
    </SCommandPalette>
  </div>
</template>

<style scoped>
.app-root {
  height: 100vh;
  position: relative;
}

.app-layout {
  display: flex;
  height: 100%;
  padding-top: 52px; /* Below fixed STopbar */
  position: relative;
  z-index: 10;
}

.app-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-4) var(--space-5);
  position: relative;
  min-width: 0;
}

/* Topbar content */
.topbar-title {
  font-size: 15px;
  font-weight: 600;
  letter-spacing: -0.02em;
  line-height: 1.2;
}

.auto-sync-indicator {
  display: flex;
  align-items: center;
  gap: var(--space-1-5);
  font-size: 12px;
  color: var(--color-status-success);
  font-weight: 500;
}

.auto-sync-label {
  display: none;
}

@media (min-width: 768px) {
  .auto-sync-label {
    display: inline;
  }
}

.sync-status {
  font-size: 12px;
  color: var(--color-text-secondary, var(--color-text-muted));
}

/* Notification badge — positioned relative to bell button */
.bell-badge {
  position: relative;
  margin-left: -12px;
  margin-right: 4px;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  background: var(--color-status-danger);
  color: white;
  font-size: 9px;
  font-weight: 700;
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
  z-index: 1;
}

/* Focus exit pill */
.focus-exit-pill {
  position: fixed !important;
  bottom: var(--space-6);
  left: 50%;
  transform: translateX(-50%);
  z-index: 30;
}

/* Transition for the exit pill */
.focus-pill-fade-enter-active {
  transition: all 0.3s ease-out;
}

.focus-pill-fade-leave-active {
  transition: all 0.2s ease-in;
}

.focus-pill-fade-enter-from,
.focus-pill-fade-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(12px);
}

/* Navigation loading overlay */
.nav-loading-overlay {
  position: absolute;
  inset: 0;
  z-index: 40;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-4);
  background: var(--color-surface, var(--color-surface-base));
}

.nav-loading-text {
  font-size: 13px;
  color: var(--color-text-secondary, var(--color-text-muted));
  font-weight: 500;
}

.nav-loading-enter-active {
  transition: opacity 0.15s ease-out;
}

.nav-loading-leave-active {
  transition: opacity 0.2s ease-in;
}

.nav-loading-enter-from,
.nav-loading-leave-to {
  opacity: 0;
}

/* Command palette result items (rendered inside SCommandPalette slot) */
.palette-item {
  display: flex;
  align-items: center;
  gap: var(--space-3, 12px);
  padding: var(--space-2, 8px) var(--space-3, 12px);
  border-radius: var(--radius-md, 6px);
  cursor: pointer;
  transition: background var(--duration-instant, 100ms) ease;
}

.palette-item:hover,
.palette-item--active {
  background: var(--color-surface-hover, rgba(255, 255, 255, 0.06));
}

.palette-item--active {
  background: var(--color-accent-subtle, rgba(20, 184, 166, 0.2));
}

.palette-item-icon {
  font-size: 16px;
  flex-shrink: 0;
  width: 24px;
  text-align: center;
}

.palette-item-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.palette-item-title {
  font-size: 14px;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.palette-item-subtitle {
  font-size: 12px;
  color: var(--color-text-secondary, rgba(255, 255, 255, 0.5));
}

.palette-empty {
  padding: var(--space-6, 24px);
  text-align: center;
  color: var(--color-text-secondary, rgba(255, 255, 255, 0.5));
  font-size: 13px;
}
</style>
