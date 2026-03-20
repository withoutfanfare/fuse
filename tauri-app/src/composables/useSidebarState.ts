import { ref, watch } from 'vue'

const collapsed = ref(localStorage.getItem('sidebar-collapsed') === 'true')

watch(collapsed, (val) => {
  localStorage.setItem('sidebar-collapsed', String(val))
})

export function useSidebarState() {
  const toggle = () => { collapsed.value = !collapsed.value }
  return { collapsed, toggle }
}
