<template>
  <div class="logs-container">
    <header class="top-bar">
      <h2>📊 Relatórios e Logs do Sistema</h2>
      <button @click="router.push('/dashboard')" class="btn-back">⬅️ Voltar</button>
    </header>

    <div class="filters-card card">
      <div class="filter-group">
        <label>Tipo de Log</label>
        <select v-model="filters.log_type" @change="fetchLogs">
          <option value="">Todos</option>
          <option value="action">Ações</option>
          <option value="login">Logins</option>
          <option value="security">Segurança</option>
        </select>
      </div>

      <div class="filter-group">
        <label>Severidade</label>
        <select v-model="filters.severity" @change="fetchLogs">
          <option value="">Todas</option>
          <option value="INFO">INFO</option>
          <option value="WARNING">WARNING</option>
          <option value="CRITICAL">CRITICAL</option>
        </select>
      </div>
      
      <div class="filter-group">
        <label>Filtrar por Admin ID</label>
        <input type="number" v-model="filters.admin_id" placeholder="Ex: 1" @change="fetchLogs" />
      </div>

      <div class="filter-group">
        <label>Filtrar por Animal ID</label>
        <input type="number" v-model="filters.animal_id" placeholder="Ex: 10" @change="fetchLogs" />
      </div>
      
      <button @click="fetchLogs" class="btn-primary" style="align-self: flex-end;">🔄 Atualizar</button>
    </div>

    <div class="card list-card">
      <div class="table-responsive">
        <table class="data-table">
          <thead>
            <tr>
              <th>Data/Hora</th>
              <th>Tipo</th>
              <th>Severidade</th>
              <th>Descrição</th>
              <th>Admin ID</th>
              <th>Animal ID</th>
              <th>IP Remoto</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="log in logs" :key="log.id + log.log_type" :class="getSeverityClass(log.severity)">
              <td>{{ new Date(log.timestamp).toLocaleString() }}</td>
              <td>{{ log.log_type }}</td>
              <td><strong>{{ log.severity }}</strong></td>
              <td>{{ log.description }}</td>
              <td>{{ log.admin_id || '-' }}</td>
              <td>{{ log.animal_id || '-' }}</td>
              <td>{{ log.remote_ip }}</td>
            </tr>
            <tr v-if="logs.length === 0">
              <td colspan="7" style="text-align: center;">Nenhum log encontrado.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import api from '../services/api'

const router = useRouter()
const logs = ref<any[]>([])
const filters = ref({
  log_type: '',
  severity: '',
  admin_id: '',
  animal_id: ''
})

const fetchLogs = async () => {
  try {
    const params = new URLSearchParams()
    if (filters.value.log_type) params.append('log_type', filters.value.log_type)
    if (filters.value.severity) params.append('severity', filters.value.severity)
    if (filters.value.admin_id) params.append('admin_id', filters.value.admin_id)
    if (filters.value.animal_id) params.append('animal_id', filters.value.animal_id)
    
    const res = await api.get(`/logs?${params.toString()}`)
    logs.value = res.data
  } catch (error) {
    alert("Erro ao buscar logs")
  }
}

const getSeverityClass = (sev: string) => {
  if (sev === 'CRITICAL') return 'row-critical'
  if (sev === 'WARNING') return 'row-warning'
  return 'row-info'
}

onMounted(() => {
  fetchLogs()
})
</script>

<style scoped>
.logs-container { padding: 2rem; max-width: 1400px; margin: auto; }
.top-bar { display: flex; justify-content: space-between; margin-bottom: 2rem; }
.btn-back { background: #6b7280; color: white; padding: 0.5rem 1rem; border: none; border-radius: 4px; cursor: pointer; }
.card { background: white; padding: 1.5rem; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); margin-bottom: 2rem; }
.filters-card { display: flex; gap: 1.5rem; flex-wrap: wrap; align-items: center; }
.filter-group { display: flex; flex-direction: column; gap: 0.25rem; }
.filter-group label { font-size: 0.85rem; font-weight: bold; color: #4b5563; }
.filter-group input, .filter-group select { padding: 0.5rem; border: 1px solid #d1d5db; border-radius: 4px; min-width: 150px; }
.btn-primary { background: #3b82f6; color: white; padding: 0.5rem 1rem; border: none; border-radius: 4px; cursor: pointer; font-weight: bold; }
.data-table { width: 100%; border-collapse: collapse; font-size: 0.9rem; }
.data-table th, .data-table td { padding: 0.75rem; text-align: left; border-bottom: 1px solid #eee; }
.row-info { background-color: #f0fdf4; color: #064e3b; }
.row-warning { background-color: #fffbeb; color: #92400e; }
.row-critical { background-color: #fef2f2; color: #991b1b; }

.table-responsive {
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
  width: 100%;
}
</style>
