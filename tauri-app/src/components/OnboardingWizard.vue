<script setup lang="ts">
import { ref } from 'vue'
import { SModal, SButton, SInput } from '@stuntrocket/ui'
import { useRepositoriesStore } from '../stores/repositories'
import { usePullRequestsStore } from '../stores/pullRequests'
import { useOnboarding } from '../composables/useOnboarding'

const repoStore = useRepositoriesStore()
const prStore = usePullRequestsStore()
const { currentStep, dismissOnboarding, completeOnboarding } = useOnboarding()

const newOwner = ref('')
const newName = ref('')
const newBranch = ref('main')
const adding = ref(false)
const addError = ref<string | null>(null)
const syncing = ref(false)
const syncDone = ref(false)

async function addRepo() {
  if (!newOwner.value.trim() || !newName.value.trim()) return
  adding.value = true
  addError.value = null
  try {
    await repoStore.add(newOwner.value.trim(), newName.value.trim(), newBranch.value.trim() || undefined)
    newOwner.value = ''
    newName.value = ''
    newBranch.value = 'main'
  } catch (e) {
    addError.value = String(e)
  } finally {
    adding.value = false
  }
}

async function runSync() {
  syncing.value = true
  await prStore.syncAll()
  syncing.value = false
  syncDone.value = true
}

function next() {
  if (currentStep.value < 2) {
    currentStep.value++
  }
}

function back() {
  if (currentStep.value > 0) {
    currentStep.value--
  }
}
</script>

<template>
  <SModal
    :open="true"
    max-width="520px"
    @close="dismissOnboarding"
  >
    <template #header>
      <!-- Step indicators -->
      <div class="step-dots">
        <span
          v-for="n in 3"
          :key="n"
          class="dot"
          :class="{ active: currentStep === n - 1, completed: currentStep > n - 1 }"
        />
      </div>
    </template>

    <!-- Step 0: Welcome -->
    <div v-if="currentStep === 0" class="step-content">
      <h2 class="step-title">Welcome to PR Review Companion</h2>
      <p class="step-description">
        Track, review, and manage pull requests across all your GitHub repositories
        from a single dashboard. Let's get you set up in a few quick steps.
      </p>
      <div class="step-actions">
        <SButton variant="secondary" @click="dismissOnboarding">Skip</SButton>
        <SButton variant="primary" @click="next">Get Started</SButton>
      </div>
    </div>

    <!-- Step 1: Add Repository -->
    <div v-else-if="currentStep === 1" class="step-content">
      <h2 class="step-title">Add a Repository</h2>
      <p class="step-description">
        Enter a GitHub repository you'd like to track. You can always add more later.
      </p>

      <form class="onboarding-form" @submit.prevent="addRepo">
        <div class="form-row">
          <SInput
            v-model="newOwner"
            placeholder="Owner (e.g. facebook)"
          />
          <span class="slash">/</span>
          <SInput
            v-model="newName"
            placeholder="Repository (e.g. react)"
          />
        </div>
        <div class="form-row">
          <SInput
            v-model="newBranch"
            placeholder="Default branch"
            class="input-branch"
          />
          <SButton variant="primary" :disabled="adding" @click="addRepo">
            {{ adding ? 'Adding...' : 'Add Repository' }}
          </SButton>
        </div>
      </form>
      <div v-if="addError" class="add-error">{{ addError }}</div>

      <div v-if="repoStore.repos.length > 0" class="added-repos">
        <span class="added-label">Added:</span>
        <span v-for="repo in repoStore.repos" :key="repo.id" class="added-chip">
          {{ repo.owner }}/{{ repo.name }}
        </span>
      </div>

      <div class="step-actions">
        <SButton variant="secondary" @click="back">Back</SButton>
        <SButton variant="primary" :disabled="repoStore.repos.length === 0" @click="next">
          Continue
        </SButton>
      </div>
    </div>

    <!-- Step 2: Initial Sync -->
    <div v-else-if="currentStep === 2" class="step-content">
      <h2 class="step-title">Sync Pull Requests</h2>
      <p class="step-description">
        Fetch open pull requests from your repositories. This may take a moment.
      </p>

      <div v-if="!syncDone" class="sync-area">
        <SButton
          variant="primary"
          size="lg"
          :disabled="syncing"
          :loading="syncing"
          @click="runSync"
        >
          {{ syncing ? 'Syncing...' : 'Sync Now' }}
        </SButton>
        <div v-if="syncing" class="sync-progress">
          <span>Fetching pull requests...</span>
        </div>
      </div>

      <div v-else class="sync-complete">
        <p class="sync-result">
          Found {{ prStore.prs.length }} pull request{{ prStore.prs.length === 1 ? '' : 's' }}.
          You're all set!
        </p>
      </div>

      <div class="step-actions">
        <SButton variant="secondary" @click="back">Back</SButton>
        <SButton variant="primary" @click="completeOnboarding">
          {{ syncDone ? 'Done' : 'Skip & Finish' }}
        </SButton>
      </div>
    </div>
  </SModal>
</template>

<style scoped>
.step-dots {
  display: flex;
  justify-content: center;
  gap: var(--space-2);
  margin-bottom: var(--space-2);
  width: 100%;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: var(--radius-full);
  background: var(--color-border-default);
  transition: all var(--transition-fast);
}

.dot.active {
  background: var(--color-accent);
  width: 24px;
}

.dot.completed {
  background: var(--color-accent);
}

.step-content {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.step-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--color-text-primary);
  text-align: center;
}

.step-description {
  font-size: 14px;
  color: var(--color-text-secondary);
  text-align: center;
  line-height: 1.6;
}

.onboarding-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.form-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.input-branch {
  max-width: 160px;
}

.slash {
  color: var(--color-text-muted);
  font-size: 18px;
}

.add-error {
  color: var(--color-status-danger);
  font-size: 13px;
  background: rgba(220, 38, 38, 0.1);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-sm);
  border: 1px solid rgba(220, 38, 38, 0.2);
}

.added-repos {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.added-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
}

.added-chip {
  font-size: 12px;
  padding: var(--space-1) var(--space-2);
  background: var(--color-accent-muted);
  color: var(--color-accent);
  border-radius: var(--radius-full);
  font-weight: 500;
}

.step-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-3);
  margin-top: var(--space-4);
}

.sync-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-4);
}

.sync-progress {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  font-size: 13px;
  color: var(--color-text-secondary);
}

.sync-complete {
  text-align: center;
}

.sync-result {
  font-size: 14px;
  color: var(--color-status-success);
  font-weight: 500;
}
</style>
