use axum::{
    extract::{ConnectInfo, Multipart, Path, State},
    http::StatusCode,
    Json,
};
use bcrypt::verify;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::{env, net::SocketAddr};

use crate::models::{
    AdminRecord, AnimalDetailResponse, AnimalListItem, AnimalRow, AppState, ChangePasswordRequest,
    Claims, ErrorResponse, LoginRequest, LoginResponse, StatusPayload, PhotoInfo, 
    UpdatePreferencesRequest, DashboardResponse
};

pub async fn login_handler(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    let ip = addr.ip().to_string();
    let user_agent = headers.get(axum::http::header::USER_AGENT).and_then(|v| v.to_str().ok()).unwrap_or("Desconhecido").to_string();
    
    let admin_opt = sqlx::query_as::<_, AdminRecord>("SELECT id, password, is_active, is_master, is_first_login, pref_show_inactive, pref_show_others, pref_sort_by, email, failed_attempts, is_locked FROM admins WHERE email = ?").bind(&payload.email).fetch_optional(&state.pool).await.map_err(|_| {(StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Erro no banco".to_string(), remaining_attempts: None }))})?;
    
    if let Some(admin) = &admin_opt {
        if admin.is_locked {
            let _ = sqlx::query("INSERT INTO login_logs (email, success, remote_ip, severity) VALUES (?, 0, ?, 'WARNING')")
                .bind(&payload.email).bind(&ip).execute(&state.pool).await;
            return Err((StatusCode::FORBIDDEN, Json(ErrorResponse { error: "Sua conta foi bloqueada por excesso de tentativas. Contate um administrador Master.".to_string(), remaining_attempts: None })));
        }
    }

    let (is_valid, admin_id, is_first, pref_inactive, pref_others, pref_sort, is_master, failed_attempts) = match &admin_opt { 
        Some(a) => (a.is_active && verify(&payload.password, &a.password).unwrap_or(false), Some(a.id), a.is_first_login, a.pref_show_inactive, a.pref_show_others, a.pref_sort_by.clone(), a.is_master, a.failed_attempts), 
        None => (false, None, false, false, false, "updated_desc".to_string(), false, 0) 
    };
    
    let severity = if is_valid { "INFO" } else { "WARNING" };
    let _ = sqlx::query("INSERT INTO login_logs (email, success, remote_ip, severity) VALUES (?, ?, ?, ?)")
        .bind(&payload.email).bind(is_valid).bind(&ip).bind(severity).execute(&state.pool).await;
        
    if !is_valid {
        let mut remaining = None;
        if let Some(id) = admin_id {
            if payload.email == "master@master.master" {
                let recent_fails: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM login_logs WHERE remote_ip = ? AND success = 0 AND timestamp >= datetime('now', '-5 minutes')")
                    .bind(&ip).fetch_one(&state.pool).await.unwrap_or(0);
                    
                if recent_fails >= 5 {
                    let msg = format!("Possível ataque de força bruta! {} falhas de login recentes do IP: {} visando {}", recent_fails, ip, payload.email);
                    let _ = sqlx::query("INSERT INTO security_warnings (msg, remote_ip, endpoint, user_agent, severity) VALUES (?, ?, ?, ?, ?)")
                        .bind(&msg).bind(&ip).bind("/api/auth/login").bind(&user_agent).bind("CRITICAL").execute(&state.pool).await;
                }
            } else {
                let new_fails = failed_attempts + 1;
                if new_fails >= 5 {
                    sqlx::query("UPDATE admins SET is_locked = 1, failed_attempts = ? WHERE id = ?").bind(new_fails).bind(id).execute(&state.pool).await.unwrap();
                    let msg = format!("Conta bloqueada por força bruta! Múltiplas falhas no usuário {} a partir do IP: {}", payload.email, ip);
                    let _ = sqlx::query("INSERT INTO security_warnings (msg, remote_ip, endpoint, user_agent, severity) VALUES (?, ?, ?, ?, ?)")
                        .bind(&msg).bind(&ip).bind("/api/auth/login").bind(&user_agent).bind("CRITICAL").execute(&state.pool).await;
                } else {
                    sqlx::query("UPDATE admins SET failed_attempts = ? WHERE id = ?").bind(new_fails).bind(id).execute(&state.pool).await.unwrap();
                    remaining = Some(5 - new_fails);
                }
            }
        } else {
            let recent_fails: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM login_logs WHERE remote_ip = ? AND success = 0 AND timestamp >= datetime('now', '-5 minutes')")
                .bind(&ip).fetch_one(&state.pool).await.unwrap_or(0);
                
            if recent_fails >= 5 {
                let msg = format!("Possível ataque de força bruta! {} falhas de login recentes do IP: {} visando {}", recent_fails, ip, payload.email);
                let _ = sqlx::query("INSERT INTO security_warnings (msg, remote_ip, endpoint, user_agent, severity) VALUES (?, ?, ?, ?, ?)")
                    .bind(&msg).bind(&ip).bind("/api/auth/login").bind(&user_agent).bind("CRITICAL").execute(&state.pool).await;
            }
        }
        return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Credenciais inválidas".to_string(), remaining_attempts: remaining }))); 
    }
    
    if admin_id.is_some() {
        sqlx::query("UPDATE admins SET failed_attempts = 0 WHERE id = ?").bind(admin_id.unwrap()).execute(&state.pool).await.unwrap();
    }
    let expiration = Utc::now().checked_add_signed(Duration::hours(24)).expect("Erro").timestamp() as usize;
    let claims = Claims { sub: admin_id.unwrap(), exp: expiration };
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET não configurada");
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref())).unwrap();
    Ok(Json(LoginResponse { token, is_master, is_first_login: is_first, pref_show_inactive: pref_inactive, pref_show_others: pref_others, pref_sort_by: pref_sort }))
}

pub async fn change_password(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<String>, (StatusCode, Json<ErrorResponse>)> {
    let hashed_pw = bcrypt::hash(payload.new_password, bcrypt::DEFAULT_COST).unwrap();
    sqlx::query("UPDATE admins SET password = ?, is_first_login = 0 WHERE id = ?").bind(hashed_pw).bind(claims.sub).execute(&state.pool).await.unwrap();
    sqlx::query("INSERT INTO action_logs (admin_id, action, severity, animal_id) VALUES (?, ?, 'WARNING', NULL)").bind(claims.sub).bind("Redefiniu a senha").execute(&state.pool).await.unwrap();
    Ok(Json("Senha updated!".to_string()))
}

pub async fn update_preferences(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<UpdatePreferencesRequest>,
) -> Result<Json<String>, (StatusCode, Json<ErrorResponse>)> {
    sqlx::query("UPDATE admins SET pref_show_inactive = ?, pref_show_others = ?, pref_sort_by = ? WHERE id = ?")
        .bind(payload.pref_show_inactive)
        .bind(payload.pref_show_others)
        .bind(payload.pref_sort_by)
        .bind(claims.sub)
        .execute(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Erro ao salvar preferências".to_string(), remaining_attempts: None })))?;
    Ok(Json("Preferências salvas".to_string()))
}

pub async fn get_dashboard(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<DashboardResponse>, (StatusCode, Json<ErrorResponse>)> {
    let admin = sqlx::query_as::<_, AdminRecord>("SELECT id, password, is_active, is_master, is_first_login, pref_show_inactive, pref_show_others, pref_sort_by, email, failed_attempts, is_locked FROM admins WHERE id = ?")
        .bind(claims.sub)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            println!("DB ERROR in get_dashboard: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Erro no banco".to_string(), remaining_attempts: None }))
        })?;

    match admin {
        Some(a) => Ok(Json(DashboardResponse {
            admin_id: a.id,
            is_master: a.is_master,
            pref_show_inactive: a.pref_show_inactive,
            pref_show_others: a.pref_show_others,
            pref_sort_by: a.pref_sort_by,
        })),
        None => Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Usuário inválido".to_string(), remaining_attempts: None })))
    }
}

pub async fn create_animal(
    claims: Claims,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<String>, (StatusCode, Json<ErrorResponse>)> {
    let (mut name, mut species, mut birth_year, mut breed, mut is_vaccinated, mut is_dewormed) = (String::new(), String::new(), 0, String::from("Sem raça definida"), false, false);
    let (mut behavior_dogs, mut behavior_humans, mut independence, mut size, mut coat_color, mut coat_length, mut description) = (String::new(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new());
    let mut photos: Vec<String> = Vec::new();
    let mut diseases: Vec<String> = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();
        if field_name == "photo" {
            if let Ok(data) = field.bytes().await {
                if !data.is_empty() { if let Ok(path) = crate::image_utils::process_and_save_image(data) { photos.push(path); } }
            }
        } else if let Ok(text) = field.text().await {
            match field_name.as_str() {
                "name" => name = text, "species" => species = text, "birth_year" => birth_year = text.parse().unwrap_or(0),
                "breed" => if !text.trim().is_empty() { breed = text }, "is_vaccinated" => is_vaccinated = text == "true",
                "is_dewormed" => is_dewormed = text == "true", "behavior_dogs" => behavior_dogs = text,
                "behavior_humans" => behavior_humans = text, "independence" => independence = text, "size" => size = text,
                "coat_color" => coat_color = text, "coat_length" => coat_length = text, "description" => description = text,
                "diseases" => { if !text.trim().is_empty() { diseases = text.split(',').map(|s| s.trim().to_string()).collect(); } }
                _ => {}
            }
        }
    }

    let mut tx = state.pool.begin().await.unwrap();

    let animal_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO animals 
        (name, species, birth_year, breed, is_vaccinated, is_dewormed, behavior_dogs, behavior_humans, independence, size, coat_color, coat_length, description) 
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id
        "#
    ).bind(&name).bind(&species).bind(birth_year).bind(&breed).bind(is_vaccinated).bind(is_dewormed).bind(&behavior_dogs).bind(&behavior_humans).bind(&independence).bind(&size).bind(&coat_color).bind(&coat_length).bind(&description).fetch_one(&mut *tx).await.unwrap();

    for disease in &diseases { sqlx::query("INSERT INTO animal_diseases (animal_id, disease_name) VALUES (?, ?)").bind(animal_id).bind(disease).execute(&mut *tx).await.unwrap(); }
    for (index, path) in photos.iter().enumerate() { sqlx::query("INSERT INTO animal_photos (animal_id, file_path, is_primary, is_active) VALUES (?, ?, ?, 1)").bind(animal_id).bind(path).bind(index == 0).execute(&mut *tx).await.unwrap(); }
    
    sqlx::query("INSERT INTO animal_tutors (animal_id, admin_id) VALUES (?, ?)").bind(animal_id).bind(claims.sub).execute(&mut *tx).await.unwrap();
    let admin_name: String = sqlx::query_scalar("SELECT name FROM admins WHERE id = ?").bind(claims.sub).fetch_one(&mut *tx).await.unwrap_or_else(|_| "Desconhecido".to_string());
    sqlx::query("INSERT INTO action_logs (admin_id, action, severity, animal_id) VALUES (?, ?, 'WARNING', ?)")
        .bind(claims.sub)
        .bind(format!("O administrador {} (ID: {}) cadastrou o animal \"{}\" (ID: {})", admin_name, claims.sub, name, animal_id))
        .bind(animal_id)
        .execute(&mut *tx).await.unwrap();
    
    tx.commit().await.unwrap();
    Ok(Json("Animal cadastrado!".to_string()))
}

pub async fn get_animals(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Vec<AnimalListItem>>, (StatusCode, Json<ErrorResponse>)> {
    
    #[derive(sqlx::FromRow)]
    struct DbAnimal {
        id: i64, name: String, species: String, is_active: bool,
        is_my_tutorship: bool, tutors_count: i32, updated_at: String
    }

    let db_animals = sqlx::query_as::<_, DbAnimal>(
        r#"
        SELECT 
            a.id, a.name, a.species, a.is_active, a.updated_at,
            EXISTS(SELECT 1 FROM animal_tutors WHERE animal_id = a.id AND admin_id = ?) as is_my_tutorship,
            (SELECT COUNT(*) FROM animal_tutors WHERE animal_id = a.id) as tutors_count
        FROM animals a
        "#
    ).bind(claims.sub).fetch_all(&state.pool).await.unwrap_or_default();

    let all_photos = sqlx::query_as::<_, (i64, String, bool, bool)>(
        "SELECT animal_id, file_path, is_active, is_primary FROM animal_photos"
    ).fetch_all(&state.pool).await.unwrap_or_default();

    let mut result = Vec::new();
    for db_a in db_animals {
        let mut photos = Vec::new();
        for (aid, path, is_act, is_prim) in &all_photos {
            if *aid == db_a.id {
                photos.push(PhotoInfo {
                    file_path: path.clone(),
                    is_active: *is_act,
                    is_primary: *is_prim,
                });
            }
        }
        
        photos.sort_by(|a, b| b.is_primary.cmp(&a.is_primary));

        result.push(AnimalListItem {
            id: db_a.id, name: db_a.name, species: db_a.species, is_active: db_a.is_active,
            is_my_tutorship: db_a.is_my_tutorship, tutors_count: db_a.tutors_count,
            updated_at: db_a.updated_at, photos
        });
    }

    Ok(Json(result))
}

pub async fn get_animal(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<AnimalDetailResponse>, (StatusCode, Json<ErrorResponse>)> {
    let animal = sqlx::query_as::<_, AnimalRow>("SELECT * FROM animals WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .unwrap();

    if animal.is_none() { 
        return Err((StatusCode::NOT_FOUND, Json(ErrorResponse { error: "Animal não encontrado".to_string(), remaining_attempts: None }))); 
    }
    let a = animal.unwrap();
    
    let diseases: Vec<String> = sqlx::query_scalar("SELECT disease_name FROM animal_diseases WHERE animal_id = ?").bind(id).fetch_all(&state.pool).await.unwrap_or_default();
    
    let photos = sqlx::query_as::<_, PhotoInfo>("SELECT file_path, is_active, is_primary FROM animal_photos WHERE animal_id = ? ORDER BY is_primary DESC").bind(id).fetch_all(&state.pool).await.unwrap_or_default();

    Ok(Json(AnimalDetailResponse {
        id: a.id, name: a.name, species: a.species, birth_year: a.birth_year, breed: a.breed.unwrap_or_default(),
        is_vaccinated: a.is_vaccinated, is_dewormed: a.is_dewormed, behavior_dogs: a.behavior_dogs,
        behavior_humans: a.behavior_humans, independence: a.independence, size: a.size, coat_color: a.coat_color,
        coat_length: a.coat_length, description: a.description, is_active: a.is_active, diseases, photos,
    }))
}

pub async fn update_animal(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    mut multipart: Multipart,
) -> Result<Json<String>, (StatusCode, Json<ErrorResponse>)> {
    let (mut name, mut species, mut birth_year, mut breed, mut is_vaccinated, mut is_dewormed) = (String::new(), String::new(), 0, String::from("Sem raça definida"), false, false);
    let (mut behavior_dogs, mut behavior_humans, mut independence, mut size, mut coat_color, mut coat_length, mut description) = (String::new(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new());
    let mut photos: Vec<String> = Vec::new();
    let mut diseases: Vec<String> = Vec::new();
    let mut active_photos: Vec<String> = Vec::new();
    let mut inactive_photos: Vec<String> = Vec::new();
    let mut primary_photo = String::new(); // Captura a capa

    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();
        if field_name == "photo" {
            if let Ok(data) = field.bytes().await {
                if !data.is_empty() { if let Ok(path) = crate::image_utils::process_and_save_image(data) { photos.push(path); } }
            }
        } else if let Ok(text) = field.text().await {
            match field_name.as_str() {
                "active_photos" => active_photos.push(text),
                "inactive_photos" => inactive_photos.push(text),
                "primary_photo" => primary_photo = text,
                "name" => name = text, "species" => species = text, "birth_year" => birth_year = text.parse().unwrap_or(0),
                "breed" => if !text.trim().is_empty() { breed = text }, "is_vaccinated" => is_vaccinated = text == "true",
                "is_dewormed" => is_dewormed = text == "true", "behavior_dogs" => behavior_dogs = text,
                "behavior_humans" => behavior_humans = text, "independence" => independence = text, "size" => size = text,
                "coat_color" => coat_color = text, "coat_length" => coat_length = text, "description" => description = text,
                "diseases" => { if !text.trim().is_empty() { diseases = text.split(',').map(|s| s.trim().to_string()).collect(); } }
                _ => {}
            }
        }
    }

    let mut tx = state.pool.begin().await.unwrap();
    
    sqlx::query(
        r#"
        UPDATE animals 
        SET name=?, species=?, birth_year=?, breed=?, is_vaccinated=?, is_dewormed=?, 
            behavior_dogs=?, behavior_humans=?, independence=?, size=?, coat_color=?, 
            coat_length=?, description=?, updated_at=CURRENT_TIMESTAMP
        WHERE id=?
        "#
    )
    .bind(&name).bind(&species).bind(birth_year).bind(&breed).bind(is_vaccinated).bind(is_dewormed)
    .bind(&behavior_dogs).bind(&behavior_humans).bind(&independence).bind(&size).bind(&coat_color)
    .bind(&coat_length).bind(&description).bind(id)
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query("DELETE FROM animal_diseases WHERE animal_id = ?").bind(id).execute(&mut *tx).await.unwrap();
    for disease in &diseases { sqlx::query("INSERT INTO animal_diseases (animal_id, disease_name) VALUES (?, ?)").bind(id).bind(disease).execute(&mut *tx).await.unwrap(); }

    sqlx::query("UPDATE animal_photos SET is_active = 0, is_primary = 0 WHERE animal_id = ?").bind(id).execute(&mut *tx).await.unwrap();

    let mut has_primary = false;

    for path in active_photos {
        let is_prim = if !primary_photo.is_empty() { path == primary_photo } else { !has_primary };
        if is_prim { has_primary = true; }

        sqlx::query("UPDATE animal_photos SET is_active = 1, is_primary = ? WHERE animal_id = ? AND file_path = ?")
            .bind(is_prim).bind(id).bind(&path).execute(&mut *tx).await.unwrap();
    }

    for path in photos {
        let is_prim = !has_primary;
        if is_prim { has_primary = true; }

        sqlx::query("INSERT INTO animal_photos (animal_id, file_path, is_primary, is_active) VALUES (?, ?, ?, 1)")
            .bind(id).bind(&path).bind(is_prim).execute(&mut *tx).await.unwrap();
    }

    let admin_name: String = sqlx::query_scalar("SELECT name FROM admins WHERE id = ?").bind(claims.sub).fetch_one(&mut *tx).await.unwrap_or_else(|_| "Desconhecido".to_string());
    sqlx::query("INSERT INTO action_logs (admin_id, action, severity, animal_id) VALUES (?, ?, 'WARNING', ?)")
        .bind(claims.sub)
        .bind(format!("O administrador {} (ID: {}) editou o animal \"{}\" (ID: {})", admin_name, claims.sub, name, id))
        .bind(id)
        .execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    Ok(Json("Animal atualizado!".to_string()))
}

pub async fn update_animal_status(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<StatusPayload>,
) -> Result<Json<String>, (StatusCode, Json<ErrorResponse>)> {
    let admin_name: String = sqlx::query_scalar("SELECT name FROM admins WHERE id = ?").bind(claims.sub).fetch_one(&state.pool).await.unwrap_or_else(|_| "Desconhecido".to_string());
    let animal_name: String = sqlx::query_scalar("SELECT name FROM animals WHERE id = ?").bind(id).fetch_one(&state.pool).await.unwrap_or_else(|_| "Desconhecido".to_string());
    sqlx::query("UPDATE animals SET is_active = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?").bind(payload.is_active).bind(id).execute(&state.pool).await.unwrap();
    let action_str = if payload.is_active { "restaurou" } else { "inativou" };
    sqlx::query("INSERT INTO action_logs (admin_id, action, severity, animal_id) VALUES (?, ?, 'WARNING', ?)")
        .bind(claims.sub).bind(format!("O administrador {} (ID: {}) {} o animal \"{}\" (ID: {})", admin_name, claims.sub, action_str, animal_name, id)).bind(id).execute(&state.pool).await.unwrap();
    Ok(Json("Status alterado".to_string()))
}

pub async fn toggle_tutorship(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<String>, (StatusCode, Json<ErrorResponse>)> {
    let mut tx = state.pool.begin().await.unwrap();
    let is_tutor: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM animal_tutors WHERE animal_id = ? AND admin_id = ?)").bind(id).bind(claims.sub).fetch_one(&mut *tx).await.unwrap_or(false);

    let admin_name: String = sqlx::query_scalar("SELECT name FROM admins WHERE id = ?").bind(claims.sub).fetch_one(&mut *tx).await.unwrap_or_else(|_| "Desconhecido".to_string());
    let animal_name: String = sqlx::query_scalar("SELECT name FROM animals WHERE id = ?").bind(id).fetch_one(&mut *tx).await.unwrap_or_else(|_| "Desconhecido".to_string());

    if is_tutor {
        let count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM animal_tutors WHERE animal_id = ?").bind(id).fetch_one(&mut *tx).await.unwrap_or(0);
        if count <= 1 { return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Você é o único responsável. Adicione outro tutor antes de sair.".to_string(), remaining_attempts: None }))); }
        sqlx::query("DELETE FROM animal_tutors WHERE animal_id = ? AND admin_id = ?").bind(id).bind(claims.sub).execute(&mut *tx).await.unwrap();
        sqlx::query("UPDATE animals SET updated_at = CURRENT_TIMESTAMP WHERE id = ?").bind(id).execute(&mut *tx).await.unwrap();
        sqlx::query("INSERT INTO action_logs (admin_id, action, severity, animal_id) VALUES (?, ?, 'WARNING', ?)")
            .bind(claims.sub).bind(format!("O administrador {} (ID: {}) removeu tutoria do animal \"{}\" (ID: {})", admin_name, claims.sub, animal_name, id)).bind(id).execute(&mut *tx).await.unwrap();
        tx.commit().await.unwrap();
        Ok(Json("Tutoria removida".to_string()))
    } else {
        sqlx::query("INSERT INTO animal_tutors (animal_id, admin_id) VALUES (?, ?)").bind(id).bind(claims.sub).execute(&mut *tx).await.unwrap();
        sqlx::query("UPDATE animals SET updated_at = CURRENT_TIMESTAMP WHERE id = ?").bind(id).execute(&mut *tx).await.unwrap();
        sqlx::query("INSERT INTO action_logs (admin_id, action, severity, animal_id) VALUES (?, ?, 'WARNING', ?)")
            .bind(claims.sub).bind(format!("O administrador {} (ID: {}) assumiu tutoria do animal \"{}\" (ID: {})", admin_name, claims.sub, animal_name, id)).bind(id).execute(&mut *tx).await.unwrap();
        tx.commit().await.unwrap();
        Ok(Json("Tutoria assumida".to_string()))
    }
}