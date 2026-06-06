import { createRouter, createWebHistory } from 'vue-router'
import LoginView from '../views/LoginView.vue'
import DashboardView from '../views/DashboardView.vue'
import ChangePasswordView from '../views/ChangePasswordView.vue'
import AddAnimalView from '../views/AddAnimalView.vue'
import EditAnimalView from '../views/EditAnimalView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'login', component: LoginView },
    { path: '/trocar-senha', name: 'change-password', component: ChangePasswordView, meta: { requiresAuth: true } },
    { path: '/dashboard', name: 'dashboard', component: DashboardView, meta: { requiresAuth: true } },
    { path: '/animais/novo', name: 'add-animal', component: AddAnimalView, meta: { requiresAuth: true } },
    { path: '/animais/editar/:id', name: 'edit-animal', component: EditAnimalView, meta: { requiresAuth: true } }
  ]
})

router.beforeEach((to, from, next) => {
  const isAuthenticated = !!localStorage.getItem('authToken')
  if (to.meta.requiresAuth && !isAuthenticated) next('/')
  else next()
})

export default router