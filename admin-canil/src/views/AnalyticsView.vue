<template>
  <div class="dashboard-container">
    <header class="top-bar">
      <h2>📈 Estatísticas e Acessos</h2>
      <div class="header-actions">
        <button @click="$router.push('/dashboard')" class="btn-master">⬅️ Voltar ao Painel</button>
        <button @click="handleLogout" class="btn-logout">Sair</button>
      </div>
    </header>

    <main class="content">
      <div class="tabs">
        <button :class="{ active: activeTab === 'dashboard' }" @click="activeTab = 'dashboard'">Visão Geral</button>
        <button :class="{ active: activeTab === 'sessions' }" @click="activeTab = 'sessions'">Sessões (Usuários)</button>
      </div>

      <div v-if="isLoading" class="loading">Carregando dados...</div>

      <div v-else-if="activeTab === 'dashboard'">
        <div class="stats-grid">
          <div class="stat-card">
            <h3>Visitantes Hoje</h3>
            <div class="stat-value">{{ dashboardData?.visitors_today }}</div>
          </div>
          <div class="stat-card">
            <h3>Visitantes (7 dias)</h3>
            <div class="stat-value">{{ dashboardData?.visitors_week }}</div>
          </div>
          <div class="stat-card">
            <h3>Visitantes (30 dias)</h3>
            <div class="stat-value">{{ dashboardData?.visitors_month }}</div>
          </div>
          <div class="stat-card">
            <h3>Visitantes (1 Ano)</h3>
            <div class="stat-value">{{ dashboardData?.visitors_year }}</div>
          </div>
        </div>

        <div class="charts-grid">
          <div class="chart-card">
            <div class="card-header">
              <h3>Top Animais (Visualizações)</h3>
              <select v-model="sortViews" class="sort-select">
                <option value="desc">Mais vistos</option>
                <option value="asc">Menos vistos</option>
              </select>
            </div>
            <ul class="scrollable-list">
              <li v-for="item in sortedTopAnimals" :key="item.animal_id">
                <span class="name">{{ item.name }}</span>
                <span class="count">{{ item.count }} views</span>
              </li>
              <li v-if="!sortedTopAnimals?.length">Sem dados</li>
            </ul>
          </div>
          <div class="chart-card">
            <h3>Top Animais (Contato Solicitado)</h3>
            <ul class="scrollable-list">
              <li v-for="item in dashboardData?.most_contacted_animals" :key="item.animal_id">
                <span class="name">{{ item.name }}</span>
                <span class="count">{{ item.count }} msg</span>
              </li>
              <li v-if="!dashboardData?.most_contacted_animals?.length">Sem dados</li>
            </ul>
          </div>
          <div class="chart-card">
            <h3>Espécies mais buscadas</h3>
            <ul class="scrollable-list">
              <li v-for="item in dashboardData?.most_searched_species" :key="item.name">
                <span class="name">{{ item.name }}</span>
                <span class="count">{{ item.count }} buscas</span>
              </li>
            </ul>
          </div>
          <div class="chart-card">
            <h3>Cores mais buscadas</h3>
            <ul class="scrollable-list">
              <li v-for="item in dashboardData?.most_searched_color" :key="item.name">
                <span class="name">{{ item.name }}</span>
                <span class="count">{{ item.count }} buscas</span>
              </li>
            </ul>
          </div>
          <div class="chart-card">
            <h3>Tamanhos mais buscados</h3>
            <ul class="scrollable-list">
              <li v-for="item in dashboardData?.most_searched_size" :key="item.name">
                <span class="name">{{ item.name }}</span>
                <span class="count">{{ item.count }} buscas</span>
              </li>
            </ul>
          </div>
          <div class="chart-card">
            <h3>Idades mais buscadas</h3>
            <ul class="scrollable-list">
              <li v-for="item in dashboardData?.most_searched_age" :key="item.name">
                <span class="name">{{ item.name }}</span>
                <span class="count">{{ item.count }} buscas</span>
              </li>
            </ul>
          </div>
        </div>
      </div>

      <div v-else-if="activeTab === 'sessions'">
        <div class="session-filters">
          <input type="date" v-model="sessionFilters.date" @change="resetAndFetchSessions" title="Filtrar por data" />
          <input type="text" v-model="sessionFilters.ip" @input="debounceSessions" placeholder="Filtrar por IP..." />
          <input type="text" v-model="sessionFilters.animals" @input="debounceSessions" placeholder="Animais visitados (Rex Bolinha...)" />
        </div>
        <table class="data-table">
          <thead>
            <tr>
              <th>ID (Visitante)</th>
              <th>IP</th>
              <th>Dispositivo / Navegador</th>
              <th>Primeiro Acesso</th>
              <th>Ações Feitas</th>
              <th>Ação</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="s in sessions" :key="s.visitor_id">
              <td class="short-id">{{ s.visitor_id }}</td>
              <td>{{ s.ip_address }}</td>
              <td class="ua-cell" :title="s.user_agent">{{ s.user_agent }}</td>
              <td>{{ new Date(s.start_time + 'Z').toLocaleString() }}</td>
              <td>{{ s.total_events }}</td>
              <td><button class="btn-sm" @click="viewSession(s.visitor_id)">Detalhes</button></td>
            </tr>
          </tbody>
        </table>
        <div class="load-more-trigger" ref="loadMoreTrigger" v-if="hasMoreSessions">
          <p v-if="isFetchingMore">Carregando mais sessões...</p>
        </div>
        <div v-if="!hasMoreSessions && sessions.length > 0" class="end-msg">Todas as sessões carregadas.</div>
        <div v-if="sessions.length === 0 && !isLoading" class="end-msg">Nenhuma sessão encontrada.</div>
      </div>
    </main>

    <!-- Modal de Sessão Detalhada -->
    <div class="modal-overlay" v-if="selectedSessionId" @click.self="selectedSessionId = null">
      <div class="modal-content">
        <header>
          <h3>Detalhes da Navegação</h3>
          <button class="btn-close" @click="selectedSessionId = null">×</button>
        </header>
        <div class="modal-body" v-if="sessionDetails">
          <p><strong>IP:</strong> {{ sessionDetails.ip_address }}</p>
          <p><strong>Browser:</strong> {{ sessionDetails.user_agent }}</p>
          <hr />
          <ul class="timeline">
            <li v-for="ev in sessionDetails.events" :key="ev.id">
              <span class="time">{{ new Date(ev.created_at + 'Z').toLocaleTimeString() }}</span>
              <span class="badge" :class="ev.event_type">{{ ev.event_type }}</span>
              <span class="path">{{ ev.path }}</span>
              <div v-if="ev.animal_id" class="animal-ref">🐾 {{ ev.animal_name || 'Animal ' + ev.animal_id }} (ID: {{ ev.animal_id }})</div>
              <pre v-if="ev.payload" class="payload-box">{{ formatPayload(ev.payload) }}</pre>
            </li>
          </ul>
        </div>
        <div v-else>Carregando...</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import api from '../services/api';

const router = useRouter();
const activeTab = ref('dashboard');
const isLoading = ref(true);
const sortViews = ref('desc');

const dashboardData = ref<any>(null);
const sessions = ref<any[]>([]);
const sessionFilters = ref({ date: '', ip: '', animals: '' });
const sessionPage = ref(1);
const hasMoreSessions = ref(true);
const isFetchingMore = ref(false);
const loadMoreTrigger = ref<HTMLElement | null>(null);

let debounceTimeout: any = null;
const debounceSessions = () => {
  clearTimeout(debounceTimeout);
  debounceTimeout = setTimeout(resetAndFetchSessions, 500);
};

const resetAndFetchSessions = () => {
  sessions.value = [];
  sessionPage.value = 1;
  hasMoreSessions.value = true;
  fetchSessions();
};

const sortedTopAnimals = computed(() => {
  if (!dashboardData.value?.top_animals) return [];
  const arr = [...dashboardData.value.top_animals];
  if (sortViews.value === 'asc') {
    arr.sort((a, b) => a.count - b.count);
  } else {
    arr.sort((a, b) => b.count - a.count);
  }
  return arr;
});

const selectedSessionId = ref<string | null>(null);
const sessionDetails = ref<any>(null);

const fetchSessions = async () => {
  if (isFetchingMore.value || !hasMoreSessions.value) return;
  if (sessionPage.value === 1) isLoading.value = true;
  else isFetchingMore.value = true;

  try {
    const params: any = { page: sessionPage.value, limit: 20 };
    if (sessionFilters.value.date) params.date = sessionFilters.value.date;
    if (sessionFilters.value.ip) params.ip = sessionFilters.value.ip;
    if (sessionFilters.value.animals) params.animals = sessionFilters.value.animals;

    const res = await api.get('/analytics/sessions', { params });
    if (res.data.length < 20) {
      hasMoreSessions.value = false;
    }
    sessions.value = [...sessions.value, ...res.data];
    sessionPage.value++;
  } catch (err) {
    console.error(err);
  } finally {
    isLoading.value = false;
    isFetchingMore.value = false;
  }
};

const fetchData = async () => {
  if (activeTab.value === 'dashboard') {
    isLoading.value = true;
    try {
      const res = await api.get('/analytics/dashboard');
      dashboardData.value = res.data;
    } catch (err) {
      console.error(err);
      alert('Erro ao carregar estatísticas');
    } finally {
      isLoading.value = false;
    }
  } else {
    resetAndFetchSessions();
  }
};

let observer: IntersectionObserver | null = null;

watch(activeTab, (newTab) => {
  fetchData();
  if (newTab === 'sessions') {
    setTimeout(setupObserver, 100);
  } else if (observer) {
    observer.disconnect();
  }
});

const setupObserver = () => {
  if (observer) observer.disconnect();
  observer = new IntersectionObserver((entries) => {
    if (entries[0].isIntersecting && hasMoreSessions.value && !isLoading.value) {
      fetchSessions();
    }
  });
  if (loadMoreTrigger.value) {
    observer.observe(loadMoreTrigger.value);
  }
};

onMounted(() => {
  fetchData();
});

const handleLogout = () => {
  localStorage.removeItem('authToken');
  localStorage.removeItem('isMaster');
  localStorage.removeItem('adminId');
  router.push('/');
};

const viewSession = async (visitorId: string) => {
  selectedSessionId.value = visitorId;
  sessionDetails.value = null;
  try {
    const res = await api.get(`/analytics/sessions/${visitorId}`);
    sessionDetails.value = res.data;
  } catch (e) {
    alert('Erro ao carregar sessão');
  }
};

const formatPayload = (str: string) => {
  try {
    return JSON.stringify(JSON.parse(str), null, 2);
  } catch {
    return str;
  }
};
</script>

<style scoped>
.dashboard-container {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.top-bar {
  background: white;
  padding: 1rem 2rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-shadow: 0 2px 10px rgba(0,0,0,0.05);
}

.header-actions {
  display: flex;
  gap: 1rem;
}

.btn-master {
  background: #3b82f6;
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 8px;
  cursor: pointer;
}

.btn-logout {
  background: #ef4444;
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 8px;
  cursor: pointer;
}

.content {
  flex: 1;
  padding: 2rem;
  max-width: 1200px;
  margin: 0 auto;
  width: 100%;
}

.tabs {
  display: flex;
  gap: 1rem;
  margin-bottom: 2rem;
}

.tabs button {
  padding: 0.8rem 1.5rem;
  border: none;
  background: white;
  border-radius: 8px;
  cursor: pointer;
  font-weight: bold;
  color: #64748b;
  box-shadow: 0 2px 5px rgba(0,0,0,0.05);
}

.tabs button.active {
  background: #4f46e5;
  color: white;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1.5rem;
  margin-bottom: 2rem;
}

.stat-card {
  background: white;
  padding: 1.5rem;
  border-radius: 12px;
  box-shadow: 0 4px 6px rgba(0,0,0,0.05);
  text-align: center;
}

.stat-card h3 {
  color: #64748b;
  font-size: 0.9rem;
  text-transform: uppercase;
  margin-bottom: 0.5rem;
}

.stat-value {
  font-size: 2.5rem;
  font-weight: bold;
  color: #4f46e5;
}

.charts-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1.5rem;
}

.chart-card {
  background: white;
  padding: 1.5rem;
  border-radius: 12px;
  box-shadow: 0 4px 6px rgba(0,0,0,0.05);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
  border-bottom: 1px solid #e2e8f0;
  padding-bottom: 0.5rem;
}

.card-header h3 {
  margin-bottom: 0;
  border-bottom: none;
  padding-bottom: 0;
}

.sort-select {
  padding: 0.2rem;
  border-radius: 4px;
  border: 1px solid #cbd5e1;
}

.chart-card h3 {
  margin-bottom: 1rem;
  border-bottom: 1px solid #e2e8f0;
  padding-bottom: 0.5rem;
}

.scrollable-list {
  max-height: 250px;
  overflow-y: auto;
}

.scrollable-list::-webkit-scrollbar {
  width: 6px;
}
.scrollable-list::-webkit-scrollbar-track {
  background: #f1f5f9; 
}
.scrollable-list::-webkit-scrollbar-thumb {
  background: #cbd5e1; 
  border-radius: 4px;
}

.chart-card ul {
  list-style: none;
  padding: 0;
  margin: 0;
}

.chart-card li {
  display: flex;
  justify-content: space-between;
  padding: 0.5rem 0;
  border-bottom: 1px solid #f1f5f9;
}

.chart-card .count {
  font-weight: bold;
  color: #4f46e5;
}

.data-table {
  width: 100%;
  border-collapse: collapse;
  background: white;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 4px 6px rgba(0,0,0,0.05);
}

.data-table th, .data-table td {
  padding: 1rem;
  text-align: left;
  border-bottom: 1px solid #e2e8f0;
}

.data-table th {
  background: #f8fafc;
  font-weight: 600;
  color: #475569;
}

.short-id {
  max-width: 80px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ua-cell {
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.btn-sm {
  background: #e2e8f0;
  border: none;
  padding: 0.4rem 0.8rem;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.85rem;
}

.btn-sm:hover {
  background: #cbd5e1;
}

.session-filters {
  display: flex;
  gap: 1rem;
  margin-bottom: 1rem;
}

.session-filters input {
  padding: 0.6rem 1rem;
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  font-size: 0.95rem;
  flex: 1;
}

.load-more-trigger {
  text-align: center;
  padding: 1.5rem;
  color: #64748b;
}

.end-msg {
  text-align: center;
  padding: 1.5rem;
  color: #94a3b8;
  font-size: 0.9rem;
}

.modal-overlay {
  position: fixed;
  top: 0; left: 0; right: 0; bottom: 0;
  background: rgba(0,0,0,0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal-content {
  background: white;
  width: 90%;
  max-width: 600px;
  max-height: 90vh;
  border-radius: 12px;
  display: flex;
  flex-direction: column;
}

.modal-content header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.5rem;
  border-bottom: 1px solid #e2e8f0;
}

.btn-close {
  background: transparent;
  border: none;
  font-size: 1.5rem;
  cursor: pointer;
}

.modal-body {
  padding: 1.5rem;
  overflow-y: auto;
}

.timeline {
  list-style: none;
  padding: 0;
}

.timeline li {
  margin-bottom: 1rem;
  padding-bottom: 1rem;
  border-bottom: 1px dashed #e2e8f0;
}

.time {
  font-size: 0.85rem;
  color: #64748b;
  margin-right: 1rem;
}

.badge {
  padding: 0.2rem 0.5rem;
  border-radius: 4px;
  font-size: 0.8rem;
  font-weight: bold;
  background: #e2e8f0;
  margin-right: 0.5rem;
}

.badge.page_view { background: #dbeafe; color: #1e3a8a; }
.badge.search { background: #fef08a; color: #854d0e; }
.badge.message_sent { background: #bbf7d0; color: #166534; }

.payload-box {
  background: #f8fafc;
  padding: 0.5rem;
  border-radius: 4px;
  font-size: 0.85rem;
  margin-top: 0.5rem;
  overflow-x: auto;
}
</style>
