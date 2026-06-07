import { createRouter, createWebHistory } from 'vue-router'

// ==========================================
// ROTEAMENTO (VUE ROUTER)
// ==========================================
// O Vue Router gerencia a navegação Client-Side (SPA - Single Page Application).
// Diferente de sites tradicionais onde cada clique recarrega a página inteira baixando
// HTML do servidor, aqui o Vue apenas troca os componentes na tela dinamicamente.

import HomeView from '../views/HomeView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView
    },
    {
      path: '/animal/:id',
      name: 'animal',
      component: () => import('../views/AnimalView.vue')
    }
  ]
})

export default router
