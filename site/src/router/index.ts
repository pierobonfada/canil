import { createRouter, createWebHistory } from 'vue-router'

// ==========================================
// 🗺️ O MAPA DO SITE (ROUTER)
// ==========================================
// Pense no Router como o GPS do nosso site. É ele que decide para qual página
// o visitante vai ser levado quando clica num link.
// Por exemplo: se ele acessar "/", ele vai pra Home. Se acessar "/animal/1", vai ver o perfil do Totó!

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
