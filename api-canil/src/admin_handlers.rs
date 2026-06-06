use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

use crate::models::{
    AdminListItem, AdminRecord, AppState, Claims, CreateAdminRequest, ErrorResponse, LogFilters,
    SystemLog, UpdateAdminStatusRequest,
};

fn generate_temp_password(full_name: &str) -> String {
    let name_lower = full_name.to_lowercase();
    let chars: Vec<char> = name_lower.chars().map(|c| match c {
        'á'|'à'|'ã'|'â'|'ä' => 'a',
        'é'|'è'|'ê'|'ë' => 'e',
        'í'|'ì'|'î'|'ï' => 'i',
        'ó'|'ò'|'õ'|'ô'|'ö' => 'o',
        'ú'|'ù'|'û'|'ü' => 'u',
        'ç' => 'c',
        'ñ' => 'n',
        _ => c
    }).collect();
    let cleaned_name: String = chars.into_iter().filter(|c| c.is_alphanumeric() || c.is_whitespace()).collect();
    
    let words: Vec<&str> = cleaned_name.split_whitespace().collect();
    if words.is_empty() {
        return "senha123".to_string();
    }
    if words.len() == 1 {
        return words[0].to_string();
    }
    format!("{}{}", words[0], words[words.len() - 1])
}

async fn check_master(claims: &Claims, state: &AppState) -> Result<AdminRecord, (StatusCode, Json<ErrorResponse>)> {
    let admin = sqlx::query_as::<_, AdminRecord>("SELECT id, password, is_active, is_master, is_first_login, pref_show_inactive, pref_show_others, pref_sort_by, email FROM admins WHERE id = ?")
        .bind(claims.sub)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Erro no banco".to_string(), remaining_attempts: None })))?;
    
    match admin {
        Some(a) if a.is_master && a.is_active => Ok(a),
        Some(a) => {
            let msg = format!("Tentativa de escalonamento de privilégio. Admin: {}", a.email);
            let _ = sqlx::query("INSERT INTO security_warnings (msg, remote_ip, endpoint, user_agent, severity) VALUES (?, 'Desconhecido', 'Rotas Master', 'Desconhecido', 'CRITICAL')")
                .bind(&msg).execute(&state.pool).await;
            Err((StatusCode::FORBIDDEN, Json(ErrorResponse { error: "Acesso negado. Apenas masters.".to_string(), remaining_attempts: None })))
        },
        _ => Err((StatusCode::FORBIDDEN, Json(ErrorResponse { error: "Acesso negado. Apenas masters.".to_string(), remaining_attempts: None })))
    }
}

pub async fn get_admins(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Vec<AdminListItem>>, (StatusCode, Json<ErrorResponse>)> {
    check_master(&claims, &state).await?;

    let admins = sqlx::query_as!(
        AdminListItem,
        "SELECT id, name, email, phone, is_active as \"is_active: bool\", is_master as \"is_master: bool\", is_locked as \"is_locked: bool\" FROM admins ORDER BY id"
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    Ok(Json(admins))
}

pub async fn create_admin(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateAdminRequest>,
) -> Result<Json<String>, (StatusCode, Json<ErrorResponse>)> {
    let req_admin = check_master(&claims, &state).await?;

    if payload.is_master && req_admin.email != "master@master.master" {
        return Err((StatusCode::FORBIDDEN, Json(ErrorResponse { error: "Apenas master@master.master pode criar novos masters.".to_string(), remaining_attempts: None })));
    }

    let temp_password = generate_temp_password(&payload.name);
    let default_password = bcrypt::hash(&temp_password, bcrypt::DEFAULT_COST).unwrap();

    let res = sqlx::query("INSERT INTO admins (name, email, phone, password, is_master, is_first_login) VALUES (?, ?, ?, ?, ?, 1)")
        .bind(&payload.name).bind(&payload.email).bind(&payload.phone).bind(default_password).bind(payload.is_master)
        .execute(&state.pool)
        .await;

    if res.is_err() {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Erro ao criar admin. Email já existe?".to_string(), remaining_attempts: None })));
    }

    sqlx::query("INSERT INTO action_logs (admin_id, action, severity) VALUES (?, ?, \"WARNING\")")
        .bind(claims.sub).bind(format!("Criou novo admin: {}", payload.email)).execute(&state.pool).await.unwrap();

    Ok(Json(format!("Administrador criado! A senha padrão é: {}", temp_password)))
}

pub async fn update_admin_status(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateAdminStatusRequest>,
) -> Result<Json<String>, (StatusCode, Json<ErrorResponse>)> {
    let req_admin = check_master(&claims, &state).await?;

    if id == 1 && req_admin.id != 1 {
        return Err((StatusCode::FORBIDDEN, Json(ErrorResponse { error: "Não pode alterar o master principal.".to_string(), remaining_attempts: None })));
    }

    let target_admin: Option<(String, bool, String)> = sqlx::query_as("SELECT email, is_master, name FROM admins WHERE id = ?").bind(id).fetch_optional(&state.pool).await.unwrap();
    if target_admin.is_none() {
        return Err((StatusCode::NOT_FOUND, Json(ErrorResponse { error: "Admin não encontrado".to_string(), remaining_attempts: None })));
    }
    let (target_email, target_is_master, target_name) = target_admin.unwrap();
    let mut response_msg = "Admin atualizado com sucesso!".to_string();

    if let Some(is_master) = payload.is_master {
        if is_master != target_is_master && req_admin.email != "master@master.master" {
            return Err((StatusCode::FORBIDDEN, Json(ErrorResponse { error: "Apenas master@master.master pode alterar nível master.".to_string(), remaining_attempts: None })));
        }
        if !is_master && target_email == "master@master.master" {
            return Err((StatusCode::FORBIDDEN, Json(ErrorResponse { error: "Não pode remover o nível master do usuário principal.".to_string(), remaining_attempts: None })));
        }
        sqlx::query("UPDATE admins SET is_master = ? WHERE id = ?").bind(is_master).bind(id).execute(&state.pool).await.unwrap();
    }

    if let Some(is_active) = payload.is_active {
        if !is_active && target_email == "master@master.master" {
            return Err((StatusCode::FORBIDDEN, Json(ErrorResponse { error: "Não pode desativar o master principal.".to_string(), remaining_attempts: None })));
        }
        sqlx::query("UPDATE admins SET is_active = ? WHERE id = ?").bind(is_active).bind(id).execute(&state.pool).await.unwrap();
    }

    if let Some(force_reset) = payload.force_password_reset {
        if force_reset {
            let temp_password = generate_temp_password(&target_name);
            let hashed_pw = bcrypt::hash(&temp_password, bcrypt::DEFAULT_COST).unwrap();
            sqlx::query("UPDATE admins SET password = ?, is_first_login = 1 WHERE id = ?").bind(hashed_pw).bind(id).execute(&state.pool).await.unwrap();
            
            sqlx::query("INSERT INTO action_logs (admin_id, action, severity) VALUES (?, ?, \"CRITICAL\")")
                .bind(claims.sub).bind(format!("Forçou reset de senha do admin: {}", target_email)).execute(&state.pool).await.unwrap();
                
            response_msg = format!("Senha resetada! A nova senha temporária é: {}", temp_password);
        }
    }

    if let Some(is_locked) = payload.is_locked {
        if !is_locked {
            sqlx::query("UPDATE admins SET is_locked = 0, failed_attempts = 0 WHERE id = ?").bind(id).execute(&state.pool).await.unwrap();
            sqlx::query("INSERT INTO action_logs (admin_id, action, severity) VALUES (?, ?, \"WARNING\")")
                .bind(claims.sub).bind(format!("Desbloqueou a conta do admin: {}", target_email)).execute(&state.pool).await.unwrap();
            response_msg = "Conta desbloqueada com sucesso!".to_string();
        }
    }

    sqlx::query("INSERT INTO action_logs (admin_id, action, severity) VALUES (?, ?, \"WARNING\")")
        .bind(claims.sub).bind(format!("Atualizou status do admin: {}", target_email)).execute(&state.pool).await.unwrap();

    Ok(Json(response_msg))
}

pub async fn get_system_logs(
    claims: Claims,
    State(state): State<AppState>,
    Query(filters): Query<LogFilters>,
) -> Result<Json<Vec<SystemLog>>, (StatusCode, Json<ErrorResponse>)> {
    check_master(&claims, &state).await?;

    let mut logs: Vec<SystemLog> = Vec::new();

    if filters.log_type.is_none() || filters.log_type.as_deref() == Some("action") {
        let mut query = String::from("SELECT id, \"action\" as log_type, severity, action as description, admin_id, animal_id, \"N/A\" as remote_ip, timestamp FROM action_logs WHERE 1=1");
        if let Some(admin_id) = filters.admin_id { query.push_str(&format!(" AND admin_id = {}", admin_id)); }
        if let Some(animal_id) = filters.animal_id { query.push_str(&format!(" AND animal_id = {}", animal_id)); }
        if let Some(ref sev) = filters.severity { query.push_str(&format!(" AND severity = \"{}\"", sev)); }
        
        #[derive(sqlx::FromRow)]
        struct ActionLogRow {
            id: i64, log_type: String, severity: String, description: String, admin_id: Option<i64>, animal_id: Option<i64>, remote_ip: String, timestamp: String
        }
        let res = sqlx::query_as::<_, ActionLogRow>(&query).fetch_all(&state.pool).await.unwrap_or_default();
        for r in res { logs.push(SystemLog { id: r.id, log_type: r.log_type, severity: r.severity, description: r.description, admin_id: r.admin_id, animal_id: r.animal_id, remote_ip: r.remote_ip, timestamp: r.timestamp }); }
    }

    if filters.log_type.is_none() || filters.log_type.as_deref() == Some("login") {
        let mut query = String::from("SELECT id, \"login\" as log_type, severity, CASE WHEN success = 1 THEN 'Login bem-sucedido: ' || email ELSE 'Falha de login: ' || email END as description, NULL as admin_id, NULL as animal_id, remote_ip, timestamp FROM login_logs WHERE 1=1");
        if let Some(ref sev) = filters.severity { query.push_str(&format!(" AND severity = \"{}\"", sev)); }
        
        #[derive(sqlx::FromRow)]
        struct LoginLogRow {
            id: i64, log_type: String, severity: String, description: String, admin_id: Option<i64>, animal_id: Option<i64>, remote_ip: String, timestamp: String
        }
        let res = sqlx::query_as::<_, LoginLogRow>(&query).fetch_all(&state.pool).await.unwrap_or_default();
        for r in res { logs.push(SystemLog { id: r.id, log_type: r.log_type, severity: r.severity, description: r.description, admin_id: r.admin_id, animal_id: r.animal_id, remote_ip: r.remote_ip, timestamp: r.timestamp }); }
    }

    if filters.log_type.is_none() || filters.log_type.as_deref() == Some("security") {
        let mut query = String::from("SELECT id, \"security\" as log_type, severity, msg as description, NULL as admin_id, NULL as animal_id, remote_ip, datetime as timestamp FROM security_warnings WHERE 1=1");
        if let Some(ref sev) = filters.severity { query.push_str(&format!(" AND severity = \"{}\"", sev)); }
        
        #[derive(sqlx::FromRow)]
        struct SecLogRow {
            id: i64, log_type: String, severity: String, description: String, admin_id: Option<i64>, animal_id: Option<i64>, remote_ip: String, timestamp: String
        }
        let res = sqlx::query_as::<_, SecLogRow>(&query).fetch_all(&state.pool).await.unwrap_or_default();
        for r in res { logs.push(SystemLog { id: r.id, log_type: r.log_type, severity: r.severity, description: r.description, admin_id: r.admin_id, animal_id: r.animal_id, remote_ip: r.remote_ip, timestamp: r.timestamp }); }
    }

    logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(Json(logs))
}
