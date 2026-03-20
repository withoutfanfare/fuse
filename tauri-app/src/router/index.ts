import type { RouteRecordRaw } from 'vue-router'

export const routes: RouteRecordRaw[] = [
  { path: '/', redirect: '/dashboard' },
  { path: '/dashboard', name: 'dashboard', component: () => import('../views/Dashboard.vue') },
  { path: '/prs', name: 'pull-requests', component: () => import('../views/PullRequests.vue') },
  { path: '/prs/:id', name: 'pr-detail', component: () => import('../views/PullRequestDetail.vue'), props: true },
  { path: '/review-session/:prId', name: 'review-session', component: () => import('../views/ReviewSession.vue'), props: true },
  { path: '/repositories', name: 'repositories', component: () => import('../views/Repositories.vue') },
  { path: '/authors', name: 'authors', component: () => import('../views/Authors.vue') },
  { path: '/digest', name: 'digest', component: () => import('../views/DigestView.vue') },
  { path: '/bookmarks', name: 'bookmarks', component: () => import('../views/BookmarksView.vue') },
  { path: '/settings', name: 'settings', component: () => import('../views/Settings.vue') },
]
