import { createRouter, createWebHistory } from 'vue-router'
import LoginView from '../views/LoginView.vue'
import DashboardView from '../views/DashboardView.vue'
import ChangePasswordView from '../views/ChangePasswordView.vue'
import AddAnimalView from '../views/AddAnimalView.vue'
import EditAnimalView from '../views/EditAnimalView.vue'
import AdminManagementView from '../views/AdminManagementView.vue'
import SystemLogsView from '../views/SystemLogsView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'login', component: LoginView },
    { path: '/trocar-senha', name: 'change-password', component: ChangePasswordView, meta: { requiresAuth: true } },
    { path: '/dashboard', name: 'dashboard', component: DashboardView, meta: { requiresAuth: true } },
    { path: '/animais/novo', name: 'add-animal', component: AddAnimalView, meta: { requiresAuth: true } },
    { path: '/animais/editar/:id', name: 'edit-animal', component: EditAnimalView, meta: { requiresAuth: true } },
    { path: '/admins', name: 'admins', component: AdminManagementView, meta: { requiresAuth: true, requiresMaster: true } },
    { path: '/logs', name: 'logs', component: SystemLogsView, meta: { requiresAuth: true, requiresMaster: true } }
  ]
})

router.beforeEach((to, from, next) => {
  const isAuthenticated = !!localStorage.getItem('authToken')
  const isMaster = localStorage.getItem('isMaster') === '1'
  
  if (to.meta.requiresAuth && !isAuthenticated) {
    next('/')
  } else if (to.meta.requiresMaster && !isMaster) {
    next('/dashboard')
  } else {
    next()
  }
})

export default router