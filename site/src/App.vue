<!-- ========================================== -->
<!-- 🏡 A VITRINE PRINCIPAL (App.vue do Site) -->
<!-- ========================================== -->
<!-- O <RouterView /> vai renderizar a Home, a página de Contato ou o Perfil do Animal
dependendo do que o visitante clicar! -->
<template>
  <div class="app-layout">
    <router-view></router-view>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useRouter } from 'vue-router';
import api from './services/api';

const router = useRouter();

onMounted(() => {
  let visitorId = localStorage.getItem('visitor_id');
  if (!visitorId) {
    visitorId = 'v_' + Math.random().toString(36).substr(2, 9) + Date.now().toString(36);
    localStorage.setItem('visitor_id', visitorId);
  }

  router.afterEach((to) => {
    api.post('/public/analytics', {
      visitor_id: visitorId,
      event_type: 'page_view',
      path: to.fullPath,
      animal_id: to.name === 'AnimalDetail' ? Number(to.params.id) : null,
      payload: null
    }).catch(() => {});
  });
});
</script>

<style scoped>
.app-layout {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}
</style>
