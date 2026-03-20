import { ref, onMounted, onUnmounted } from 'vue'
import type { Ref, ShallowRef } from 'vue'
import type { Router } from 'vue-router'
import type { usePullRequestsStore } from '../stores/pullRequests'

/**
 * PR list navigation context — optionally provided by the PullRequests view
 * so j/k/Enter/x shortcuts can manipulate the focused row.
 */
export interface PrListContext {
  focusedIndex: Ref<number>
  listLength: () => number
  openDetail: (id: number) => void
  getIdAtIndex: (index: number) => number | undefined
  toggleSelection: (id: number) => void
}

/**
 * PR detail review context — optionally provided by PullRequestDetail view
 * so review workflow shortcuts can operate on the current PR.
 */
export interface ReviewContext {
  /** Navigate to the next file in the diff */
  nextFile: () => void
  /** Navigate to the previous file in the diff */
  prevFile: () => void
  /** Toggle the focused checklist item */
  toggleChecklistItem: () => void
  /** Jump to the next annotation/bookmark */
  nextAnnotation: () => void
  /** Mark current PR review as complete and advance to next queued PR */
  completeAndAdvance: () => void
  /** Switch to the next tab */
  nextTab: () => void
  /** Switch to the previous tab */
  prevTab: () => void
}

export function useKeyboardShortcuts(
  router: Router,
  prStore: ReturnType<typeof usePullRequestsStore>,
  prListContext?: Ref<PrListContext | null> | ShallowRef<PrListContext | null>,
  reviewContext?: Ref<ReviewContext | null> | ShallowRef<ReviewContext | null>,
) {
  const showShortcutOverlay = ref(false)

  function isInputFocused(): boolean {
    const el = document.activeElement
    if (!el) return false
    const tag = el.tagName.toLowerCase()
    return tag === 'input' || tag === 'textarea' || tag === 'select' || (el as HTMLElement).isContentEditable
  }

  /** Whether the PR list view is currently active */
  function isPrListRoute(): boolean {
    return router.currentRoute.value.name === 'pull-requests'
  }

  /** Whether the PR detail view is currently active */
  function isPrDetailRoute(): boolean {
    return router.currentRoute.value.name === 'pr-detail'
  }

  function handleKeydown(e: KeyboardEvent) {
    // Cmd/Ctrl+K is handled in App.vue for the command palette — skip here
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') return

    // '/' should focus search even if nothing is focused
    if (e.key === '/') {
      if (isInputFocused()) return
      e.preventDefault()
      const searchInput = document.querySelector('.search-input') as HTMLInputElement | null
      if (searchInput) searchInput.focus()
      return
    }

    // Escape works regardless of focus
    if (e.key === 'Escape') {
      showShortcutOverlay.value = false
      const active = document.activeElement as HTMLElement | null
      if (active) active.blur()
      return
    }

    // All other shortcuts are blocked when an input is focused
    if (isInputFocused()) return

    // PR detail review workflow shortcuts
    const rctx = reviewContext?.value
    if (rctx && isPrDetailRoute()) {
      // Bracket keys for file navigation
      if (e.key === ']' || (e.key === 'n' && !e.metaKey && !e.ctrlKey)) {
        e.preventDefault()
        rctx.nextFile()
        return
      }
      if (e.key === '[' || (e.key === 'p' && !e.metaKey && !e.ctrlKey)) {
        e.preventDefault()
        rctx.prevFile()
        return
      }
      // 'c' to toggle checklist item
      if (e.key === 'c') {
        e.preventDefault()
        rctx.toggleChecklistItem()
        return
      }
      // 'a' to jump to next annotation
      if (e.key === 'a') {
        e.preventDefault()
        rctx.nextAnnotation()
        return
      }
      // Shift+Enter to complete review and advance
      if (e.key === 'Enter' && e.shiftKey) {
        e.preventDefault()
        rctx.completeAndAdvance()
        return
      }
      // Tab navigation: Ctrl+] / Ctrl+[ for next/prev tab
      if (e.key === ']' && e.ctrlKey) {
        e.preventDefault()
        rctx.nextTab()
        return
      }
      if (e.key === '[' && e.ctrlKey) {
        e.preventDefault()
        rctx.prevTab()
        return
      }
    }

    // PR list keyboard navigation — only active on the PR list route
    const ctx = prListContext?.value
    if (ctx && isPrListRoute()) {
      const len = ctx.listLength()
      switch (e.key) {
        case 'j': {
          e.preventDefault()
          if (len === 0) break
          ctx.focusedIndex.value = Math.min(ctx.focusedIndex.value + 1, len - 1)
          return
        }
        case 'k': {
          e.preventDefault()
          if (len === 0) break
          ctx.focusedIndex.value = Math.max(ctx.focusedIndex.value - 1, 0)
          return
        }
        case 'Enter': {
          if (ctx.focusedIndex.value >= 0 && ctx.focusedIndex.value < len) {
            e.preventDefault()
            const id = ctx.getIdAtIndex(ctx.focusedIndex.value)
            if (id !== undefined) ctx.openDetail(id)
          }
          return
        }
        case 'x': {
          if (ctx.focusedIndex.value >= 0 && ctx.focusedIndex.value < len) {
            e.preventDefault()
            const id = ctx.getIdAtIndex(ctx.focusedIndex.value)
            if (id !== undefined) ctx.toggleSelection(id)
          }
          return
        }
      }
    }

    switch (e.key) {
      case '1':
        router.push('/dashboard')
        break
      case '2':
        router.push('/prs')
        break
      case '3':
        router.push('/repositories')
        break
      case '4':
        router.push('/settings')
        break
      case '5':
        router.push('/aggregate')
        break
      case 'r':
        prStore.syncAll()
        break
      case '?':
        showShortcutOverlay.value = !showShortcutOverlay.value
        break
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', handleKeydown)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeydown)
  })

  return {
    showShortcutOverlay,
  }
}
