<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { usePullRequestsStore } from '../stores/pullRequests'
import type { AuthorStats } from '../types'
import AuthorStatsTable from '../components/AuthorStatsTable.vue'
import ContentLoader from '../components/ContentLoader.vue'

const prStore = usePullRequestsStore()
const authors = ref<AuthorStats[]>([])
const loading = ref(true)

onMounted(async () => {
  authors.value = await prStore.fetchAuthorStats()
  loading.value = false
})
</script>

<template>
  <div class="authors-view">
    <h1 class="page-title">Author Performance</h1>
    <p class="page-description">Overview of contributor activity across all tracked repositories.</p>

    <ContentLoader v-if="loading" variant="list" :count="5" />
    <AuthorStatsTable v-else :authors="authors" />
  </div>
</template>

<style scoped>
.authors-view {
  width: 100%;
}

.page-title {
  font-size: 22px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: var(--space-1);
}

.page-description {
  font-size: 14px;
  color: var(--color-text-muted);
  margin-bottom: var(--space-6);
}

</style>
