<script setup lang="ts">
import { SToastContainer } from '@stuntrocket/ui'

/* Ensure the Pinia toast store is initialised so it shares the same
   useToastStack singleton that SToastContainer renders from. */
import { useToastStore } from '../stores/toast'
useToastStore()
</script>

<template>
  <SToastContainer />
</template>
