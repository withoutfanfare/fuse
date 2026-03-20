import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ReviewSession } from '../types'

/**
 * Composable for managing a focused review session on a single PR.
 *
 * Handles session lifecycle (create/resume/pause/complete),
 * file review tracking, notes, and a Pomodoro-style timer.
 */
export function useReviewSession(prId: number) {
  const session = ref<ReviewSession | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Pomodoro timer state
  const timerElapsed = ref(0)
  const timerRunning = ref(false)
  const pomodoroMinutes = ref(25)
  let intervalId: ReturnType<typeof setInterval> | null = null
  let isMounted = false

  /** Formatted timer display (MM:SS countdown or count-up). */
  const timerDisplay = computed(() => {
    const remaining = pomodoroMinutes.value * 60 - timerElapsed.value
    const displaySeconds = remaining > 0 ? remaining : timerElapsed.value
    const mins = Math.floor(Math.abs(displaySeconds) / 60)
    const secs = Math.abs(displaySeconds) % 60
    const prefix = remaining < 0 ? '+' : ''
    return `${prefix}${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`
  })

  /** Whether the Pomodoro timer has exceeded its target. */
  const timerOvertime = computed(() => timerElapsed.value > pomodoroMinutes.value * 60)

  /** Progress percentage for the Pomodoro timer (0-100, can exceed 100). */
  const timerProgress = computed(() => {
    const total = pomodoroMinutes.value * 60
    return Math.min((timerElapsed.value / total) * 100, 100)
  })

  /** File review progress as a percentage. */
  const reviewProgress = computed(() => {
    if (!session.value || totalFiles.value === 0) return 0
    return Math.round((session.value.files_reviewed.length / totalFiles.value) * 100)
  })

  // Total files in the diff — set externally via setTotalFiles
  const totalFiles = ref(0)

  function setTotalFiles(count: number) {
    totalFiles.value = count
  }

  /** Start or resume an existing session for this PR. */
  async function startSession() {
    loading.value = true
    error.value = null
    try {
      session.value = await invoke<ReviewSession>('create_review_session', { prId })
      startTimer()
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  /** Resume a previously paused session. */
  async function resumeSession() {
    if (!session.value) return
    try {
      await invoke('update_session_status', {
        sessionId: session.value.id,
        status: 'active',
      })
      session.value = { ...session.value, status: 'active' }
      startTimer()
    } catch (e) {
      error.value = String(e)
    }
  }

  /** Pause the current session. */
  async function pauseSession() {
    if (!session.value) return
    stopTimer()
    try {
      await invoke('update_session_status', {
        sessionId: session.value.id,
        status: 'paused',
      })
      session.value = { ...session.value, status: 'paused' }
    } catch (e) {
      error.value = String(e)
    }
  }

  /** Complete the session (marks it as finished). */
  async function completeSession() {
    if (!session.value) return
    stopTimer()
    try {
      await invoke('update_session_status', {
        sessionId: session.value.id,
        status: 'completed',
      })
      session.value = { ...session.value, status: 'completed' }
    } catch (e) {
      error.value = String(e)
    }
  }

  /** Toggle a file's reviewed state. */
  async function toggleFileReviewed(filePath: string) {
    if (!session.value) return
    const current = [...session.value.files_reviewed]
    const idx = current.indexOf(filePath)
    if (idx >= 0) {
      current.splice(idx, 1)
    } else {
      current.push(filePath)
    }
    session.value = { ...session.value, files_reviewed: current }
    try {
      await invoke('update_session_files', {
        sessionId: session.value.id,
        filesReviewed: current,
      })
    } catch (e) {
      error.value = String(e)
    }
  }

  /** Check whether a file has been marked as reviewed. */
  function isFileReviewed(filePath: string): boolean {
    return session.value?.files_reviewed.includes(filePath) ?? false
  }

  /** Save session notes (debounced externally if desired). */
  async function saveNotes(notes: string) {
    if (!session.value) return
    session.value = { ...session.value, session_notes: notes }
    try {
      await invoke('update_session_notes', {
        sessionId: session.value.id,
        notes,
      })
    } catch (e) {
      error.value = String(e)
    }
  }

  /** Try to load an existing session for this PR on mount. */
  async function loadExistingSession() {
    try {
      const existing = await invoke<ReviewSession | null>('get_session_for_pr', { prId })
      if (!isMounted) return
      if (existing) {
        session.value = existing
        if (existing.status === 'active') {
          startTimer()
        }
      }
    } catch {
      // No existing session — that is fine
    }
  }

  // Timer controls
  function startTimer() {
    if (timerRunning.value) return
    timerRunning.value = true
    intervalId = setInterval(() => {
      timerElapsed.value++
    }, 1000)
  }

  function stopTimer() {
    timerRunning.value = false
    if (intervalId !== null) {
      clearInterval(intervalId)
      intervalId = null
    }
  }

  function resetTimer() {
    stopTimer()
    timerElapsed.value = 0
  }

  // Pause timer when the tab is hidden
  function handleVisibilityChange() {
    if (!session.value || session.value.status !== 'active') return
    if (document.hidden) {
      stopTimer()
    } else {
      startTimer()
    }
  }

  onMounted(() => {
    isMounted = true
    document.addEventListener('visibilitychange', handleVisibilityChange)
    loadExistingSession()
  })

  onUnmounted(() => {
    isMounted = false
    document.removeEventListener('visibilitychange', handleVisibilityChange)
    stopTimer()
  })

  return {
    session,
    loading,
    error,
    timerElapsed,
    timerRunning,
    timerDisplay,
    timerOvertime,
    timerProgress,
    pomodoroMinutes,
    reviewProgress,
    totalFiles,
    setTotalFiles,
    startSession,
    resumeSession,
    pauseSession,
    completeSession,
    toggleFileReviewed,
    isFileReviewed,
    saveNotes,
    resetTimer,
  }
}
