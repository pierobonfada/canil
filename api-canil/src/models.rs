use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub is_first_login: bool,
    pub pref_show_inactive: bool,
    pub pref_show_others: bool,
    pub pref_sort_by: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub exp: usize,
}

#[derive(sqlx::FromRow)]
pub struct AdminRecord {
    pub id: i64,
    pub password: String,
    pub is_active: bool,
    pub is_first_login: bool,
    pub pref_show_inactive: bool,
    pub pref_show_others: bool,
    pub pref_sort_by: String,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

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
    pub behavior_humans: String,
    pub independence: String,
    pub size: String,
    pub coat_color: String,
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
    pub behavior_humans: String,
    pub independence: String,
    pub size: String,
    pub coat_color: String,
    pub coat_length: String,
    pub description: String,
    pub is_active: bool,
}