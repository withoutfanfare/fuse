import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Bookmark, BookmarkWithContext, BookmarkCategory } from '../types'

/**
 * Composable for managing file-level annotation bookmarks within a PR.
 * Provides CRUD operations for private review annotations.
 */
export function useBookmarks() {
  const bookmarks = ref<Bookmark[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchBookmarks(prId: number) {
    loading.value = true
    error.value = null
    try {
      bookmarks.value = await invoke<Bookmark[]>('list_bookmarks', { prId })
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function addBookmark(
    prId: number,
    filePath: string,
    note: string,
    lineStart?: number | null,
    lineEnd?: number | null,
    category?: BookmarkCategory,
  ): Promise<Bookmark | null> {
    error.value = null
    try {
      const bookmark = await invoke<Bookmark>('create_bookmark', {
        prId,
        filePath,
        lineStart: lineStart ?? null,
        lineEnd: lineEnd ?? null,
        note,
        category: category ?? null,
      })
      bookmarks.value.push(bookmark)
      return bookmark
    } catch (e) {
      error.value = String(e)
      return null
    }
  }

  async function updateBookmark(
    id: number,
    note: string,
    lineStart?: number | null,
    lineEnd?: number | null,
    category?: BookmarkCategory | null,
    resolved?: boolean | null,
  ): Promise<Bookmark | null> {
    error.value = null
    try {
      const updated = await invoke<Bookmark>('update_bookmark', {
        id,
        note,
        lineStart: lineStart ?? null,
        lineEnd: lineEnd ?? null,
        category: category ?? null,
        resolved: resolved ?? null,
      })
      const idx = bookmarks.value.findIndex(b => b.id === id)
      if (idx !== -1) bookmarks.value[idx] = updated
      return updated
    } catch (e) {
      error.value = String(e)
      return null
    }
  }

  async function toggleResolved(id: number): Promise<Bookmark | null> {
    error.value = null
    try {
      const updated = await invoke<Bookmark>('toggle_bookmark_resolved', { id })
      const idx = bookmarks.value.findIndex(b => b.id === id)
      if (idx !== -1) bookmarks.value[idx] = updated
      return updated
    } catch (e) {
      error.value = String(e)
      return null
    }
  }

  async function removeBookmark(id: number): Promise<boolean> {
    error.value = null
    try {
      await invoke('delete_bookmark', { id })
      bookmarks.value = bookmarks.value.filter(b => b.id !== id)
      return true
    } catch (e) {
      error.value = String(e)
      return false
    }
  }

  return {
    bookmarks,
    loading,
    error,
    fetchBookmarks,
    addBookmark,
    updateBookmark,
    toggleResolved,
    removeBookmark,
  }
}

/**
 * Composable for global bookmark operations across all PRs.
 */
export function useGlobalBookmarks() {
  const allBookmarks = ref<BookmarkWithContext[]>([])
  const bookmarkCount = ref(0)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchAllBookmarks() {
    loading.value = true
    error.value = null
    try {
      allBookmarks.value = await invoke<BookmarkWithContext[]>('list_all_bookmarks')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function fetchBookmarkCount() {
    try {
      bookmarkCount.value = await invoke<number>('get_bookmark_count')
    } catch (e) {
      // Silently fail for badge count
    }
  }

  async function toggleResolved(id: number): Promise<Bookmark | null> {
    error.value = null
    try {
      const updated = await invoke<Bookmark>('toggle_bookmark_resolved', { id })
      const idx = allBookmarks.value.findIndex(b => b.id === id)
      if (idx !== -1) {
        allBookmarks.value[idx] = { ...allBookmarks.value[idx], ...updated }
      }
      return updated
    } catch (e) {
      error.value = String(e)
      return null
    }
  }

  async function removeBookmark(id: number): Promise<boolean> {
    error.value = null
    try {
      await invoke('delete_bookmark', { id })
      allBookmarks.value = allBookmarks.value.filter(b => b.id !== id)
      bookmarkCount.value = Math.max(0, bookmarkCount.value - 1)
      return true
    } catch (e) {
      error.value = String(e)
      return false
    }
  }

  return {
    allBookmarks,
    bookmarkCount,
    loading,
    error,
    fetchAllBookmarks,
    fetchBookmarkCount,
    toggleResolved,
    removeBookmark,
  }
}
