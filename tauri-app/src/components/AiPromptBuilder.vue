<script setup lang="ts">
import { ref, computed } from 'vue'
import { ClipboardCopy, FileText, CheckCircle, XCircle } from 'lucide-vue-next'
import { SButton, STextarea, SBadge, SCard, SSectionHeader, SEmptyState } from '@stuntrocket/ui'
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

/** Map severity to SBadge variant */
function severityVariant(severity: string): 'error' | 'warning' | 'info' | 'default' {
  switch (severity) {
    case 'critical': return 'error'
    case 'warning': return 'warning'
    case 'suggestion': return 'info'
    default: return 'default'
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
      <SSectionHeader title="AI Review" />
      <SButton variant="primary" size="sm" @click="handleGenerate">
        <FileText :size="14" />
        {{ hasPrompt ? 'Regenerate Prompt' : 'Generate Prompt' }}
      </SButton>
    </div>

    <!-- Generated Prompt -->
    <SCard v-if="promptVisible && hasPrompt" variant="content">
      <div class="prompt-header">
        <span class="prompt-label">Generated Prompt</span>
        <SButton variant="ghost" size="sm" @click="handleCopy">
          <ClipboardCopy :size="14" />
          Copy to Clipboard
        </SButton>
      </div>
      <pre class="prompt-preview">{{ generatedPrompt }}</pre>
    </SCard>

    <!-- Paste Response -->
    <div v-if="hasPrompt" class="response-section">
      <STextarea
        v-model="pastedResponse"
        label="Paste AI Response"
        placeholder="Paste the AI's review response here..."
        :rows="8"
      />
      <SButton
        variant="secondary"
        size="sm"
        :disabled="!pastedResponse.trim()"
        @click="handleParse"
      >
        Parse Response
      </SButton>
    </div>

    <!-- Parsed Results -->
    <div v-if="parsedResult" class="results-section">
      <!-- Approval Status -->
      <SBadge :variant="parsedResult.approved ? 'success' : 'error'">
        <CheckCircle v-if="parsedResult.approved" :size="14" />
        <XCircle v-else :size="14" />
        {{ parsedResult.approved ? 'Approved' : 'Not Approved' }}
      </SBadge>

      <!-- Summary -->
      <SCard variant="nested">
        <h3 class="result-card-title">Summary</h3>
        <p class="result-card-body">{{ parsedResult.summary }}</p>
      </SCard>

      <!-- Issues -->
      <div v-if="parsedResult.issues.length > 0" class="issues-list">
        <h3 class="issues-title">Issues Found ({{ parsedResult.issues.length }})</h3>
        <div
          v-for="(issue, idx) in parsedResult.issues"
          :key="idx"
          class="issue-item"
        >
          <SBadge :variant="severityVariant(issue.severity)">
            {{ severityLabel(issue.severity) }}
          </SBadge>
          <span class="issue-description">{{ issue.description }}</span>
          <span v-if="issue.file" class="issue-file">{{ issue.file }}</span>
        </div>
      </div>

      <SEmptyState v-else title="No issues found">
        <template #icon>
          <CheckCircle :size="16" />
        </template>
      </SEmptyState>
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

/* Prompt Section */
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

/* Results Section */
.results-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
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
</style>
