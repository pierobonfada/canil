<template>
  <div class="dashboard-container">
    <header class="top-bar">
      <h2>🐾 Painel do Canil</h2>
      <div class="header-actions">
        <button @click="router.push('/analytics')" class="btn-master" style="background: var(--primary-color);">📈 Estatísticas</button>
        <button v-if="isMaster" @click="router.push('/admins')" class="btn-master">👑 Admins</button>
        <button v-if="isMaster" @click="router.push('/logs')" class="btn-master">📊 Logs</button>
        <button @click="handleLogout" class="btn-logout">Sair</button>
      </div>
    </header>

    <main class="content">
      
      <section class="controls-card">
        <div class="controls-row-top">
          <div class="search-box">
            <span class="search-icon">🔍</span>
            <input 
              type="text" 
              v-model="searchQuery" 
              placeholder="Buscar animal pelo nome..." 
            />
          </div>
          <button @click="router.push('/animais/novo')" class="btn-add-pet">
            ➕ Adicionar Novo Animal
          </button>
        </div>
        
        <div class="toggle-filters">
          <div class="sort-box">
            <label>Ordenar por:</label>
            <select v-model="sortBy" @change="savePreferences">
              <option value="updated_desc">🕒 Última Atualização</option>
              <option value="name_asc">🔤 Nome (A-Z)</option>
            </select>
          </div>
          
          <label class="filter-label">
            <input type="checkbox" v-model="showOthers" @change="savePreferences" />
            Mostrar animais de outros tutores
          </label>
          <label class="filter-label">
            <input type="checkbox" v-model="showInactive" @change="savePreferences" />
            Mostrar inativos
          </label>
        </div>
      </section>

      <div v-if="isLoading" class="loading-state">
        <p>Sincronizando registros do canil...</p>
      </div>
      
      <div class="animal-grid" v-else-if="filteredAnimals.length > 0">
        <div v-for="animal in filteredAnimals" :key="animal.id" class="animal-card" :class="{ 'inactive-card': !animal.is_active }">
          
          <div class="photo-stack" @click="cyclePhoto(animal)">
            <template v-if="getStackedPhotos(animal).length > 0">
              <img 
                v-for="(photo, index) in getStackedPhotos(animal).slice(0, 3)" 
                :key="photo.file_path"
                :src="getPhotoUrl(photo.file_path)" 
                class="stacked-img"
                :class="'stack-layer-' + index"
                alt="Foto do animal"
              />
              <div v-if="getActivePhotos(animal).length > 1" class="stack-badge">
                📸 {{ getActivePhotos(animal).length }}
              </div>
            </template>
            <div v-else class="no-img">📷 Sem Foto</div>
            <span v-if="!animal.is_active" class="badge-inactive">Inativo</span>
          </div>

          <div class="card-info">
            <h3>{{ animal.name }} <span>{{ animal.species === 'Cachorro' ? '🐶' : '🐱' }}</span></h3>
            <p>Tutores vinculados: {{ animal.tutors_count }}</p>
          </div>

          <div class="card-actions">
            <button 
              @click="toggleTutor(animal.id)" 
              :class="animal.is_my_tutorship ? 'btn-tutor-active' : 'btn-tutor'"
            >
              {{ animal.is_my_tutorship ? '❤️ Sou Tutor (Remover)' : '🤍 Assumir Tutoria' }}
            </button>
            
            <div class="row-actions">
              <button @click="router.push(`/animais/editar/${animal.id}`)" class="btn-edit">✏️ Editar</button>
              <button @click="toggleStatus(animal)" :class="animal.is_active ? 'btn-danger' : 'btn-restore'">
                {{ animal.is_active ? '🗑️ Inativar' : '♻️ Reativar' }}
              </button>
            </div>
          </div>
        </div>
      </div>
      
      <div v-else class="empty-state">
        <p>Nenhum animal corresponde aos filtros aplicados.</p>
      </div>

    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import api from '../services/api'

const router = useRouter()
const animals = ref<any[]>([])
const isLoading = ref(true)
const isMaster = ref(localStorage.getItem('isMaster') === '1')

const searchQuery = ref('')
const showInactive = ref(false)
const showOthers = ref(false)
const sortBy = ref('updated_desc')

const getPhotoUrl = (photo: string) => {
  if (!photo) return '';
  const apiUrl = import.meta.env.VITE_API_URL || 'http://localhost:8000/api';
  const baseUrl = apiUrl.replace('/api', '');
  const cleanPhoto = photo.startsWith('uploads/') ? photo : `uploads/${photo}`;
  return `${baseUrl}/${cleanPhoto}`;
}

const getActivePhotos = (animal: any) => {
  return animal.photos ? animal.photos.filter((p: any) => p.is_active) : []
}

const cyclePhoto = (animal: any) => {
  const active = getActivePhotos(animal);
  if (active.length <= 1) return;
  
  if (animal.photoOffset === undefined) {
    animal.photoOffset = 0;
  }
  animal.photoOffset = (animal.photoOffset + 1) % active.length;
}

const getStackedPhotos = (animal: any) => {
  const active = getActivePhotos(animal);
  if (active.length === 0) return [];
  const offset = animal.photoOffset || 0;
  return [...active.slice(offset), ...active.slice(0, offset)];
}

let pollingInterval: any = null;

onMounted(async () => {
  try {
    const resPref = await api.get('/dashboard')
    showInactive.value = resPref.data.pref_show_inactive
    showOthers.value = resPref.data.pref_show_others
    sortBy.value = resPref.data.pref_sort_by || 'updated_desc'
    
    await fetchAnimals()

    pollingInterval = setInterval(() => {
      if (showOthers.value) {
        fetchAnimals(true)
      }
    }, 30000)
  } catch (error: any) {
    if (error.response && error.response.status === 401) {
      handleLogout()
    }
  }
})

onUnmounted(() => {
  if (pollingInterval) clearInterval(pollingInterval)
})

const savePreferences = async () => {
  try {
    await api.patch('/auth/preferences', {
      pref_show_inactive: showInactive.value,
      pref_show_others: showOthers.value,
      pref_sort_by: sortBy.value
    })
  } catch (error) {
    console.error("Falha ao salvar preferências no banco", error)
  }
}

const filteredAnimals = computed(() => {
  let filtered = animals.value.filter(animal => {
    const matchName = animal.name.toLowerCase().includes(searchQuery.value.toLowerCase())
    if (!matchName) return false
    if (!showInactive.value && !animal.is_active) return false
    if (!showOthers.value && !animal.is_my_tutorship) return false
    return true
  })

  filtered.sort((a, b) => {
    const scoreA = (a.is_my_tutorship ? 2 : 0) + (a.is_active ? 1 : 0);
    const scoreB = (b.is_my_tutorship ? 2 : 0) + (b.is_active ? 1 : 0);

    if (scoreA !== scoreB) return scoreB - scoreA;

    if (sortBy.value === 'name_asc') {
      return a.name.localeCompare(b.name);
    } else {
      const dateA = new Date(a.updated_at.replace(' ', 'T')).getTime();
      const dateB = new Date(b.updated_at.replace(' ', 'T')).getTime();
      return dateB - dateA;
    }
  });

  return filtered;
})

const fetchAnimals = async (silent = false) => {
  if (!silent) isLoading.value = true
  try {
    const res = await api.get('/animais')
    animals.value = res.data
  } catch (e) {
    console.error('Erro ao buscar listagem', e)
  } finally {
    if (!silent) isLoading.value = false
  }
}

const toggleStatus = async (animal: any) => {
  if (!confirm(`Deseja realmente ${animal.is_active ? 'inativar' : 'reativar'} ${animal.name}?`)) return
  try {
    await api.patch(`/animais/${animal.id}/status`, { is_active: !animal.is_active })
    await fetchAnimals()
  } catch (e) {
    alert('Erro ao alterar status do animal')
  }
}

const toggleTutor = async (id: number) => {
  try {
    await api.post(`/animais/${id}/tutores`)
    await fetchAnimals()
  } catch (error: any) {
    alert(error.response?.data?.error || 'Erro ao modificar tutoria')
  }
}

const handleLogout = () => {
  localStorage.removeItem('authToken')
  router.push('/')
}
</script>

<style scoped>
.dashboard-container { min-height: 100vh; background-color: #f8fafc; font-family: 'Segoe UI', Tahoma, sans-serif; padding-bottom: 4rem; }

.top-bar { 
  display: flex; justify-content: space-between; align-items: center; 
  background: linear-gradient(135deg, #166534 0%, #14532d 100%); 
  padding: 1rem 2rem; color: white; box-shadow: 0 4px 10px rgba(0,0,0,0.1);
  position: sticky; top: 0; z-index: 100;
}
.top-bar h2 { margin: 0; font-size: 1.4rem; font-weight: 600; }

.header-actions {
  display: flex;
  gap: 0.8rem;
  align-items: center;
}

.btn-master, .btn-logout {
  background-color: rgba(255,255,255,0.1); 
  border: 1px solid rgba(255,255,255,0.3); 
  color: white; 
  padding: 0.5rem 1rem; 
  border-radius: 6px; 
  cursor: pointer; 
  font-weight: 600; 
  transition: 0.2s;
}
.btn-master:hover, .btn-logout:hover { 
  background-color: white; 
  color: #166534; 
  border-color: white; 
}

.content { padding: 1.5rem; max-width: 1200px; margin: 0 auto; }

.controls-card { 
  background: white; border-radius: 12px; padding: 1.5rem; 
  margin-top: 1.5rem; margin-bottom: 2rem; 
  box-shadow: 0 4px 15px rgba(0,0,0,0.03); border: 1px solid #f1f5f9;
  display: flex; flex-direction: column; gap: 1.2rem;
}

.controls-row-top { display: flex; gap: 1rem; width: 100%; align-items: center; }
.search-box { position: relative; flex: 1; }
.search-icon { position: absolute; left: 15px; top: 50%; transform: translateY(-50%); color: #94a3b8; }
.search-box input {
  width: 100%; padding: 0.8rem 1rem 0.8rem 2.5rem; border: 1.5px solid #cbd5e1;
  border-radius: 8px; font-size: 1rem; color: #1e293b; background: #f8fafc; transition: 0.2s; box-sizing: border-box;
}
.search-box input:focus { outline: none; border-color: #22c55e; background: white; box-shadow: 0 0 0 3px rgba(34, 197, 94, 0.1); }

.btn-add-pet { 
  background-color: #166534; color: white; border: none; padding: 0.85rem 1.5rem; 
  border-radius: 8px; font-weight: bold; cursor: pointer; white-space: nowrap; transition: 0.2s;
}
.btn-add-pet:hover { background-color: #14532d; transform: translateY(-1px); }

.toggle-filters { display: flex; gap: 1.5rem; flex-wrap: wrap; align-items: center; background: #f8fafc; padding: 1rem; border-radius: 8px;}
.sort-box { display: flex; align-items: center; gap: 8px; font-weight: 600; color: #475569; font-size: 0.95rem; border-right: 2px solid #e2e8f0; padding-right: 1.5rem;}
.sort-box select { padding: 6px; border: 1px solid #cbd5e1; border-radius: 6px; background: white; color: #1e293b; font-weight: bold; outline: none;}

.filter-label { display: flex; align-items: center; gap: 8px; font-weight: 600; color: #475569; font-size: 0.95rem; cursor: pointer; }
.filter-label input { width: 18px; height: 18px; cursor: pointer; accent-color: #166534; }

.loading-state, .empty-state { text-align: center; padding: 4rem; color: #166534; font-weight: bold; font-size: 1.2rem; }

.animal-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 1.5rem; }
.animal-card { 
  background: white; border-radius: 12px; overflow: hidden; 
  box-shadow: 0 4px 15px rgba(0,0,0,0.03); border: 1px solid #f1f5f9; 
  display: flex; flex-direction: column; transition: 0.3s; 
}
.animal-card:hover { transform: translateY(-4px); box-shadow: 0 10px 25px rgba(0,0,0,0.08); }
.inactive-card { opacity: 0.65; filter: grayscale(50%); }

.photo-stack { 
  height: 220px; position: relative; background: #e2e8f0; cursor: pointer;
}
.stacked-img {
  position: absolute; top: 0; left: 0; width: 100%; height: 100%; object-fit: cover;
  transition: all 0.3s cubic-bezier(0.4, 0.0, 0.2, 1);
  border: 4px solid white; box-sizing: border-box; background: white;
}
.stack-layer-0 { z-index: 3; transform: scale(1) rotate(0deg); box-shadow: 0 2px 6px rgba(0,0,0,0.1); }
.stack-layer-1 { z-index: 2; transform: scale(0.97) translateY(12px) rotate(3deg); opacity: 0.9; box-shadow: 0 4px 8px rgba(0,0,0,0.2); border-color: #f1f5f9;}
.stack-layer-2 { z-index: 1; transform: scale(0.93) translateY(22px) rotate(-2deg); opacity: 0.8; box-shadow: 0 6px 10px rgba(0,0,0,0.2); border-color: #e2e8f0;}
.stack-badge {
  position: absolute; bottom: 12px; right: 12px; background: rgba(0,0,0,0.8);
  color: white; padding: 6px 10px; border-radius: 20px; font-size: 0.85rem; font-weight: bold; z-index: 10;
  backdrop-filter: blur(4px); box-shadow: 0 2px 4px rgba(0,0,0,0.3); pointer-events: none;
}
.no-img { height: 100%; display: flex; align-items: center; justify-content: center; color: #94a3b8; font-weight: bold; }
.badge-inactive { position: absolute; top: 12px; left: 12px; background: #ef4444; color: white; padding: 4px 10px; border-radius: 6px; font-size: 0.8rem; font-weight: bold; box-shadow: 0 2px 4px rgba(0,0,0,0.15); z-index: 20;}

.card-info { padding: 1.2rem; flex-grow: 1; z-index: 4; background: white;}
.card-info h3 { margin: 0 0 0.5rem 0; color: #1e293b; font-size: 1.25rem; }
.card-info p { margin: 0; color: #64748b; font-size: 0.9rem; font-weight: 500; }

.card-actions { padding: 1.2rem; background: #f8fafc; border-top: 1px solid #f1f5f9; display: flex; flex-direction: column; gap: 0.8rem; z-index: 4; }
.row-actions { display: flex; gap: 0.8rem; }

button { padding: 0.8rem; border: none; border-radius: 8px; font-weight: bold; cursor: pointer; transition: 0.2s; font-size: 0.9rem; display: flex; align-items: center; justify-content: center; }
.btn-tutor { background: #e2e8f0; color: #475569; width: 100%; }
.btn-tutor:hover { background: #cbd5e1; }
.btn-tutor-active { background: #fce7f3; color: #be185d; border: 1px solid #fbcfe8; width: 100%; }
.btn-tutor-active:hover { background: #fbcfe8; }

.btn-edit { background: #3b82f6; color: white; flex: 1; }
.btn-edit:hover { background: #2563eb; }
.btn-danger { background: #fee2e2; color: #dc2626; flex: 1; }
.btn-danger:hover { background: #fecaca; }
.btn-restore { background: #22c55e; color: white; flex: 1; }
.btn-restore:hover { background: #16a34a; }

@media (max-width: 768px) {
  .top-bar { padding: 1rem; flex-direction: column; gap: 1rem; text-align: center; }
  .top-bar h2 { font-size: 1.3rem; }
  .header-actions { flex-wrap: wrap; justify-content: center; }
  .controls-row-top { flex-direction: column; gap: 0.8rem; }
  .btn-add-pet { width: 100%; text-align: center; }
  .toggle-filters { flex-direction: column; gap: 1rem; align-items: flex-start;}
  .sort-box { border-right: none; padding-right: 0; width: 100%; justify-content: space-between;}
  .content { padding: 0.8rem; }
}
</style>