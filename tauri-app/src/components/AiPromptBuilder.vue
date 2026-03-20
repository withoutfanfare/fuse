<script setup lang="ts">
import { ref, computed } from 'vue'
import { ClipboardCopy, Sparkles, FileText, CheckCircle, XCircle } from 'lucide-vue-next'
import type { PullRequest, ReviewRule, ParsedAiResponse } from '../types'
import { usePromptBuilder } from '../composables/usePromptBuilder'
import { useResponseParser } from '../composables/useResponseParser'
import { useToastStore } from '../stores/toast'

const props = defineProps<{
  pr: PullRequest
  diff?: string
  rules?: ReviewRule[]
}>()

const toastStore = useToastStore()
const { buildPrompt } = usePromptBuilder()
const { parseResponse } = useResponseParser()

const generatedPrompt = ref('')
const pastedResponse = ref('')
const parsedResult = ref<ParsedAiResponse | null>(null)
const promptVisible = ref(false)

const hasPrompt = computed(() => generatedPrompt.value.length > 0)

function handleGenerate() {
  generatedPrompt.value = buildPrompt(props.pr, {
    diff: props.diff,
    rules: props.rules,
  })
  promptVisible.value = true
  parsedResult.value = null
  pastedResponse.value = ''
}

async function handleCopy() {
  try {
    await navigator.clipboard.writeText(generatedPrompt.value)
    toastStore.addToast('success', 'Copied', 'Prompt copied to clipboard')
  } catch {
    toastStore.addToast('error', 'Copy failed', 'Could not copy to clipboard')
  }
}

function handleParse() {
  if (!pastedResponse.value.trim()) return
  parsedResult.value = parseResponse(pastedResponse.value)
}

function severityClass(severity: string): string {
  switch (severity) {
    case 'critical': return 'severity-critical'
    case 'warning': return 'severity-warning'
    case 'suggestion': return 'severity-suggestion'
    default: return ''
  }
}

function severityLabel(severity: string): string {
  switch (severity) {
    case 'critical': return 'Critical'
    case 'warning': return 'Warning'
    case 'suggestion': return 'Suggestion'
    default: return severity
  }
}
</script>

<template>
  <div class="ai-prompt-builder">
    <div class="builder-header">
      <h2 class="section-title">
        <Sparkles :size="16" />
        AI Review
      </h2>
      <button class="btn-generate" @click="handleGenerate">
        <FileText :size="14" />
        {{ hasPrompt ? 'Regenerate Prompt' : 'Generate Prompt' }}
      </button>
    </div>

    <!-- Generated Prompt -->
    <div v-if="promptVisible && hasPrompt" class="prompt-section">
      <div class="prompt-header">
        <span class="prompt-label">Generated Prompt</span>
        <button class="btn-copy" @click="handleCopy">
          <ClipboardCopy :size="14" />
          Copy to Clipboard
        </button>
      </div>
      <pre class="prompt-preview">{{ generatedPrompt }}</pre>
    </div>

    <!-- Paste Response -->
    <div v-if="hasPrompt" class="response-section">
      <label class="response-label" for="ai-response">Paste AI Response</label>
      <textarea
        id="ai-response"
        v-model="pastedResponse"
        class="response-textarea"
        placeholder="Paste the AI's review response here..."
        rows="8"
      />
      <button
        class="btn-parse"
        :disabled="!pastedResponse.trim()"
        @click="handleParse"
      >
        Parse Response
      </button>
    </div>

    <!-- Parsed Results -->
    <div v-if="parsedResult" class="results-section">
      <!-- Approval Status -->
      <div class="approval-badge" :class="parsedResult.approved ? 'approved' : 'not-approved'">
        <CheckCircle v-if="parsedResult.approved" :size="16" />
        <XCircle v-else :size="16" />
        {{ parsedResult.approved ? 'Approved' : 'Not Approved' }}
      </div>

      <!-- Summary -->
      <div class="result-card">
        <h3 class="result-card-title">Summary</h3>
        <p class="result-card-body">{{ parsedResult.summary }}</p>
      </div>

      <!-- Issues -->
      <div v-if="parsedResult.issues.length > 0" class="issues-list">
        <h3 class="issues-title">Issues Found ({{ parsedResult.issues.length }})</h3>
        <div
          v-for="(issue, idx) in parsedResult.issues"
          :key="idx"
          class="issue-item"
        >
          <span class="severity-badge" :class="severityClass(issue.severity)">
            {{ severityLabel(issue.severity) }}
          </span>
          <span class="issue-description">{{ issue.description }}</span>
          <span v-if="issue.file" class="issue-file">{{ issue.file }}</span>
        </div>
      </div>

      <div v-else class="no-issues">
        <CheckCircle :size="16" />
        No issues found
      </div>
    </div>
  </div>
</template>

<style scoped>
.ai-prompt-builder {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.builder-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.section-title {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin: 0;
}

.btn-generate {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1-5);
  background: var(--color-accent-muted);
  color: var(--color-accent);
  font-weight: 600;
  font-size: 13px;
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  border: 1px solid rgba(20, 184, 166, 0.3);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-generate:hover {
  background: rgba(20, 184, 166, 0.3);
  border-color: rgba(20, 184, 166, 0.5);
}

.btn-generate:active {
  transform: scale(0.97);
}

.btn-generate:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

/* Prompt Section */
.prompt-section {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  overflow: hidden;
}

.prompt-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--color-border-default);
}

.prompt-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.btn-copy {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  background: var(--color-surface-raised);
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 500;
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-border-default);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-copy:hover {
  background: var(--color-surface-hover);
  color: var(--color-text-primary);
  border-color: var(--color-border-hover);
}

.btn-copy:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.prompt-preview {
  margin: 0;
  padding: var(--space-4);
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.6;
  color: var(--color-text-secondary);
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 320px;
  overflow-y: auto;
}

/* Response Section */
.response-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.response-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.response-textarea {
  background: var(--color-surface-input);
  color: var(--color-text-primary);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-3);
  font-family: var(--font-mono);
  font-size: 13px;
  line-height: 1.5;
  resize: vertical;
  transition: border-color var(--transition-fast);
}

.response-textarea::placeholder {
  color: var(--color-text-muted);
}

.response-textarea:focus {
  outline: none;
  border-color: var(--color-border-focus);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
}

.btn-parse {
  align-self: flex-start;
  background: var(--color-surface-raised);
  color: var(--color-text-primary);
  font-weight: 600;
  font-size: 13px;
  padding: var(--space-2) var(--space-4);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-default);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-parse:hover:not(:disabled) {
  background: var(--color-surface-hover);
  border-color: var(--color-border-hover);
}

.btn-parse:active:not(:disabled) {
  transform: scale(0.97);
}

.btn-parse:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-parse:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

/* Results Section */
.results-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.approval-badge {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1-5);
  align-self: flex-start;
  font-weight: 600;
  font-size: 13px;
  padding: var(--space-1-5) var(--space-3);
  border-radius: var(--radius-full);
}

.approval-badge.approved {
  background: rgba(34, 197, 94, 0.2);
  color: var(--color-status-success);
  border: 1px solid rgba(34, 197, 94, 0.3);
}

.approval-badge.not-approved {
  background: rgba(220, 38, 38, 0.2);
  color: var(--color-status-danger);
  border: 1px solid rgba(220, 38, 38, 0.3);
}

.result-card {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-4);
}

.result-card-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-muted);
  margin: 0 0 var(--space-2) 0;
}

.result-card-body {
  font-size: 13px;
  line-height: 1.6;
  color: var(--color-text-secondary);
  margin: 0;
}

/* Issues */
.issues-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.issues-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-muted);
  margin: 0;
}

.issue-item {
  display: flex;
  align-items: flex-start;
  gap: var(--space-2);
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-3);
}

.severity-badge {
  flex-shrink: 0;
  font-size: 11px;
  font-weight: 600;
  padding: 1px var(--space-2);
  border-radius: var(--radius-full);
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.severity-critical {
  background: rgba(220, 38, 38, 0.2);
  color: var(--color-status-danger);
}

.severity-warning {
  background: rgba(234, 179, 8, 0.2);
  color: var(--color-status-warning);
}

.severity-suggestion {
  background: rgba(59, 130, 246, 0.2);
  color: var(--color-status-info);
}

.issue-description {
  font-size: 13px;
  line-height: 1.5;
  color: var(--color-text-secondary);
}

.issue-file {
  flex-shrink: 0;
  margin-left: auto;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--color-text-muted);
  background: var(--color-surface-raised);
  padding: 1px var(--space-2);
  border-radius: var(--radius-sm);
}

.no-issues {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: 13px;
  color: var(--color-status-success);
  padding: var(--space-3);
  background: rgba(34, 197, 94, 0.1);
  border-radius: var(--radius-md);
  border: 1px solid rgba(34, 197, 94, 0.2);
}
</style>
