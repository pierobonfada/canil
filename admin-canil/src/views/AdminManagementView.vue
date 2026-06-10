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
        <input :value="newAdmin.phone" @input="(e) => newAdmin.phone = formatPhone((e.target as HTMLInputElement).value)" placeholder="Telefone" required />
        <label>
          <input type="checkbox" v-model="newAdmin.is_master" /> É Master?
        </label>
        <button type="submit" class="btn-submit">Criar</button>
      </form>
    </div>

    <div class="card list-card">
      <h3>📋 Administradores Cadastrados</h3>
      <div class="table-responsive">
        <table class="data-table">
          <thead>
            <tr>
              <th>ID</th>
              <th>Nome</th>
              <th>E-mail</th>
              <th>Telefone</th>
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
              <td>{{ formatPhone(admin.phone) }}</td>
              <td>{{ admin.is_master ? 'Sim' : 'Não' }}</td>
              <td>
                <span v-if="admin.is_locked" class="badge-black">🔒 Bloqueado</span>
                <span v-else :class="admin.is_active ? 'badge-green' : 'badge-red'">
                  {{ admin.is_active ? 'Ativo' : 'Inativo' }}
                </span>
              </td>
              <td class="actions-cell">
                <button @click="openEditModal(admin)" class="btn-sm btn-blue">
                  ✏️ Editar
                </button>
                <button v-if="admin.email !== 'master@master.master'" @click="toggleStatus(admin)" class="btn-sm btn-warn">
                  {{ admin.is_active ? 'Desativar' : 'Reativar' }}
                </button>
                <button v-if="admin.email !== 'master@master.master'" @click="toggleMaster(admin)" class="btn-sm btn-purple">
                  {{ admin.is_master ? 'Remover Master' : 'Tornar Master' }}
                </button>
                <button v-if="admin.is_locked" @click="unlockAccount(admin)" class="btn-sm btn-green">
                  🔓 Desbloquear
                </button>
                <button v-if="admin.email !== 'master@master.master'" @click="deleteAdmin(admin)" class="btn-sm btn-danger">
                  🗑️ Excluir
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
    <div v-if="editingAdmin" class="modal-overlay">
      <div class="modal-content card">
        <h3>✏️ Editar Administrador</h3>
        <form @submit.prevent="updateAdmin" class="form-column">
          <label>
            Nome:
            <input v-model="editFormData.name" required />
          </label>
          <label>
            E-mail:
            <input v-model="editFormData.email" type="email" required />
          </label>
          <label>
            Telefone:
            <input :value="editFormData.phone" @input="(e) => editFormData.phone = formatPhone((e.target as HTMLInputElement).value)" required />
          </label>
          <div class="modal-actions">
            <button type="button" @click="closeEditModal" class="btn-back">Cancelar</button>
            <button type="submit" class="btn-submit">Salvar Alterações</button>
          </div>
        </form>
      </div>
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

const editingAdmin = ref<any>(null)
const editFormData = ref({ name: '', email: '', phone: '' })

const openEditModal = (admin: any) => {
  editingAdmin.value = admin
  editFormData.value = {
    name: admin.name,
    email: admin.email,
    phone: formatPhone(admin.phone)
  }
}

const closeEditModal = () => {
  editingAdmin.value = null
}

const updateAdmin = async () => {
  try {
    const payload = {
      name: editFormData.value.name,
      email: editFormData.value.email,
      phone: editFormData.value.phone.replace(/\D/g, '')
    }
    const res = await api.put(`/admins/${editingAdmin.value.id}`, payload)
    alert(res.data)
    closeEditModal()
    fetchAdmins()
  } catch (error: any) {
    alert(error.response?.data?.error || "Erro ao atualizar admin")
  }
}

const formatPhone = (val: string) => {
  if (!val) return ''
  let num = val.replace(/\D/g, '')
  if (num.length > 11) num = num.substring(0, 11)
  
  if (num.length === 0) return ''
  if (num.length <= 2) return `(${num}`
  if (num.length <= 3) return `(${num.substring(0, 2)}) ${num.substring(2)}`
  if (num.length <= 7) return `(${num.substring(0, 2)}) ${num.substring(2, 3)} ${num.substring(3)}`
  return `(${num.substring(0, 2)}) ${num.substring(2, 3)} ${num.substring(3, 7)} ${num.substring(7)}`
}

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
    const payload = { ...newAdmin.value, phone: newAdmin.value.phone.replace(/\D/g, '') }
    const res = await api.post('/admins', payload)
    alert(res.data)
    newAdmin.value = { name: '', email: '', phone: '', is_master: false }
    fetchAdmins()
  } catch (error: any) {
    alert(error.response?.data?.error || "Erro ao criar admin")
  }
}

const deleteAdmin = async (admin: any) => {
  if (!confirm(`Tem certeza que deseja APAGAR DEFINITIVAMENTE a conta de ${admin.name}?`)) return;
  try {
    const res = await api.delete(`/admins/${admin.id}`)
    alert(res.data)
    fetchAdmins()
  } catch (error: any) {
    alert(error.response?.data?.error || "Erro ao excluir")
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
.badge-green { background: #d1fae5; color: #065f46; padding: 0.2rem 0.5rem; border-radius: 99px; font-size: 0.8rem; white-space: nowrap; }
.badge-red { background: #fee2e2; color: #991b1b; padding: 0.2rem 0.5rem; border-radius: 99px; font-size: 0.8rem; white-space: nowrap; }
.badge-black { background: #1f2937; color: white; padding: 0.2rem 0.5rem; border-radius: 4px; font-size: 0.8rem; font-weight: bold; }
.btn-sm { padding: 0.4rem 0.6rem; font-size: 0.85rem; border: none; border-radius: 4px; cursor: pointer; color: white; font-weight: bold; transition: 0.2s; white-space: nowrap; }
.btn-blue { background: #3b82f6; } .btn-blue:hover { background: #2563eb; }
.btn-warn { background: #eab308; color: #1e293b; } .btn-warn:hover { background: #ca8a04; color: white; }
.btn-purple { background: #8b5cf6; } .btn-purple:hover { background: #7c3aed; }
.btn-danger { background: #ef4444; } .btn-danger:hover { background: #dc2626; }
.btn-green { background: #22c55e; } .btn-green:hover { background: #16a34a; }

.table-responsive {
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
  width: 100%;
}

@media (max-width: 768px) {
  .form-row { flex-direction: column; align-items: stretch; }
}

.modal-overlay { position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000; }
.modal-content { min-width: 400px; }
.form-column { display: flex; flex-direction: column; gap: 1rem; }
.form-column label { display: flex; flex-direction: column; font-size: 0.9rem; font-weight: bold; color: #4b5563; gap: 0.3rem; }
.form-column input { padding: 0.5rem; border: 1px solid #ccc; border-radius: 4px; }
.modal-actions { display: flex; justify-content: flex-end; gap: 1rem; margin-top: 1rem; }
</style>
