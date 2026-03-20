<script setup lang="ts">
import { ref } from 'vue'
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
  <Transition name="overlay-fade">
    <div class="onboarding-overlay" @click.self="dismissOnboarding">
      <div class="onboarding-panel">
        <button class="dismiss-btn" @click="dismissOnboarding" title="Skip onboarding">&times;</button>

        <!-- Step indicators -->
        <div class="step-dots">
          <span
            v-for="n in 3"
            :key="n"
            class="dot"
            :class="{ active: currentStep === n - 1, completed: currentStep > n - 1 }"
          />
        </div>

        <!-- Step 0: Welcome -->
        <div v-if="currentStep === 0" class="step-content">
          <h2 class="step-title">Welcome to PR Review Companion</h2>
          <p class="step-description">
            Track, review, and manage pull requests across all your GitHub repositories
            from a single dashboard. Let's get you set up in a few quick steps.
          </p>
          <div class="step-actions">
            <button class="btn-secondary" @click="dismissOnboarding">Skip</button>
            <button class="btn-primary" @click="next">Get Started</button>
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
              <input
                v-model="newOwner"
                placeholder="Owner (e.g. facebook)"
                class="input-field"
                required
              />
              <span class="slash">/</span>
              <input
                v-model="newName"
                placeholder="Repository (e.g. react)"
                class="input-field"
                required
              />
            </div>
            <div class="form-row">
              <input
                v-model="newBranch"
                placeholder="Default branch"
                class="input-field input-branch"
              />
              <button type="submit" class="btn-primary" :disabled="adding">
                {{ adding ? 'Adding...' : 'Add Repository' }}
              </button>
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
            <button class="btn-secondary" @click="back">Back</button>
            <button class="btn-primary" :disabled="repoStore.repos.length === 0" @click="next">
              Continue
            </button>
          </div>
        </div>

        <!-- Step 2: Initial Sync -->
        <div v-else-if="currentStep === 2" class="step-content">
          <h2 class="step-title">Sync Pull Requests</h2>
          <p class="step-description">
            Fetch open pull requests from your repositories. This may take a moment.
          </p>

          <div v-if="!syncDone" class="sync-area">
            <button
              class="btn-primary btn-sync-action"
              :disabled="syncing"
              @click="runSync"
            >
              {{ syncing ? 'Syncing...' : 'Sync Now' }}
            </button>
            <div v-if="syncing" class="sync-progress">
              <div class="spinner" />
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
            <button class="btn-secondary" @click="back">Back</button>
            <button class="btn-primary" @click="completeOnboarding">
              {{ syncDone ? 'Done' : 'Skip & Finish' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.onboarding-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.6);
}

.onboarding-panel {
  position: relative;
  width: 520px;
  max-width: 90vw;
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-overlay);
  padding: var(--space-8);
}

.dismiss-btn {
  position: absolute;
  top: var(--space-4);
  right: var(--space-4);
  background: none;
  border: none;
  color: var(--color-text-muted);
  font-size: 22px;
  line-height: 1;
  cursor: pointer;
  padding: var(--space-1);
  border-radius: var(--radius-sm);
  transition: color var(--transition-fast);
}

.dismiss-btn:hover {
  color: var(--color-text-primary);
}

.step-dots {
  display: flex;
  justify-content: center;
  gap: var(--space-2);
  margin-bottom: var(--space-6);
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

.input-field {
  flex: 1;
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-3);
  color: var(--color-text-primary);
  font-size: 13px;
  transition: border-color var(--transition-fast);
}

.input-field:focus {
  border-color: var(--color-border-focus);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.input-field::placeholder {
  color: var(--color-text-muted);
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

.btn-primary {
  background: var(--color-accent);
  color: var(--color-text-inverse);
  font-weight: 600;
  padding: var(--space-2) var(--space-5);
  border-radius: var(--radius-md);
  border: none;
  cursor: pointer;
  font-size: 13px;
  transition: background var(--transition-fast), transform var(--transition-fast);
}

.btn-primary:hover:not(:disabled) {
  background: var(--color-accent-hover);
}

.btn-primary:active:not(:disabled) {
  transform: scale(0.97);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background: var(--color-surface-raised);
  color: var(--color-text-secondary);
  font-weight: 500;
  padding: var(--space-2) var(--space-4);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-default);
  cursor: pointer;
  font-size: 13px;
  transition: all var(--transition-fast);
}

.btn-secondary:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-border-hover);
  color: var(--color-text-primary);
}

.sync-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-4);
}

.btn-sync-action {
  padding: var(--space-3) var(--space-8);
  font-size: 14px;
}

.sync-progress {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  font-size: 13px;
  color: var(--color-text-secondary);
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--color-border-default);
  border-top-color: var(--color-accent);
  border-radius: var(--radius-full);
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.sync-complete {
  text-align: center;
}

.sync-result {
  font-size: 14px;
  color: var(--color-status-success);
  font-weight: 500;
}

.overlay-fade-enter-active,
.overlay-fade-leave-active {
  transition: opacity var(--transition-normal);
}

.overlay-fade-enter-from,
.overlay-fade-leave-to {
  opacity: 0;
}
</style>
