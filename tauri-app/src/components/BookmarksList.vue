<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Bug, HelpCircle, Lightbulb, AlertOctagon, StickyNote, Check, Trash2 } from 'lucide-vue-next'
import { useBookmarks } from '../composables/useBookmarks'
import type { Bookmark, BookmarkCategory } from '../types'

const props = withDefaults(defineProps<{
  prId: number
  availableFiles?: string[]
}>(), {
  availableFiles: () => [],
})

const emit = defineEmits<{
  'navigate-to-bookmark': [bookmark: Bookmark]
}>()

const { bookmarks, loading, error, fetchBookmarks, addBookmark, removeBookmark, updateBookmark, toggleResolved } = useBookmarks()

const expanded = ref(true)
const showAddForm = ref(false)
const newFilePath = ref('')
const newNote = ref('')
const newLineStart = ref<number | undefined>(undefined)
const newLineEnd = ref<number | undefined>(undefined)
const newCategory = ref<BookmarkCategory>('note')
const editingId = ref<number | null>(null)
const editNote = ref('')
const editCategory = ref<BookmarkCategory>('note')

const categoryOptions: { value: BookmarkCategory; label: string }[] = [
  { value: 'note', label: 'Note' },
  { value: 'bug', label: 'Bug' },
  { value: 'question', label: 'Question' },
  { value: 'suggestion', label: 'Suggestion' },
  { value: 'blocker', label: 'Blocker' },
]

const categoryIcons: Record<BookmarkCategory, typeof Bug> = {
  note: StickyNote,
  bug: Bug,
  question: HelpCircle,
  suggestion: Lightbulb,
  blocker: AlertOctagon,
}

const categoryColours: Record<BookmarkCategory, string> = {
  note: 'var(--color-text-muted)',
  bug: 'var(--color-status-danger)',
  question: 'var(--color-status-info)',
  suggestion: 'var(--color-status-success)',
  blocker: 'var(--color-status-warning)',
}

onMounted(() => {
  fetchBookmarks(props.prId)
})

/**
 * Pre-fill and show the add form (used by DiffViewer bookmark button).
 */
function prefillBookmark(filePath: string, lineStart?: number | null, lineEnd?: number | null) {
  newFilePath.value = filePath
  newLineStart.value = lineStart ?? undefined
  newLineEnd.value = lineEnd ?? undefined
  newNote.value = ''
  newCategory.value = 'note'
  showAddForm.value = true
}

async function handleAdd() {
  if (!newFilePath.value.trim()) return
  await addBookmark(
    props.prId,
    newFilePath.value.trim(),
    newNote.value.trim(),
    newLineStart.value ?? null,
    newLineEnd.value ?? null,
    newCategory.value,
  )
  newFilePath.value = ''
  newNote.value = ''
  newLineStart.value = undefined
  newLineEnd.value = undefined
  newCategory.value = 'note'
  showAddForm.value = false
}

function startEdit(bookmark: Bookmark) {
  editingId.value = bookmark.id
  editNote.value = bookmark.note
  editCategory.value = bookmark.category
}

async function saveEdit(bookmark: Bookmark) {
  await updateBookmark(bookmark.id, editNote.value, bookmark.line_start, bookmark.line_end, editCategory.value)
  editingId.value = null
  editNote.value = ''
}

function cancelEdit() {
  editingId.value = null
  editNote.value = ''
}

function handleBookmarkClick(bookmark: Bookmark) {
  emit('navigate-to-bookmark', bookmark)
}

function formatLineRange(bookmark: Bookmark): string {
  if (bookmark.line_start == null) return ''
  if (bookmark.line_end == null || bookmark.line_end === bookmark.line_start) {
    return `L${bookmark.line_start}`
  }
  return `L${bookmark.line_start}-${bookmark.line_end}`
}

defineExpose({ prefillBookmark })
</script>

<template>
  <section class="bookmarks-panel">
    <button class="bookmarks-toggle" @click="expanded = !expanded">
      <h3 class="bookmarks-title">Bookmarks</h3>
      <span class="bookmarks-count" v-if="bookmarks.length > 0">{{ bookmarks.length }}</span>
      <span class="toggle-icon" :class="{ expanded }">&#9656;</span>
    </button>

    <div v-if="expanded" class="bookmarks-content">
      <div v-if="loading" class="bookmarks-loading">Loading bookmarks...</div>

      <div v-else>
        <div v-if="bookmarks.length === 0 && !showAddForm" class="bookmarks-empty">
          No bookmarks yet. Add one to annotate files in this PR.
        </div>

        <div class="bookmarks-list">
          <div
            v-for="bookmark in bookmarks"
            :key="bookmark.id"
            class="bookmark-card"
            :class="{ 'bookmark-resolved': bookmark.resolved }"
            @click="handleBookmarkClick(bookmark)"
          >
            <div class="bookmark-header">
              <span
                class="bookmark-category-icon"
                :style="{ color: categoryColours[bookmark.category] }"
                :title="bookmark.category"
              >
                <component :is="categoryIcons[bookmark.category]" :size="14" />
              </span>
              <code class="bookmark-file">{{ bookmark.file_path }}</code>
              <span v-if="formatLineRange(bookmark)" class="bookmark-lines">
                {{ formatLineRange(bookmark) }}
              </span>
              <button
                class="bookmark-resolve-btn"
                :title="bookmark.resolved ? 'Mark as unresolved' : 'Mark as resolved'"
                @click.stop="toggleResolved(bookmark.id)"
              >
                <Check :size="14" :class="{ 'resolved-check': bookmark.resolved }" />
              </button>
              <button
                class="bookmark-delete"
                title="Delete bookmark"
                @click.stop="removeBookmark(bookmark.id)"
              >
                <Trash2 :size="13" />
              </button>
            </div>

            <div v-if="editingId === bookmark.id" class="bookmark-edit" @click.stop>
              <select v-model="editCategory" class="bookmark-category-select">
                <option v-for="opt in categoryOptions" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
              <textarea
                v-model="editNote"
                class="bookmark-edit-input"
                rows="2"
                placeholder="Update note..."
                @keydown.enter.meta="saveEdit(bookmark)"
              />
              <div class="bookmark-edit-actions">
                <button class="btn-save-edit" @click="saveEdit(bookmark)">Save</button>
                <button class="btn-cancel-edit" @click="cancelEdit">Cancel</button>
              </div>
            </div>
            <p v-else class="bookmark-note" @dblclick.stop="startEdit(bookmark)">
              {{ bookmark.note || 'No note' }}
            </p>
          </div>
        </div>

        <div v-if="showAddForm" class="bookmark-add-form" @click.stop>
          <select
            v-if="props.availableFiles.length > 0"
            v-model="newFilePath"
            class="bookmark-input bookmark-file-select"
          >
            <option value="" disabled>Select a file…</option>
            <option v-for="file in props.availableFiles" :key="file" :value="file">
              {{ file }}
            </option>
          </select>
          <input
            v-else
            v-model="newFilePath"
            class="bookmark-input"
            placeholder="File path (e.g. src/main.rs)"
            @keydown.enter="handleAdd"
          />
          <div class="bookmark-line-inputs">
            <input
              v-model.number="newLineStart"
              type="number"
              class="bookmark-input bookmark-line-input"
              placeholder="Start line"
            />
            <input
              v-model.number="newLineEnd"
              type="number"
              class="bookmark-input bookmark-line-input"
              placeholder="End line"
            />
          </div>
          <select v-model="newCategory" class="bookmark-input bookmark-category-select">
            <option v-for="opt in categoryOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
          <textarea
            v-model="newNote"
            class="bookmark-input bookmark-note-input"
            placeholder="Note (optional)"
            rows="2"
          />
          <div class="bookmark-form-actions">
            <button class="btn-add-bookmark" @click="handleAdd">Add</button>
            <button class="btn-cancel-bookmark" @click="showAddForm = false">Cancel</button>
          </div>
        </div>

        <button
          v-if="!showAddForm"
          class="btn-new-bookmark"
          @click="showAddForm = true"
        >
          + Add Bookmark
        </button>
      </div>

      <div v-if="error" class="bookmarks-error">{{ error }}</div>
    </div>
  </section>
</template>

<style scoped>
.bookmarks-panel {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  margin-top: var(--space-4);
  overflow: hidden;
}

.bookmarks-toggle {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  width: 100%;
  padding: var(--space-4) var(--space-5);
  background: none;
  border: none;
  cursor: pointer;
  color: inherit;
  text-align: left;
  transition: background var(--transition-fast);
}

.bookmarks-toggle:hover {
  background: rgba(255, 255, 255, 0.02);
}

.bookmarks-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
  flex: 1;
}

.bookmarks-count {
  background: var(--color-accent-muted);
  color: var(--color-accent);
  font-size: 11px;
  font-weight: 700;
  min-width: 20px;
  height: 20px;
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 var(--space-1);
}

.toggle-icon {
  font-size: 14px;
  color: var(--color-text-muted);
  transition: transform var(--transition-fast);
  display: inline-block;
}

.toggle-icon.expanded {
  transform: rotate(90deg);
}

.bookmarks-content {
  padding: 0 var(--space-5) var(--space-4);
}

.bookmarks-loading,
.bookmarks-empty {
  text-align: center;
  color: var(--color-text-muted);
  font-size: 13px;
  padding: var(--space-2);
}

.bookmarks-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.bookmark-card {
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-3);
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  cursor: pointer;
  transition: border-color var(--transition-fast), background var(--transition-fast);
}

.bookmark-card:hover {
  border-color: var(--color-accent);
  background: rgba(20, 184, 166, 0.04);
}

.bookmark-resolved {
  opacity: 0.6;
}

.bookmark-resolved .bookmark-note {
  text-decoration: line-through;
}

.bookmark-header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.bookmark-category-icon {
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.bookmark-file {
  font-size: 12px;
  font-family: var(--font-mono);
  color: var(--color-accent);
  background: rgba(20, 184, 166, 0.1);
  padding: 1px var(--space-2);
  border-radius: var(--radius-sm);
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.bookmark-lines {
  font-size: 11px;
  font-family: var(--font-mono);
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.bookmark-resolve-btn {
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 0 2px;
  line-height: 1;
  transition: color var(--transition-fast);
  flex-shrink: 0;
  display: flex;
  align-items: center;
}

.bookmark-resolve-btn:hover {
  color: var(--color-status-success);
}

.resolved-check {
  color: var(--color-status-success);
}

.bookmark-delete {
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 0 2px;
  line-height: 1;
  transition: color var(--transition-fast);
  flex-shrink: 0;
  display: flex;
  align-items: center;
}

.bookmark-delete:hover {
  color: var(--color-status-danger);
}

.bookmark-note {
  font-size: 13px;
  color: var(--color-text-secondary);
  margin: 0;
  line-height: 1.4;
  cursor: pointer;
}

.bookmark-note:hover {
  color: var(--color-text-primary);
}

.bookmark-edit {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.bookmark-edit-input {
  width: 100%;
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-2);
  color: var(--color-text-primary);
  font-size: 13px;
  font-family: var(--font-sans);
  resize: vertical;
}

.bookmark-edit-input:focus {
  border-color: var(--color-border-focus);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.bookmark-edit-actions {
  display: flex;
  gap: var(--space-2);
}

.btn-save-edit,
.btn-cancel-edit {
  font-size: 12px;
  padding: var(--space-1) var(--space-3);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-default);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-save-edit {
  background: rgba(20, 184, 166, 0.2);
  color: var(--color-accent);
  border-color: rgba(20, 184, 166, 0.3);
}

.btn-save-edit:hover {
  background: rgba(20, 184, 166, 0.3);
}

.btn-cancel-edit {
  background: var(--color-surface-raised);
  color: var(--color-text-secondary);
}

.btn-cancel-edit:hover {
  background: var(--color-surface-hover);
}

.bookmark-category-select {
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-1) var(--space-2);
  color: var(--color-text-primary);
  font-size: 12px;
  cursor: pointer;
}

.bookmark-category-select:focus {
  border-color: var(--color-border-focus);
  outline: none;
}

.bookmark-add-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  margin-top: var(--space-3);
  padding: var(--space-3);
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
}

.bookmark-input {
  width: 100%;
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-3);
  color: var(--color-text-primary);
  font-size: 13px;
  font-family: var(--font-sans);
  transition: border-color var(--transition-fast);
}

.bookmark-input:focus {
  border-color: var(--color-border-focus);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.bookmark-file-select {
  font-family: var(--font-mono);
  font-size: 12px;
  cursor: pointer;
}

.bookmark-line-inputs {
  display: flex;
  gap: var(--space-2);
}

.bookmark-line-input {
  flex: 1;
}

.bookmark-note-input {
  resize: vertical;
  min-height: 40px;
}

.bookmark-form-actions {
  display: flex;
  gap: var(--space-2);
}

.btn-add-bookmark {
  font-size: 12px;
  padding: var(--space-1) var(--space-3);
  border-radius: var(--radius-md);
  background: rgba(20, 184, 166, 0.2);
  color: var(--color-accent);
  border: 1px solid rgba(20, 184, 166, 0.3);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-add-bookmark:hover {
  background: rgba(20, 184, 166, 0.3);
}

.btn-cancel-bookmark {
  font-size: 12px;
  padding: var(--space-1) var(--space-3);
  border-radius: var(--radius-md);
  background: var(--color-surface-raised);
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border-default);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-cancel-bookmark:hover {
  background: var(--color-surface-hover);
}

.btn-new-bookmark {
  width: 100%;
  margin-top: var(--space-3);
  padding: var(--space-2);
  background: none;
  border: 1px dashed var(--color-border-default);
  border-radius: var(--radius-md);
  color: var(--color-text-muted);
  font-size: 13px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-new-bookmark:hover {
  border-color: var(--color-accent);
  color: var(--color-accent);
  background: rgba(20, 184, 166, 0.05);
}

.bookmarks-error {
  color: var(--color-status-danger);
  font-size: 12px;
  margin-top: var(--space-2);
}
</style>
