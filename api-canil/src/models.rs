use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

// ==========================================
// 📦 MODELOS DE DADOS E ESTRUTURAS (STRUCTS)
// ==========================================
// Pense neste arquivo como a "fábrica de forminhas". Cada Struct dita 
// exatamente o formato que os dados devem ter quando entram e saem do sistema!

// 🔐 Como o administrador vai bater na porta (login)


#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

// 🎟️ O que entregamos pro administrador caso a senha esteja correta
#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String, // O crachá digital!
    pub is_master: bool,
    pub is_first_login: bool,
    pub pref_show_inactive: bool,
    pub pref_show_others: bool,
    pub pref_sort_by: String,
}

// ⚠️ E se algo der errado? Retornamos essa caixinha de erro amigável.
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub remaining_attempts: Option<i32>,
}

// 📜 O que fica escrito "dentro" do nosso crachá JWT
#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub exp: usize,
}

// 🧑‍💼 Representação fiel de um administrador salvo lá no banco SQLite
#[derive(sqlx::FromRow)]
pub struct AdminRecord {
    pub id: i64,
    pub password: String,
    pub is_active: bool,
    pub is_master: bool,
    pub is_first_login: bool,
    pub email: String,
    pub pref_show_inactive: bool,
    pub pref_show_others: bool,
    pub pref_sort_by: String,
    pub failed_attempts: i32,
    pub is_locked: bool,
}

// 🌍 Estado Global: é assim que nossos Handlers conseguem conversar com o banco
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

// ==========================================
// 🐶 MODELOS DO PAINEL (CRUD DE ANIMAIS)
// ==========================================


#[derive(Deserialize)]
pub struct StatusPayload {
    pub is_active: bool,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct UpdatePreferencesRequest {
    pub pref_show_inactive: bool,
    pub pref_show_others: bool,
    pub pref_sort_by: String,
}

#[derive(Serialize)]
pub struct DashboardResponse {
    pub admin_id: i64,
    pub is_master: bool,
    pub pref_show_inactive: bool,
    pub pref_show_others: bool,
    pub pref_sort_by: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct PhotoInfo {
    pub file_path: String,
    pub is_active: bool,
    pub is_primary: bool,
}

#[derive(Serialize)]
pub struct AnimalListItem {
    pub id: i64,
    pub name: String,
    pub species: String,
    pub is_active: bool,
    pub is_my_tutorship: bool,
    pub tutors_count: i32,
    pub updated_at: String,
    pub photos: Vec<PhotoInfo>,
}

#[derive(Serialize)]
pub struct AnimalDetailResponse {
    pub id: i64,
    pub name: String,
    pub species: String,
    pub birth_year: i64,
    pub breed: String,
    pub is_vaccinated: bool,
    pub is_dewormed: bool,
    pub behavior_dogs: String,
    pub behavior_cats: String,
    pub behavior_humans: String,
    pub independence: String,
    pub size: String,
    pub coat_color: String,
    pub predominant_color: String,
    pub coat_length: String,
    pub description: String,
    pub is_active: bool,
    pub diseases: Vec<String>,
    pub photos: Vec<PhotoInfo>,
}

#[derive(sqlx::FromRow)]
pub struct AnimalRow {
    pub id: i64,
    pub name: String,
    pub species: String,
    pub birth_year: i64,
    pub breed: Option<String>,
    pub is_vaccinated: bool,
    pub is_dewormed: bool,
    pub behavior_dogs: String,
    pub behavior_cats: String,
    pub behavior_humans: Option<String>,
    pub independence: String,
    pub size: String,
    pub coat_color: String,
    pub predominant_color: String,
    pub coat_length: String,
    pub description: String,
    pub is_active: bool,
}

#[derive(Serialize)]
pub struct AdminListItem {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub is_active: bool,
    pub is_master: bool,
    pub is_locked: bool,
}

#[derive(Deserialize)]
pub struct CreateAdminRequest {
    pub name: String,
    pub email: String,
    pub phone: String,
    pub is_master: bool,
}

#[derive(Deserialize)]
pub struct UpdateAdminStatusRequest {
    pub is_active: Option<bool>,
    pub is_master: Option<bool>,
    pub force_password_reset: Option<bool>,
    pub is_locked: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateAdminRequest {
    pub name: String,
    pub email: String,
    pub phone: String,
}

#[derive(Serialize)]
pub struct SystemLog {
    pub id: i64,
    pub log_type: String,
    pub severity: String,
    pub description: String,
    pub admin_id: Option<i64>,
    pub animal_id: Option<i64>,
    pub remote_ip: String,
    pub timestamp: String,
}

#[derive(Deserialize)]
pub struct LogFilters {
    pub admin_id: Option<i64>,
    pub severity: Option<String>,
    pub animal_id: Option<i64>,
    pub log_type: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct PublicAnimalFilters {
    pub name: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub species: Option<String>,
    pub predominant_color: Option<String>,
    pub size: Option<String>,
    pub behavior_dogs: Option<String>,
    pub behavior_cats: Option<String>,
    pub behavior_humans: Option<String>,
    pub age_min: Option<i64>,
    pub age_max: Option<i64>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct PublicTutorContact {
    pub name: String,
    pub phone: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct PublicAnimalDetail {
    pub animal: AnimalDetailResponse,
    pub tutors: Vec<PublicTutorContact>,
}

#[derive(Deserialize, Debug)]
pub struct AnalyticsEventRequest {
    pub visitor_id: String,
    pub event_type: String,
    pub path: String,
    pub animal_id: Option<i64>,
    pub payload: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct SessionFilters {
    pub date: Option<String>,
    pub ip: Option<String>,
    pub animals: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Serialize, Debug)]
pub struct AnalyticsSessionResponse {
    pub visitor_id: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: String,
    pub events: Vec<AnalyticsEventDetail>,
}

#[derive(Serialize, Debug, sqlx::FromRow)]
pub struct AnalyticsEventDetail {
    pub id: i64,
    pub event_type: String,
    pub path: String,
    pub animal_id: Option<i64>,
    pub animal_name: Option<String>,
    pub payload: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, Debug)]
pub struct AnalyticsDashboardResponse {
    pub visitors_today: i64,
    pub visitors_week: i64,
    pub visitors_month: i64,
    pub visitors_year: i64,
    pub top_animals: Vec<TopAnimalStats>,
    pub most_searched_species: Vec<StatCount>,
    pub most_searched_color: Vec<StatCount>,
    pub most_searched_size: Vec<StatCount>,
    pub most_searched_age: Vec<StatCount>,
    pub most_contacted_animals: Vec<TopAnimalStats>,
}

#[derive(Serialize, Debug, sqlx::FromRow)]
pub struct TopAnimalStats {
    pub animal_id: i64,
    pub name: String,
    pub count: i64,
}

#[derive(Serialize, Debug, sqlx::FromRow)]
pub struct StatCount {
    pub name: String,
    pub count: i64,
}
