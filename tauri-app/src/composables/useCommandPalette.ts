import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Router } from 'vue-router'
import type { usePullRequestsStore } from '../stores/pullRequests'
import type { useRepositoriesStore } from '../stores/repositories'
import type { BookmarkWithContext } from '../types'

export interface PaletteCommand {
  id: string
  icon: string
  title: string
  subtitle: string
  action: () => void
}

export function useCommandPalette(
  router: Router,
  prStore: ReturnType<typeof usePullRequestsStore>,
  repoStore: ReturnType<typeof useRepositoriesStore>,
) {
  const isOpen = ref(false)
  const searchQuery = ref('')
  const bookmarkCommands = ref<PaletteCommand[]>([])

  async function loadBookmarkCommands() {
    try {
      const bookmarks = await invoke<BookmarkWithContext[]>('list_all_bookmarks')
      bookmarkCommands.value = bookmarks.map(b => {
        const lineRef = b.line_start != null ? `:L${b.line_start}` : ''
        const notePreview = b.note ? ` \u2014 ${b.note.slice(0, 50)}` : ''
        return {
          id: `bookmark-${b.id}`,
          icon: '\uD83D\uDCCC',
          title: `${b.file_path}${lineRef}${notePreview}`,
          subtitle: `Bookmark \u00B7 PR #${b.pr_number}`,
          action: () => router.push({
            name: 'pr-detail',
            params: { id: b.pr_id },
            query: {
              bookmarkFile: b.file_path,
              bookmarkLine: b.line_start?.toString() ?? '',
            },
          }),
        }
      })
    } catch {
      bookmarkCommands.value = []
    }
  }

  function open() {
    isOpen.value = true
    searchQuery.value = ''
    // Lazily load bookmark commands when palette opens (Phase 5.6)
    loadBookmarkCommands()
  }

  function close() {
    isOpen.value = false
    searchQuery.value = ''
  }

  function toggle() {
    if (isOpen.value) {
      close()
    } else {
      open()
    }
  }

  /** Static navigation and action commands — always available. */
  const staticCommands: PaletteCommand[] = [
    { id: 'nav-dashboard', icon: '📊', title: 'Dashboard', subtitle: 'Navigation', action: () => router.push('/dashboard') },
    { id: 'nav-prs', icon: '🔀', title: 'Pull Requests', subtitle: 'Navigation', action: () => router.push('/prs') },
    { id: 'nav-bookmarks', icon: '📌', title: 'Bookmarks', subtitle: 'Navigation', action: () => router.push('/bookmarks') },
    { id: 'nav-repos', icon: '📁', title: 'Repositories', subtitle: 'Navigation', action: () => router.push('/repositories') },
    { id: 'nav-settings', icon: '⚙️', title: 'Settings', subtitle: 'Navigation', action: () => router.push('/settings') },
    { id: 'nav-aggregate', icon: '📋', title: 'Aggregate Dashboard', subtitle: 'Navigation', action: () => router.push('/aggregate') },
    { id: 'action-sync', icon: '🔄', title: 'Sync all repositories', subtitle: 'Action', action: () => prStore.syncAll() },
    { id: 'action-sync-delta', icon: '⚡', title: 'Incremental sync (delta)', subtitle: 'Action', action: () => prStore.syncIncremental() },
    { id: 'action-refresh', icon: '♻️', title: 'Refresh pull requests', subtitle: 'Action', action: () => prStore.fetchAll() },
  ]

  /** Full command list — only builds PR entries when the palette is open. */
  const allCommands = computed<PaletteCommand[]>(() => {
    if (!isOpen.value) return staticCommands

    const cmds: PaletteCommand[] = [...staticCommands]

    // PR entries — only built when the palette is visible
    for (const pr of prStore.prs) {
      const repo = repoStore.repos.find(r => r.id === pr.repo_id)
      const repoLabel = repo ? `${repo.owner}/${repo.name}` : ''
      cmds.push({
        id: `pr-${pr.id}`,
        icon: pr.merged_at ? '🟣' : pr.closed_at ? '🔴' : '🟢',
        title: `#${pr.number} ${pr.title}`,
        subtitle: [repoLabel, pr.author].filter(Boolean).join(' · '),
        action: () => router.push({ name: 'pr-detail', params: { id: pr.id } }),
      })
    }

    // Bookmark entries (Phase 5.6) — only built when palette is open
    cmds.push(...bookmarkCommands.value)

    return cmds
  })

  const commands = computed<PaletteCommand[]>(() => {
    const q = searchQuery.value.toLowerCase().trim()
    if (!q) return allCommands.value.slice(0, 50)

    const scored = allCommands.value
      .map(cmd => {
        const haystack = `${cmd.title} ${cmd.subtitle}`.toLowerCase()
        let score = 0
        if (haystack.includes(q)) {
          score = 1
          // Boost exact start-of-word matches
          if (haystack.startsWith(q)) score = 3
          else if (haystack.includes(` ${q}`)) score = 2
        }
        return { cmd, score }
      })
      .filter(item => item.score > 0)
      .sort((a, b) => b.score - a.score)

    return scored.map(s => s.cmd).slice(0, 50)
  })

  return {
    isOpen,
    searchQuery,
    commands,
    open,
    close,
    toggle,
  }
}
