<script setup lang="ts">
import { ref, nextTick, onMounted, computed } from 'vue'
import { BookmarkPlus } from 'lucide-vue-next'
import { SIconButton, SEmptyState, useContextMenu } from '@stuntrocket/ui'
import type { DiffFile, DiffLine } from '../types'
import DiffFileTree from './DiffFileTree.vue'
import { useSyntaxHighlight } from '../composables/useSyntaxHighlight'

const props = defineProps<{
  files: DiffFile[]
}>()

const { ensureLoaded, detectLanguage, highlightLine, loaded: hlLoaded } = useSyntaxHighlight()

// Lazily load highlight.js when component mounts
onMounted(() => {
  ensureLoaded()
})

/** Cache detected languages per file to avoid repeated lookups. */
const fileLanguageMap = computed(() => {
  const map: Record<string, string | null> = {}
  for (const file of props.files) {
    map[file.path] = detectLanguage(file.path)
  }
  return map
})

/**
 * Get highlighted HTML for a diff line's content.
 * Returns raw HTML that must be bound with v-html.
 */
function getHighlightedContent(filePath: string, content: string): string {
  if (!hlLoaded.value) return escapeHtml(content)
  const lang = fileLanguageMap.value[filePath]
  return highlightLine(content, lang)
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
}

const emit = defineEmits<{
  'bookmark-file': [filePath: string, lineStart: number | null, lineEnd: number | null]
}>()

const sidebarCollapsed = ref(false)

/** Context menu — powered by library composable */
const { visible: ctxVisible, position: ctxPosition, items: ctxItems, open: ctxOpen, close: ctxClose } = useContextMenu()
/** Tracks which file/line the context menu was opened on */
const ctxTarget = ref<{ filePath: string; lineNumber: number }>({ filePath: '', lineNumber: 0 })

function scrollToFile(path: string) {
  nextTick(() => {
    const el = document.getElementById(`diff-file-${path.replace(/[^a-zA-Z0-9]/g, '-')}`)
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'start' })
    }
  })
}

/**
 * Scroll to a specific line within a file in the diff.
 * Used by bookmark navigation (5.2).
 */
function scrollToLine(filePath: string, lineNumber: number) {
  nextTick(() => {
    // First ensure the file section is visible
    const fileEl = document.getElementById(`diff-file-${filePath.replace(/[^a-zA-Z0-9]/g, '-')}`)
    if (!fileEl) return

    // Find the line element within this file section
    const lineEl = fileEl.querySelector(`[data-line="${lineNumber}"]`) as HTMLElement | null
    if (lineEl) {
      lineEl.scrollIntoView({ behavior: 'smooth', block: 'center' })
      // Add highlight animation
      lineEl.classList.add('diff-line-highlight')
      setTimeout(() => lineEl.classList.remove('diff-line-highlight'), 2000)
    } else {
      // Fall back to scrolling to the file header
      fileEl.scrollIntoView({ behavior: 'smooth', block: 'start' })
    }
  })
}

function handleBookmarkFile(filePath: string) {
  emit('bookmark-file', filePath, null, null)
}

function getLineDataAttr(line: DiffLine): number | undefined {
  return line.newLineNumber ?? line.oldLineNumber
}

function handleLineContextMenu(event: MouseEvent, filePath: string, line: DiffLine) {
  const lineNum = line.newLineNumber ?? line.oldLineNumber
  if (lineNum == null) return

  ctxTarget.value = { filePath, lineNumber: lineNum }
  ctxOpen(event, [
    {
      label: 'Bookmark this line',
      action: () => {
        emit('bookmark-file', ctxTarget.value.filePath, ctxTarget.value.lineNumber, ctxTarget.value.lineNumber)
      },
    },
  ])
}

defineExpose({ scrollToLine, scrollToFile })
</script>

<template>
  <div class="diff-viewer" :class="{ 'sidebar-collapsed': sidebarCollapsed }">
    <div v-if="!sidebarCollapsed" class="diff-sidebar">
      <SIconButton class="sidebar-toggle" @click="sidebarCollapsed = true" title="Collapse file tree">
        &laquo;
      </SIconButton>
      <DiffFileTree :files="files" @select-file="scrollToFile" />
    </div>
    <div v-else class="diff-sidebar-collapsed">
      <SIconButton class="sidebar-toggle" @click="sidebarCollapsed = false" title="Expand file tree">
        &raquo;
      </SIconButton>
    </div>

    <div class="diff-content">
      <SEmptyState
        v-if="files.length === 0"
        title="No changes"
        description="No changes found in this diff."
      />

      <div
        v-for="file in files"
        :key="file.path"
        :id="`diff-file-${file.path.replace(/[^a-zA-Z0-9]/g, '-')}`"
        class="diff-file-section"
      >
        <div class="diff-file-header">
          <span class="diff-file-path">{{ file.path }}</span>
          <span class="diff-file-stats">
            <span class="diff-stat-add">+{{ file.additions }}</span>
            <span class="diff-stat-del">-{{ file.deletions }}</span>
          </span>
          <SIconButton
            size="sm"
            title="Bookmark this file"
            @click.stop="handleBookmarkFile(file.path)"
          >
            <BookmarkPlus :size="14" />
          </SIconButton>
        </div>

        <div v-for="(hunk, hunkIdx) in file.hunks" :key="hunkIdx" class="diff-hunk">
          <div class="diff-hunk-header">{{ hunk.header }}</div>
          <div class="diff-lines">
            <div
              v-for="(line, lineIdx) in hunk.lines"
              :key="lineIdx"
              class="diff-line"
              :class="`diff-line-${line.type}`"
              :data-line="getLineDataAttr(line)"
              @contextmenu="handleLineContextMenu($event, file.path, line)"
            >
              <span class="line-number line-number-old">{{ line.oldLineNumber ?? '' }}</span>
              <span class="line-number line-number-new">{{ line.newLineNumber ?? '' }}</span>
              <span class="line-prefix">{{ line.type === 'add' ? '+' : line.type === 'remove' ? '-' : ' ' }}</span>
              <span class="line-content" v-html="getHighlightedContent(file.path, line.content)" />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Context menu for right-click bookmark -->
    <Teleport to="body">
      <div
        v-if="ctxVisible"
        class="diff-context-menu"
        :style="{ left: ctxPosition.x + 'px', top: ctxPosition.y + 'px' }"
      >
        <button
          v-for="(item, idx) in ctxItems"
          :key="idx"
          class="diff-context-menu-item"
          @click="item.action(); ctxClose()"
        >
          <BookmarkPlus :size="14" />
          {{ item.label }}
        </button>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.diff-viewer {
  display: flex;
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  overflow: hidden;
  background: var(--color-surface-panel);
  max-height: 80vh;
}

.diff-sidebar {
  width: 160px;
  min-width: 160px;
  border-right: 1px solid var(--color-border-default);
  overflow-y: auto;
  position: relative;
  flex-shrink: 0;
}

.diff-sidebar-collapsed {
  width: 28px;
  min-width: 28px;
  border-right: 1px solid var(--color-border-default);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: var(--space-2);
  flex-shrink: 0;
}

.diff-sidebar .sidebar-toggle {
  position: absolute;
  top: var(--space-2);
  right: var(--space-2);
  z-index: 1;
}

.diff-content {
  flex: 1;
  overflow: auto;
}

.diff-file-section {
  border-bottom: 1px solid var(--color-border-default);
}

.diff-file-section:last-child {
  border-bottom: none;
}

.diff-file-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-2) var(--space-4);
  background: rgba(255, 255, 255, 0.03);
  border-bottom: 1px solid var(--color-border-default);
  position: sticky;
  top: 0;
  z-index: 1;
  gap: var(--space-2);
}

.diff-file-path {
  font-family: var(--font-mono);
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-primary);
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.diff-file-stats {
  display: flex;
  gap: var(--space-2);
  font-family: var(--font-mono);
  font-size: 12px;
  font-weight: 600;
  flex-shrink: 0;
}

.diff-stat-add {
  color: var(--color-status-success);
}

.diff-stat-del {
  color: var(--color-status-danger);
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
  font-variant-ligatures: var(--code-ligatures, common-ligatures contextual);
}

.diff-line {
  display: flex;
  white-space: pre;
  min-height: 20px;
  transition: background 0.3s ease;
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

/* Subtle alternating-row stripe for context lines */
.diff-line-context:nth-child(even) {
  background: rgba(255, 255, 255, 0.015);
}

/* Bookmark navigation highlight animation (5.2) */
.diff-line-highlight {
  background: rgba(234, 179, 8, 0.25) !important;
  animation: bookmark-highlight-fade 2s ease-out forwards;
}

@keyframes bookmark-highlight-fade {
  0% { background: rgba(234, 179, 8, 0.35); }
  70% { background: rgba(234, 179, 8, 0.15); }
  100% { background: transparent; }
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
</style>

<style>
/* highlight.js theme colours for dark mode — applied to diff content */
.diff-viewer .hljs-keyword { color: #c678dd; }
.diff-viewer .hljs-string { color: #98c379; }
.diff-viewer .hljs-number { color: #d19a66; }
.diff-viewer .hljs-comment { color: #5c6370; font-style: italic; }
.diff-viewer .hljs-function { color: #61afef; }
.diff-viewer .hljs-title { color: #61afef; }
.diff-viewer .hljs-params { color: #abb2bf; }
.diff-viewer .hljs-type { color: #e5c07b; }
.diff-viewer .hljs-built_in { color: #e06c75; }
.diff-viewer .hljs-literal { color: #56b6c2; }
.diff-viewer .hljs-attr { color: #d19a66; }
.diff-viewer .hljs-selector-class { color: #d19a66; }
.diff-viewer .hljs-selector-tag { color: #e06c75; }
.diff-viewer .hljs-tag { color: #e06c75; }
.diff-viewer .hljs-name { color: #e06c75; }
.diff-viewer .hljs-attribute { color: #d19a66; }
.diff-viewer .hljs-variable { color: #e06c75; }
.diff-viewer .hljs-regexp { color: #98c379; }
.diff-viewer .hljs-symbol { color: #56b6c2; }
.diff-viewer .hljs-meta { color: #5c6370; }
.diff-viewer .hljs-punctuation { color: #abb2bf; }

/* Context menu — unscoped so Teleport works */
.diff-context-menu {
  position: fixed;
  z-index: 9999;
  background: var(--color-surface-raised, #1e1e1e);
  border: 1px solid var(--color-border-default, #333);
  border-radius: 6px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  padding: 4px 0;
  min-width: 180px;
}

.diff-context-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 12px;
  background: none;
  border: none;
  color: var(--color-text-primary, #e0e0e0);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.1s;
}

.diff-context-menu-item:hover {
  background: rgba(20, 184, 166, 0.15);
}
</style>
