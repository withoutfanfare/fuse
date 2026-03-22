<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { FolderGit2 } from 'lucide-vue-next'
import { useRepositoriesStore } from '../stores/repositories'
import { usePullRequestsStore } from '../stores/pullRequests'
import { useGroupsStore } from '../stores/groups'
import { useConfirm, SEmptyState, SInput, SButton } from '@stuntrocket/ui'
import RepositoryCard from '../components/RepositoryCard.vue'
import ReviewRulesEditor from '../components/ReviewRulesEditor.vue'
import GroupManager from '../components/GroupManager.vue'
import ContentLoader from '../components/ContentLoader.vue'

const repoStore = useRepositoriesStore()
const prStore = usePullRequestsStore()
const groupsStore = useGroupsStore()
const { confirm } = useConfirm()

const initialLoad = ref(true)
const newOwner = ref('')
const newName = ref('')
const newBranch = ref('main')
const adding = ref(false)
const addError = ref<string | null>(null)
const rulesOpenForRepo = ref<number | null>(null)

function toggleRules(repoId: number) {
  rulesOpenForRepo.value = rulesOpenForRepo.value === repoId ? null : repoId
}

onMounted(async () => {
  if (repoStore.repos.length === 0) await repoStore.fetchAll()
  await groupsStore.fetchAll()
  initialLoad.value = false
})

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

async function removeRepo(id: number) {
  const repo = repoStore.repos.find(r => r.id === id)
  const label = repo ? `${repo.owner}/${repo.name}` : 'this repository'
  const confirmed = await confirm({
    title: 'Remove Repository',
    message: `Stop tracking ${label}?`,
    confirmLabel: 'Remove',
    danger: true,
  })
  if (!confirmed) return
  await repoStore.remove(id)
}

async function syncRepo(id: number) {
  await prStore.syncAll(id)
}

async function updateBranch(id: number, branch: string) {
  try {
    await repoStore.updateBranch(id, branch)
  } catch {
    // Error is already set on the store
  }
}

/** Repos sorted by most recently added first */
const sortedRepos = computed(() =>
  [...repoStore.repos].sort((a, b) => new Date(b.added_at).getTime() - new Date(a.added_at).getTime())
)

function prCountForRepo(repoId: number): number {
  return prStore.prs.filter(pr => pr.repo_id === repoId && pr.state === 'OPEN').length
}
</script>

<template>
  <div class="repositories-view">
    <section class="add-repo-section">
      <h2 class="section-title">Add Repository</h2>
      <form class="add-repo-form" @submit.prevent="addRepo">
        <SInput
          :model-value="newOwner"
          placeholder="Owner (e.g. bemanza)"
          size="sm"
          required
          @update:model-value="newOwner = $event"
        />
        <span class="slash">/</span>
        <SInput
          :model-value="newName"
          placeholder="Repository (e.g. my-project)"
          size="sm"
          required
          @update:model-value="newName = $event"
        />
        <SInput
          :model-value="newBranch"
          placeholder="Default branch"
          size="sm"
          class="input-branch"
          @update:model-value="newBranch = $event"
        />
        <SButton variant="primary" size="sm" :loading="adding" type="submit">
          {{ adding ? 'Adding…' : 'Add' }}
        </SButton>
      </form>
      <div v-if="addError" class="add-error">{{ addError }}</div>
    </section>

    <section class="groups-section">
      <GroupManager />
    </section>

    <section class="repos-section">
      <h2 class="section-title">Tracked Repositories</h2>
      <ContentLoader v-if="initialLoad || repoStore.loading" variant="cards" :count="3" />
      <div v-else-if="repoStore.repos.length > 0" class="repos-grid">
        <div v-for="repo in sortedRepos" :key="repo.id" class="repo-wrapper">
          <RepositoryCard
            :repo="repo"
            :pr-count="prCountForRepo(repo.id)"
            :groups="groupsStore.getGroupsForRepo(repo.id)"
            @remove="removeRepo"
            @sync="syncRepo"
            @update-branch="updateBranch"
          />
          <div class="repo-extras">
            <button
              class="btn-toggle-rules"
              :class="{ active: rulesOpenForRepo === repo.id }"
              @click="toggleRules(repo.id)"
            >
              {{ rulesOpenForRepo === repo.id ? 'Hide Review Rules' : 'Review Rules' }}
            </button>
          </div>
          <ReviewRulesEditor
            v-if="rulesOpenForRepo === repo.id"
            :repo-id="repo.id"
          />
        </div>
      </div>
      <SEmptyState
        v-else
        title="No repositories tracked"
        description="Add a repository above to start tracking pull requests."
      >
        <template #icon><FolderGit2 :size="36" /></template>
      </SEmptyState>
    </section>
  </div>
</template>

<style scoped>
.repositories-view {
  width: 100%;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: var(--space-4);
  color: var(--color-text-primary);
}

.add-repo-section {
  margin-bottom: var(--space-8);
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-5);
}

.add-repo-form {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.input-branch {
  max-width: 150px;
}

.slash {
  color: var(--color-text-muted);
  font-size: 18px;
}

.add-error {
  margin-top: var(--space-3);
  color: var(--color-status-danger);
  font-size: 13px;
  background: rgba(220, 38, 38, 0.1);
  padding: var(--space-3);
  border-radius: var(--radius-sm);
  border: 1px solid rgba(220, 38, 38, 0.2);
}

.groups-section {
  margin-bottom: var(--space-8);
}

.repos-grid {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.repo-wrapper {
  display: flex;
  flex-direction: column;
}

.repo-extras {
  display: flex;
  justify-content: flex-end;
  padding-top: var(--space-2);
}

.btn-toggle-rules {
  background: var(--color-surface-raised);
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 500;
  padding: var(--space-1-5) var(--space-3);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-default);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-toggle-rules:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-border-hover);
  color: var(--color-text-primary);
}

.btn-toggle-rules.active {
  background: var(--color-accent-muted);
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.btn-toggle-rules:active {
  transform: scale(0.97);
}

.btn-toggle-rules:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

</style>
