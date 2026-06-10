<!-- ========================================== -->
<!-- DASHBOARD DO ADMIN: RENDERIZAÇÃO CONDICIONAL -->
<!-- ========================================== -->
<!-- Este componente demonstra o uso avançado de diretivas como 'v-if' e 'v-else-if'.
Botões como "Estatísticas" e "Logs" só são inseridos no DOM se o usuário tiver
permissão de Master ('v-if="isMaster"'). -->
<template>
  <div class="dashboard-container">
    <header class="top-bar">
      <div class="header-left">
        <h2>🐾 Painel do Canil</h2>
        
        <div v-if="adminInfo" class="admin-profile" @click="openEditProfile" title="Editar Perfil">
          <div class="admin-details">
            <span class="admin-name">{{ adminInfo.name }}</span>
            <span class="admin-contact">{{ adminInfo.email }} • {{ formatPhone(adminInfo.phone) }}</span>
          </div>
          <span class="edit-icon">✏️</span>
        </div>
      </div>

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
                loading="lazy"
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

    <!-- Modal de Edição de Perfil -->
    <div v-if="showEditProfile" class="modal-overlay">
      <div class="modal-content card">
        <h3>✏️ Editar Meu Perfil</h3>
        <form @submit.prevent="updateProfile" class="form-column">
          <label>
            Nome:
            <input v-model="editProfileData.name" required />
          </label>
          <label>
            E-mail:
            <input v-model="editProfileData.email" type="email" required />
          </label>
          <label>
            WhatsApp (Telefone):
            <input :value="editProfileData.phone" @input="(e) => editProfileData.phone = formatPhone((e.target as HTMLInputElement).value)" required />
          </label>
          <label>
            Nova Senha:
            <div class="password-input-wrapper">
              <input :type="showPassword ? 'text' : 'password'" v-model="editProfileData.password" placeholder="Manter inalterada" />
              <button type="button" class="btn-toggle-password" @click="showPassword = !showPassword" title="Mostrar/Ocultar senha">
                <svg v-if="!showPassword" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color: #64748b;"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color: #64748b;"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path><line x1="1" y1="1" x2="23" y2="23"></line></svg>
              </button>
            </div>
            <p class="password-hint" v-if="editProfileData.password.length > 0">
              A senha deve ter no mínimo 10 caracteres, 1 número e 1 caractere especial (ex: @, !, #).
            </p>
            <PasswordStrength v-if="editProfileData.password.length > 0" :password="editProfileData.password" />
          </label>
          <label v-if="editProfileData.password.length > 0">
            Confirme a Nova Senha:
            <div class="password-input-wrapper">
              <input :type="showConfirmPassword ? 'text' : 'password'" v-model="editProfileData.confirmPassword" placeholder="Confirme a nova senha" required />
              <button type="button" class="btn-toggle-password" @click="showConfirmPassword = !showConfirmPassword" title="Mostrar/Ocultar senha">
                <svg v-if="!showConfirmPassword" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color: #64748b;"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color: #64748b;"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path><line x1="1" y1="1" x2="23" y2="23"></line></svg>
              </button>
            </div>
          </label>
          <div class="modal-actions">
            <button type="button" @click="showEditProfile = false" class="btn-back">Cancelar</button>
            <button type="submit" class="btn-submit">Salvar Alterações</button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// ==========================================
// LÓGICA DO DASHBOARD: Polling e Computed Properties
// ==========================================
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import api from '../services/api'
import PasswordStrength from '../components/PasswordStrength.vue'

const router = useRouter()
const animals = ref<any[]>([])
const isLoading = ref(true)
const isMaster = ref(localStorage.getItem('isMaster') === '1')

const adminInfo = ref<any>(null)
const showEditProfile = ref(false)
const showPassword = ref(false)
const showConfirmPassword = ref(false)
const editProfileData = ref({ name: '', email: '', phone: '', password: '', confirmPassword: '' })

const searchQuery = ref('')
const showInactive = ref(false)
const showOthers = ref(false)
const sortBy = ref('updated_desc')

const getPhotoUrl = (photo: string) => {
  if (!photo) return '';
  const apiUrl = import.meta.env.VITE_API_URL || (import.meta.env.DEV ? 'http://localhost:8000/api' : '/api');
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

const openEditProfile = () => {
  if (!adminInfo.value) return;
  editProfileData.value = {
    name: adminInfo.value.name,
    email: adminInfo.value.email,
    phone: formatPhone(adminInfo.value.phone),
    password: '',
    confirmPassword: ''
  }
  showPassword.value = false
  showConfirmPassword.value = false
  showEditProfile.value = true
}

const updateProfile = async () => {
  if (editProfileData.value.password.length > 0) {
    if (editProfileData.value.password.length < 10) {
      alert('A nova senha deve ter pelo menos 10 caracteres.'); return;
    }
    if (!/\d/.test(editProfileData.value.password)) {
      alert('A nova senha deve conter pelo menos um número.'); return;
    }
    if (!/[^a-zA-Z0-9]/.test(editProfileData.value.password)) {
      alert('A nova senha deve conter pelo menos um caractere especial.'); return;
    }
    if (editProfileData.value.password !== editProfileData.value.confirmPassword) {
      alert('A confirmação da nova senha não coincide.'); return;
    }
  }

  try {
    const payload: any = {
      name: editProfileData.value.name,
      email: editProfileData.value.email,
      phone: editProfileData.value.phone.replace(/\D/g, '')
    }
    if (editProfileData.value.password) {
      payload.password = editProfileData.value.password
    }

    await api.put('/auth/me', payload)
    showEditProfile.value = false
    
    // Refresh admin info
    const resPref = await api.get('/dashboard')
    adminInfo.value = resPref.data
    alert('Perfil atualizado com sucesso!')
  } catch (error: any) {
    alert(error.response?.data?.error || "Erro ao atualizar perfil")
  }
}

let pollingInterval: any = null;

// CICLO DE VIDA: onMounted
// Executa assim que o componente é desenhado na tela.
// Aqui fazemos a busca inicial e configuramos um 'Polling' (intervalo)
// para buscar atualizações a cada 30 segundos, caso o usuário queira ver
// se outros tutores adicionaram animais.
onMounted(async () => {
  try {
    const resPref = await api.get('/dashboard')
    adminInfo.value = resPref.data
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

// CICLO DE VIDA: onUnmounted
// *Pitfall*: Sempre limpe os intervalos (setInterval) quando o componente for destruído,
// senão eles continuarão rodando em 'background' vazando memória!
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

// COMPUTED PROPERTIES (Propriedades Computadas)
// Ao invés de mudar a lista original sempre que o usuário digita na busca,
// criamos uma lista 'derivada'. O Vue observa 'searchQuery', 'showInactive', etc.
// e recalcula essa lista automaticamente (e de forma muito otimizada) quando algo muda.
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

.header-left {
  display: flex;
  align-items: center;
  gap: 2rem;
}

.top-bar h2 { margin: 0; font-size: 1.4rem; font-weight: 600; }

.admin-profile {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  background: rgba(255, 255, 255, 0.1);
  padding: 0.4rem 0.8rem;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.admin-profile:hover {
  background: rgba(255, 255, 255, 0.15);
  border-color: rgba(255, 255, 255, 0.3);
}

.admin-details {
  display: flex;
  flex-direction: column;
  line-height: 1.2;
}

.admin-name {
  font-weight: 600;
  font-size: 0.9rem;
}

.admin-contact {
  font-size: 0.75rem;
  color: rgba(255, 255, 255, 0.8);
}

.edit-icon {
  font-size: 1rem;
  opacity: 0.7;
}

.admin-profile:hover .edit-icon {
  opacity: 1;
}

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
  content-visibility: auto; contain-intrinsic-size: auto none auto 380px;
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
  .header-left { flex-direction: column; gap: 0.5rem; }
  .admin-profile { width: 100%; justify-content: center; }
  .top-bar { padding: 1rem; flex-direction: column; gap: 1rem; text-align: center; }
  .top-bar h2 { font-size: 1.3rem; }
  .header-actions { flex-wrap: wrap; justify-content: center; }
  .controls-row-top { flex-direction: column; gap: 0.8rem; }
  .btn-add-pet { width: 100%; text-align: center; }
  .toggle-filters { flex-direction: column; gap: 1rem; align-items: flex-start;}
  .sort-box { border-right: none; padding-right: 0; width: 100%; justify-content: space-between;}
  .content { padding: 0.8rem; }
}

/* Modal styles */
.modal-overlay { position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000; }
.modal-content { min-width: 400px; background: white; padding: 1.5rem; border-radius: 8px; box-shadow: 0 4px 15px rgba(0,0,0,0.1); }
.modal-content h3 { margin-top: 0; color: #1e293b; }
.form-column { display: flex; flex-direction: column; gap: 1rem; }
.form-column label { display: flex; flex-direction: column; font-size: 0.9rem; font-weight: bold; color: #4b5563; gap: 0.3rem; }
.form-column input { padding: 0.5rem; border: 1px solid #ccc; border-radius: 4px; font-size: 1rem; }
.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 1rem;
  margin-top: 1.5rem;
}

.password-input-wrapper {
  position: relative;
  display: block;
}
.password-input-wrapper input {
  width: 100%;
  box-sizing: border-box;
  padding-right: 40px;
}
.btn-toggle-password {
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  background: transparent !important;
  border: none;
  cursor: pointer;
  padding: 0 !important;
  width: auto !important;
  display: flex;
  align-items: center;
  justify-content: center;
}
.btn-toggle-password:hover {
  background: transparent !important;
  opacity: 0.7;
}
.password-hint {
  font-size: 0.8rem;
  color: #6b7280;
  margin-top: 4px;
  margin-bottom: 4px;
}
.btn-back { background: #e2e8f0; color: #475569; padding: 0.6rem 1rem; border: none; border-radius: 6px; cursor: pointer; font-weight: bold; }
.btn-submit { background: #166534; color: white; padding: 0.6rem 1rem; border: none; border-radius: 6px; cursor: pointer; font-weight: bold; }
.btn-submit:hover { background: #14532d; }
</style>