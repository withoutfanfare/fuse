<script setup lang="ts">
import { ref, onMounted, computed, watch, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { usePullRequestsStore } from '../stores/pullRequests'
import { useRepositoriesStore } from '../stores/repositories'
import { useDiff } from '../composables/useDiff'
import { useReviewSession } from '../composables/useReviewSession'
import type { PullRequest, DiffFile } from '../types'
import { ArrowLeft } from 'lucide-vue-next'
import { SBreadcrumbs, SSelect, SProgressBar } from '@stuntrocket/ui'
import ContentLoader from '../components/ContentLoader.vue'

const route = useRoute()
const router = useRouter()
const prStore = usePullRequestsStore()
const repoStore = useRepositoriesStore()

const prId = Number(route.params.prId)
const pr = ref<PullRequest | null>(null)
const loading = ref(true)
const selectedFile = ref<string | null>(null)
const notesText = ref('')
let notesSaveTimeout: ReturnType<typeof setTimeout> | null = null

const { files: diffFiles, fetchDiff, loading: diffLoading } = useDiff()
const {
  session,
  timerDisplay,
  timerOvertime,
  timerProgress,
  pomodoroMinutes,
  reviewProgress,
  setTotalFiles,
  startSession,
  resumeSession,
  pauseSession,
  completeSession,
  toggleFileReviewed,
  isFileReviewed,
  saveNotes,
} = useReviewSession(prId)

/** SSelect string bridge for pomodoro duration. */
const pomodoroSelectValue = computed({
  get() { return String(pomodoroMinutes.value) },
  set(val: string) { pomodoroMinutes.value = Number(val) },
})
const pomodoroOptions = [
  { value: '15', label: '15 min' },
  { value: '25', label: '25 min' },
  { value: '45', label: '45 min' },
  { value: '60', label: '60 min' },
]

const repoFullName = computed(() => {
  if (!pr.value) return ''
  const repo = repoStore.repos.find(r => r.id === pr.value!.repo_id)
  return repo ? `${repo.owner}/${repo.name}` : ''
})

const breadcrumbItems = computed(() => [
  { label: 'Pull Requests', to: '/prs' },
  { label: pr.value ? `#${pr.value.number} ${pr.value.title}` : '...', to: pr.value ? `/prs/${pr.value.id}` : undefined },
  { label: 'Review Session' },
])

/** The currently selected file's diff data. */
const activeDiffFile = computed<DiffFile | null>(() => {
  if (!selectedFile.value) return null
  return diffFiles.value.find(f => f.path === selectedFile.value) ?? null
})

/** Build a flat list of file paths for the tree panel. */
const filePaths = computed(() => diffFiles.value.map(f => f.path))

function selectFile(path: string) {
  selectedFile.value = path
  // Scroll the diff panel to top when switching files
  nextTick(() => {
    const el = document.querySelector('.session-diff-panel')
    if (el) el.scrollTop = 0
  })
}

function handleNotesInput(event: Event) {
  const value = (event.target as HTMLTextAreaElement).value
  notesText.value = value
  // Debounce saving notes to avoid excessive writes
  if (notesSaveTimeout) clearTimeout(notesSaveTimeout)
  notesSaveTimeout = setTimeout(() => {
    saveNotes(value)
  }, 800)
}

async function handleBackToPr() {
  await pauseSession()
  router.push({ name: 'pr-detail', params: { id: prId } })
}

async function handleComplete() {
  await completeSession()
  router.push({ name: 'pr-detail', params: { id: prId } })
}

onMounted(async () => {
  if (repoStore.repos.length === 0) await repoStore.fetchAll()
  pr.value = await prStore.fetchOne(prId)
  loading.value = false

  if (pr.value) {
    await fetchDiff(pr.value.id)
    setTotalFiles(diffFiles.value.length)

    // Auto-start a new session, or resume a paused one
    if (!session.value) {
      await startSession()
    } else if (session.value.status === 'paused') {
      await resumeSession()
    }

    // Pre-populate notes from existing session
    if (session.value?.session_notes) {
      notesText.value = session.value.session_notes
    }

    // Select first file by default
    if (diffFiles.value.length > 0) {
      selectedFile.value = diffFiles.value[0].path
    }
  }
})

// Update total files when diff loads
watch(() => diffFiles.value.length, (count) => {
  setTotalFiles(count)
})
</script>

<template>
  <div class="review-session-page">
  <ContentLoader v-if="loading" variant="detail" :count="3" />
  <div v-else-if="!pr" class="session-not-found">
    <p>Pull request not found.</p>
    <button @click="router.push({ name: 'pull-requests' })">Back to PRs</button>
  </div>
  <div v-else class="review-session">
    <div class="session-header">
      <div class="session-header-top">
        <button class="btn-back" @click="handleBackToPr" title="Back to PR (pauses timer)">
          <ArrowLeft :size="16" />
          <span>Back to PR</span>
        </button>
        <SBreadcrumbs :segments="breadcrumbItems" @navigate="$router.push($event)" />
      </div>
      <div class="session-toolbar">
        <div class="session-pr-info">
          <span class="session-pr-number">#{{ pr.number }}</span>
          <span class="session-pr-title">{{ pr.title }}</span>
          <span class="session-pr-repo">{{ repoFullName }}</span>
        </div>

        <!-- Pomodoro timer -->
        <div class="timer-section">
          <div class="timer-ring" :class="{ 'timer-overtime': timerOvertime }">
            <svg viewBox="0 0 36 36" class="timer-svg">
              <circle
                cx="18" cy="18" r="15.9"
                fill="none"
                stroke="rgba(255,255,255,0.06)"
                stroke-width="2"
              />
              <circle
                cx="18" cy="18" r="15.9"
                fill="none"
                :stroke="timerOvertime ? 'var(--color-status-warning)' : 'var(--color-accent)'"
                stroke-width="2"
                stroke-linecap="round"
                :stroke-dasharray="`${timerProgress} ${100 - timerProgress}`"
                stroke-dashoffset="25"
                class="timer-progress-ring"
              />
            </svg>
            <span class="timer-value">{{ timerDisplay }}</span>
          </div>
          <div class="timer-controls">
            <button
              v-if="session?.status === 'active'"
              class="btn-timer"
              @click="pauseSession"
              title="Pause session"
            >
              Pause
            </button>
            <button
              v-else-if="session?.status === 'paused'"
              class="btn-timer btn-resume"
              @click="resumeSession"
              title="Resume session"
            >
              Resume
            </button>
            <SSelect v-model="pomodoroSelectValue" size="sm">
              <option v-for="opt in pomodoroOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </SSelect>
          </div>
        </div>

        <!-- Progress bar -->
        <div class="progress-section">
          <SProgressBar :value="reviewProgress / 100" variant="accent" size="sm" />
          <span class="progress-label">{{ reviewProgress }}% reviewed</span>
        </div>

        <button class="btn-complete" @click="handleComplete">
          Complete Review
        </button>
      </div>
    </div>

    <div class="session-panels">
      <!-- Left panel: file tree with checkboxes -->
      <div class="session-file-tree">
        <div class="file-tree-header">Files</div>
        <div v-if="diffLoading" class="file-tree-loading">Loading diff...</div>
        <div v-else class="file-tree-list">
          <div
            v-for="path in filePaths"
            :key="path"
            class="file-tree-item"
            :class="{
              'file-selected': selectedFile === path,
              'file-reviewed': isFileReviewed(path),
            }"
            @click="selectFile(path)"
          >
            <label class="file-checkbox-label" @click.stop>
              <input
                type="checkbox"
                class="file-checkbox"
                :checked="isFileReviewed(path)"
                @change="toggleFileReviewed(path)"
              />
            </label>
            <div class="file-info" :title="path">
              <span class="file-name">{{ path.split('/').pop() }}</span>
              <span v-if="path.includes('/')" class="file-path-hint">
                {{ path.substring(0, path.lastIndexOf('/')) }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Centre panel: diff viewer for selected file -->
      <div class="session-diff-panel">
        <div v-if="!activeDiffFile" class="diff-placeholder">
          Select a file from the tree to view its diff.
        </div>
        <div v-else class="diff-single-file">
          <div class="diff-file-header-bar">
            <span class="diff-file-path">{{ activeDiffFile.path }}</span>
            <span class="diff-file-stats">
              <span class="diff-stat-add">+{{ activeDiffFile.additions }}</span>
              <span class="diff-stat-del">-{{ activeDiffFile.deletions }}</span>
            </span>
          </div>
          <div class="diff-hunks-scroll">
            <div v-for="(hunk, hunkIdx) in activeDiffFile.hunks" :key="hunkIdx" class="diff-hunk">
              <div class="diff-hunk-header">{{ hunk.header }}</div>
              <div class="diff-lines">
                <div
                  v-for="(line, lineIdx) in hunk.lines"
                  :key="lineIdx"
                  class="diff-line"
                  :class="`diff-line-${line.type}`"
                >
                  <span class="line-number line-number-old">{{ line.oldLineNumber ?? '' }}</span>
                  <span class="line-number line-number-new">{{ line.newLineNumber ?? '' }}</span>
                  <span class="line-prefix">{{ line.type === 'add' ? '+' : line.type === 'remove' ? '-' : ' ' }}</span>
                  <span class="line-content">{{ line.content }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Right panel: notes and checklist -->
      <div class="session-notes-panel">
        <div class="notes-section">
          <h4 class="notes-title">Session Notes</h4>
          <textarea
            class="notes-textarea"
            :value="notesText"
            placeholder="Write your review notes here..."
            @input="handleNotesInput"
          />
        </div>

        <div class="session-info-section">
          <h4 class="notes-title">Session Info</h4>
          <div class="info-row">
            <span class="info-label">Status</span>
            <span class="info-value" :class="`status-${session?.status}`">
              {{ session?.status ?? 'N/A' }}
            </span>
          </div>
          <div class="info-row">
            <span class="info-label">Started</span>
            <span class="info-value">
              {{ session?.started_at ? new Date(session.started_at).toLocaleTimeString('en-GB') : '—' }}
            </span>
          </div>
          <div class="info-row">
            <span class="info-label">Files reviewed</span>
            <span class="info-value">
              {{ session?.files_reviewed.length ?? 0 }} / {{ filePaths.length }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
  </div>
</template>

<style scoped>
.session-loading,
.session-not-found {
  text-align: center;
  padding: var(--space-12);
  color: var(--color-text-muted);
}

.session-not-found button {
  margin-top: var(--space-4);
  background: var(--color-surface-raised);
  color: var(--color-text-primary);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-4);
  cursor: pointer;
}

.review-session {
  display: flex;
  flex-direction: column;
  height: calc(100vh - 60px);
  overflow: hidden;
}

.session-header {
  padding: var(--space-4) var(--space-6);
  border-bottom: 1px solid var(--color-border-default);
  background: var(--color-surface-panel);
  flex-shrink: 0;
}

.session-header-top {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.session-header-top :deep(.breadcrumb) {
  margin-bottom: 0;
}

.btn-back {
  display: flex;
  align-items: center;
  gap: var(--space-1-5);
  padding: var(--space-1) var(--space-3) var(--space-1) var(--space-2);
  background: var(--color-surface-hover);
  border: none;
  border-radius: var(--radius-full);
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
  white-space: nowrap;
  flex-shrink: 0;
}

.btn-back:hover {
  background: var(--color-border-default);
  color: var(--color-text-primary);
}

.session-toolbar {
  display: flex;
  align-items: center;
  gap: var(--space-6);
  margin-top: var(--space-3);
}

.session-pr-info {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex: 1;
  min-width: 0;
}

.session-pr-number {
  font-family: var(--font-mono);
  font-size: 13px;
  font-weight: 700;
  color: var(--color-text-primary);
  flex-shrink: 0;
}

.session-pr-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.session-pr-repo {
  font-size: 11px;
  color: var(--color-text-muted);
  flex-shrink: 0;
}

/* Timer section */
.timer-section {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  flex-shrink: 0;
}

.timer-ring {
  position: relative;
  width: 42px;
  height: 42px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.timer-svg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  transform: rotate(-90deg);
}

.timer-progress-ring {
  transition: stroke-dasharray 1s linear;
}

.timer-value {
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-primary);
  z-index: 1;
}

.timer-overtime .timer-value {
  color: var(--color-status-warning);
}

.timer-controls {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.btn-timer {
  background: var(--color-surface-raised);
  color: var(--color-text-secondary);
  font-size: 11px;
  font-weight: 500;
  padding: 2px var(--space-2);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-timer:hover {
  background: var(--color-surface-hover);
}

.btn-resume {
  color: var(--color-accent);
  border-color: rgba(20, 184, 166, 0.3);
}


/* Progress section */
.progress-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  min-width: 140px;
  flex: 1;
  max-width: 240px;
}


.progress-label {
  font-size: 10px;
  color: var(--color-text-muted);
  text-align: right;
}

.btn-complete {
  background: rgba(34, 197, 94, 0.2);
  color: var(--color-status-success);
  font-size: 13px;
  font-weight: 600;
  padding: var(--space-2) var(--space-5);
  border: 1px solid rgba(34, 197, 94, 0.3);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  white-space: nowrap;
  flex-shrink: 0;
}

.btn-complete:hover {
  background: rgba(34, 197, 94, 0.3);
  border-color: rgba(34, 197, 94, 0.5);
}

/* Three-panel layout */
.session-panels {
  display: grid;
  grid-template-columns: 240px 1fr 280px;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

/* Left: file tree */
.session-file-tree {
  border-right: 1px solid var(--color-border-default);
  overflow-y: auto;
  background: var(--color-surface-panel);
}

.file-tree-header {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: var(--space-3);
  border-bottom: 1px solid var(--color-border-default);
  position: sticky;
  top: 0;
  background: var(--color-surface-panel);
  z-index: 1;
}

.file-tree-loading {
  text-align: center;
  color: var(--color-text-muted);
  font-size: 12px;
  padding: var(--space-4);
}

.file-tree-list {
  display: flex;
  flex-direction: column;
}

.file-tree-item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  font-size: 12px;
  cursor: pointer;
  transition: background var(--transition-fast);
}

.file-tree-item:hover {
  background: var(--color-surface-hover);
}

.file-tree-item.file-selected {
  background: rgba(20, 184, 166, 0.1);
}

.file-tree-item.file-reviewed {
  opacity: 0.6;
}

.file-tree-item.file-reviewed .file-name {
  text-decoration: line-through;
  color: var(--color-text-muted);
}

.file-checkbox-label {
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.file-checkbox {
  appearance: none;
  width: 14px;
  height: 14px;
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-sm);
  background: var(--color-surface-input);
  cursor: pointer;
  transition: all var(--transition-fast);
  position: relative;
}

.file-checkbox:checked {
  background: var(--color-accent);
  border-color: var(--color-accent);
}

.file-checkbox:checked::after {
  content: '';
  position: absolute;
  left: 50%;
  top: 45%;
  width: 5px;
  height: 8px;
  border: solid var(--color-text-inverse);
  border-width: 0 2px 2px 0;
  transform: translate(-50%, -55%) rotate(45deg);
}

.file-info {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
  flex: 1;
}

.file-name {
  color: var(--color-text-primary);
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-path-hint {
  color: var(--color-text-muted);
  font-size: 10px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Centre: diff viewer */
.session-diff-panel {
  overflow: auto;
  background: var(--color-surface-base);
}

.diff-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--color-text-muted);
  font-size: 13px;
}

.diff-single-file {
  display: flex;
  flex-direction: column;
  min-height: 100%;
}

.diff-file-header-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-2) var(--space-4);
  background: rgba(255, 255, 255, 0.03);
  border-bottom: 1px solid var(--color-border-default);
  position: sticky;
  top: 0;
  z-index: 1;
}

.diff-file-path {
  font-family: var(--font-mono);
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.diff-file-stats {
  display: flex;
  gap: var(--space-2);
  font-family: var(--font-mono);
  font-size: 12px;
  font-weight: 600;
}

.diff-stat-add {
  color: var(--color-status-success);
}

.diff-stat-del {
  color: var(--color-status-danger);
}

.diff-hunks-scroll {
  flex: 1;
}

.diff-hunk-header {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--color-text-muted);
  background: rgba(96, 165, 250, 0.08);
  padding: var(--space-1) var(--space-4);
  border-bottom: 1px solid var(--color-border-default);
}

.diff-lines {
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: var(--diff-line-height, 1.6);
}

.diff-line {
  display: flex;
  white-space: pre;
  min-height: 20px;
}

.diff-line-add {
  background: rgba(34, 197, 94, 0.1);
}

.diff-line-remove {
  background: rgba(220, 38, 38, 0.1);
}

.diff-line-context {
  background: transparent;
}

.line-number {
  display: inline-block;
  width: 48px;
  min-width: 48px;
  text-align: right;
  padding-right: var(--space-2);
  color: var(--color-text-muted);
  font-size: 11px;
  user-select: none;
  flex-shrink: 0;
  opacity: 0.6;
}

.line-prefix {
  display: inline-block;
  width: 16px;
  min-width: 16px;
  text-align: center;
  color: var(--color-text-muted);
  user-select: none;
  flex-shrink: 0;
}

.diff-line-add .line-prefix {
  color: var(--color-status-success);
}

.diff-line-remove .line-prefix {
  color: var(--color-status-danger);
}

.line-content {
  padding-right: var(--space-4);
  overflow-x: visible;
}

/* Right: notes panel */
.session-notes-panel {
  border-left: 1px solid var(--color-border-default);
  overflow-y: auto;
  background: var(--color-surface-panel);
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
  padding: var(--space-4);
}

.notes-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  flex: 1;
  min-height: 200px;
}

.notes-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 0;
}

.notes-textarea {
  flex: 1;
  width: 100%;
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-3);
  color: var(--color-text-primary);
  font-size: 13px;
  font-family: var(--font-sans);
  resize: none;
  line-height: 1.5;
  transition: border-color var(--transition-fast);
}

.notes-textarea:focus {
  border-color: var(--color-border-focus);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.notes-textarea::placeholder {
  color: var(--color-text-muted);
}

.session-info-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.info-row {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  padding: var(--space-1) 0;
}

.info-label {
  color: var(--color-text-muted);
}

.info-value {
  color: var(--color-text-secondary);
  font-family: var(--font-mono);
  font-size: 11px;
}

.info-value.status-active {
  color: var(--color-status-success);
}

.info-value.status-paused {
  color: var(--color-status-warning);
}

.info-value.status-completed {
  color: var(--color-accent);
}
</style>
