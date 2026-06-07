import { createRouter, createWebHistory } from 'vue-router'

// ==========================================
// ROTEAMENTO E PROTEÇÃO DE ROTAS (ROUTE GUARDS)
// ==========================================
// Define as rotas do painel. Rotas protegidas possuem um objeto 'meta' 
// indicando os níveis de permissão necessários (ex: requiresAuth, requiresMaster).
// Usamos o 'router.beforeEach' para interceptar todas as mudanças de página
// e bloquear acessos indevidos antes mesmo do componente ser carregado.

import LoginView from '../views/LoginView.vue'
import DashboardView from '../views/DashboardView.vue'
import ChangePasswordView from '../views/ChangePasswordView.vue'
import AddAnimalView from '../views/AddAnimalView.vue'
import EditAnimalView from '../views/EditAnimalView.vue'
import AdminManagementView from '../views/AdminManagementView.vue'
import SystemLogsView from '../views/SystemLogsView.vue'
import AnalyticsView from '../views/AnalyticsView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'login', component: LoginView },
    { path: '/trocar-senha', name: 'change-password', component: ChangePasswordView, meta: { requiresAuth: true } },
    { path: '/dashboard', name: 'dashboard', component: DashboardView, meta: { requiresAuth: true } },
    { path: '/animais/novo', name: 'add-animal', component: AddAnimalView, meta: { requiresAuth: true } },
    { path: '/animais/editar/:id', name: 'edit-animal', component: EditAnimalView, meta: { requiresAuth: true } },
    { path: '/analytics', name: 'analytics', component: AnalyticsView, meta: { requiresAuth: true } },
    { path: '/admins', name: 'admins', component: AdminManagementView, meta: { requiresAuth: true, requiresMaster: true } },
    { path: '/logs', name: 'logs', component: SystemLogsView, meta: { requiresAuth: true, requiresMaster: true } }
  ]
})

// ROUTE GUARD GLOBAL
// Esta função é chamada toda vez que o usuário tenta mudar de URL.
router.beforeEach((to, from, next) => {
  // Verifica se existe um token salvo (o que indica que o usuário logou)
  const isAuthenticated = !!localStorage.getItem('authToken')
  // Verifica se o usuário tem privilégios Master
  const isMaster = localStorage.getItem('isMaster') === '1'
  
  // Se a rota exige login e o usuário não está logado, manda pro login '/'
    next('/')
  } else if (to.meta.requiresMaster && !isMaster) {
    next('/dashboard')
  } else {
    next()
  }
})

export default router