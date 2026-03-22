<script setup lang="ts">
import { defineAsyncComponent, onMounted, onUnmounted, ref, shallowRef, provide, watch } from 'vue'
import { useRouter } from 'vue-router'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { useRepositoriesStore } from './stores/repositories'
import { usePullRequestsStore } from './stores/pullRequests'
import { useKeyboardShortcuts, type PrListContext } from './composables/useKeyboardShortcuts'
import { useCommandPalette } from './composables/useCommandPalette'
import { useAutoSync } from './composables/useAutoSync'
import { useFocusMode } from './composables/useFocusMode'
import { useTheme } from './composables/useTheme'
import { SRouteProgressBar, SCommandPalette } from '@stuntrocket/ui'
import AppSidebar from './components/layout/AppSidebar.vue'
import AppHeader from './components/layout/AppHeader.vue'
import ToastContainer from './components/ToastContainer.vue'
const NotificationDrawer = defineAsyncComponent(() => import('./components/NotificationDrawer.vue'))
import ConfirmDialog from './components/ConfirmDialog.vue'
const ShortcutOverlay = defineAsyncComponent(() => import('./components/ShortcutOverlay.vue'))
const OnboardingWizard = defineAsyncComponent(() => import('./components/OnboardingWizard.vue'))
import { useOnboarding } from './composables/useOnboarding'

const router = useRouter()
const repoStore = useRepositoriesStore()
const prStore = usePullRequestsStore()

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
    <!--
      AppHeader now uses STopbar which provides:
      - Fixed position at top (52px height)
      - macOS traffic light padding (drag region)
      This replaces the old custom TitleBar entirely.
    -->
    <AppHeader
      :syncing="prStore.syncing"
      :last-synced="prStore.lastSynced"
      :auto-sync-active="isPolling"
      @sync-requested="prStore.syncAll()"
    />

    <!--
      App layout: sidebar + main.
      SAppShell was considered but not used here because AppSidebar already
      wraps its content in SSidebar, which provides its own container styling.
      Using SAppShell would create a redundant double-wrapper.
    -->
    <div class="app-layout">
      <AppSidebar />
      <div class="app-main">
        <main class="app-content">
          <SRouteProgressBar :loading="routeLoading" />
          <!-- Navigation loading overlay — shows immediately on tray PR clicks -->
          <Transition name="nav-loading">
            <div v-if="navigating" class="nav-loading-overlay">
              <div class="nav-loading-spinner" />
              <span class="nav-loading-text">Loading pull request…</span>
            </div>
          </Transition>
          <router-view :key="$route.path" />
        </main>
      </div>
    </div>

    <!-- Focus mode: floating exit pill -->
    <Transition name="focus-pill-fade">
      <button
        v-if="focusActive"
        class="focus-exit-pill"
        @click="focusActive = false"
      >
        Exit Focus
        <kbd class="focus-exit-kbd">⌘⇧F</kbd>
      </button>
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
  height: calc(100% - 52px);
  margin-top: 52px;
  position: relative;
  z-index: 10;
}

.app-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

.app-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-6);
  position: relative;
}

/* Focus exit pill — fixed floating button */
.focus-exit-pill {
  position: fixed;
  bottom: var(--space-6);
  left: 50%;
  transform: translateX(-50%);
  z-index: 30;
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-5);
  background: var(--color-surface-panel);
  backdrop-filter: blur(24px) saturate(1.4);
  -webkit-backdrop-filter: blur(24px) saturate(1.4);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-full);
  box-shadow: var(--shadow-overlay);
  color: var(--color-text-primary);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.focus-exit-pill:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-border-hover);
}

.focus-exit-kbd {
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--color-text-muted);
  background: rgba(255, 255, 255, 0.06);
  padding: 1px var(--space-1-5);
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-border-default);
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
  background: var(--color-surface-base);
}

.nav-loading-spinner {
  width: 28px;
  height: 28px;
  border: 2.5px solid var(--color-border-default);
  border-top-color: var(--color-accent);
  border-radius: 50%;
  animation: nav-spin 0.7s linear infinite;
}

.nav-loading-text {
  font-size: 13px;
  color: var(--color-text-muted);
  font-weight: 500;
}

@keyframes nav-spin {
  to { transform: rotate(360deg); }
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
  transition: background 0.1s ease;
}

.palette-item:hover,
.palette-item--active {
  background: var(--color-surface-hover, rgba(255, 255, 255, 0.06));
}

.palette-item--active {
  background: var(--color-accent-muted, rgba(20, 184, 166, 0.2));
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
  color: var(--color-text-muted, rgba(255, 255, 255, 0.5));
}

.palette-empty {
  padding: var(--space-6, 24px);
  text-align: center;
  color: var(--color-text-muted, rgba(255, 255, 255, 0.5));
  font-size: 13px;
}
</style>
