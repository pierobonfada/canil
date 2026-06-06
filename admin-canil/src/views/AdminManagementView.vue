<template>
  <div class="master-container">
    <header class="top-bar">
      <h2>👑 Gerenciamento de Administradores</h2>
      <button @click="router.push('/dashboard')" class="btn-back">⬅️ Voltar</button>
    </header>

    <div class="card add-admin-card">
      <h3>➕ Novo Administrador</h3>
      <form @submit.prevent="createAdmin" class="form-row">
        <input v-model="newAdmin.name" placeholder="Nome completo" required />
        <input v-model="newAdmin.email" type="email" placeholder="E-mail" required />
        <input v-model="newAdmin.phone" placeholder="Telefone" required />
        <label>
          <input type="checkbox" v-model="newAdmin.is_master" /> É Master?
        </label>
        <button type="submit" class="btn-submit">Criar</button>
      </form>
    </div>

    <div class="card list-card">
      <h3>📋 Administradores Cadastrados</h3>
      <table class="data-table">
        <thead>
          <tr>
            <th>ID</th>
            <th>Nome</th>
            <th>E-mail</th>
            <th>Master?</th>
            <th>Status</th>
            <th>Ações</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="admin in admins" :key="admin.id" :class="{'inactive-row': !admin.is_active}">
            <td>{{ admin.id }}</td>
            <td>{{ admin.name }}</td>
            <td>{{ admin.email }}</td>
            <td>{{ admin.is_master ? 'Sim' : 'Não' }}</td>
            <td>
              <span v-if="admin.is_locked" class="badge-black">🔒 Bloqueado</span>
              <span v-else :class="admin.is_active ? 'badge-green' : 'badge-red'">
                {{ admin.is_active ? 'Ativo' : 'Inativo' }}
              </span>
            </td>
            <td>
              <button v-if="admin.email !== 'master@master.master'" @click="toggleStatus(admin)" class="btn-sm btn-warn">
                {{ admin.is_active ? 'Desativar' : 'Reativar' }}
              </button>
              <button v-if="admin.email !== 'master@master.master'" @click="toggleMaster(admin)" class="btn-sm btn-purple">
                {{ admin.is_master ? 'Remover Master' : 'Tornar Master' }}
              </button>
              <button v-if="admin.is_locked" @click="unlockAccount(admin)" class="btn-sm btn-green">
                🔓 Desbloquear
              </button>
              <button @click="forcePasswordReset(admin)" class="btn-sm btn-danger">
                Resetar Senha
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import api from '../services/api'

const router = useRouter()
const admins = ref<any[]>([])

const newAdmin = ref({
  name: '',
  email: '',
  phone: '',
  is_master: false
})

const fetchAdmins = async () => {
  try {
    const res = await api.get('/admins')
    admins.value = res.data
  } catch (error) {
    alert("Erro ao buscar administradores")
  }
}

const createAdmin = async () => {
  try {
    const res = await api.post('/admins', newAdmin.value)
    alert(res.data)
    newAdmin.value = { name: '', email: '', phone: '', is_master: false }
    fetchAdmins()
  } catch (error: any) {
    alert(error.response?.data?.error || "Erro ao criar admin")
  }
}

const toggleStatus = async (admin: any) => {
  try {
    await api.patch(`/admins/${admin.id}/status`, { is_active: !admin.is_active })
    fetchAdmins()
  } catch (error: any) {
    alert(error.response?.data?.error || "Erro ao atualizar")
  }
}

const toggleMaster = async (admin: any) => {
  try {
    await api.patch(`/admins/${admin.id}/status`, { is_master: !admin.is_master })
    fetchAdmins()
  } catch (error: any) {
    alert(error.response?.data?.error || "Erro ao atualizar")
  }
}

const unlockAccount = async (admin: any) => {
  if (!confirm(`Tem certeza que deseja desbloquear a conta de ${admin.name}?`)) return;
  try {
    const res = await api.patch(`/admins/${admin.id}/status`, { is_locked: false })
    alert(res.data)
    fetchAdmins()
  } catch (error: any) {
    alert(error.response?.data?.error || "Erro ao desbloquear")
  }
}

const forcePasswordReset = async (admin: any) => {
  if (!confirm(`Tem certeza que deseja forçar o reset de senha para ${admin.name}?`)) return;
  try {
    const res = await api.patch(`/admins/${admin.id}/status`, { force_password_reset: true })
    alert(res.data)
  } catch (error: any) {
    alert(error.response?.data?.error || "Erro ao atualizar")
  }
}

onMounted(() => {
  fetchAdmins()
})
</script>

<style scoped>
.master-container { padding: 2rem; max-width: 1200px; margin: auto; }
.top-bar { display: flex; justify-content: space-between; margin-bottom: 2rem; }
.btn-back { background: #6b7280; color: white; padding: 0.5rem 1rem; border: none; border-radius: 4px; cursor: pointer; }
.card { background: white; padding: 1.5rem; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); margin-bottom: 2rem; }
.form-row { display: flex; gap: 1rem; align-items: center; }
.form-row input { padding: 0.5rem; border: 1px solid #ccc; border-radius: 4px; }
.btn-submit { background: #10b981; color: white; padding: 0.5rem 1rem; border: none; border-radius: 4px; cursor: pointer; }
.data-table { width: 100%; border-collapse: collapse; }
.data-table th, .data-table td { padding: 0.75rem; text-align: left; border-bottom: 1px solid #eee; }
.inactive-row { background-color: #fef2f2; color: #666; }
.badge-green { background: #d1fae5; color: #065f46; padding: 0.2rem 0.5rem; border-radius: 99px; font-size: 0.8rem; }
.badge-red { background: #fee2e2; color: #991b1b; padding: 0.2rem 0.5rem; border-radius: 99px; font-size: 0.8rem; }
.badge-black { background: #1f2937; color: #ffffff; padding: 0.2rem 0.5rem; border-radius: 99px; font-size: 0.8rem; display: inline-block; }
.btn-sm { margin-right: 0.5rem; padding: 0.3rem 0.6rem; border: none; border-radius: 4px; cursor: pointer; font-size: 0.8rem; color: white; margin-bottom: 0.2rem; }
.btn-warn { background: #f59e0b; }
.btn-purple { background: #8b5cf6; }
.btn-danger { background: #ef4444; }
.btn-green { background: #10b981; }
</style>
