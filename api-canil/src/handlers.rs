use axum::{
    extract::{ConnectInfo, Multipart, Path, State, Query},
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
    UpdatePreferencesRequest, DashboardResponse, PublicAnimalFilters, PublicTutorContact, PublicAnimalDetail
};
// ==========================================
// CONTROLADORES DA API (HANDLERS)
// ==========================================
// No Axum, um "Handler" é uma função assíncrona que recebe requisições HTTP 
// e retorna uma resposta. O Axum usa "Extractors" (como State, Json, Path, Query)
// nos argumentos da função para extrair automaticamente dados do corpo, URL ou estado global.

// AUTENTICAÇÃO: Handler de Login
// Valida e-mail e senha usando bcrypt. Se as credenciais estiverem corretas, 
// assina e retorna um JSON Web Token (JWT).
// O JWT é 'stateless' (não precisa ser guardado no banco) e será usado nas próximas requisições.
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<String>, (StatusCode, Json<ErrorResponse>)> {
    let ip = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()).map(|s| s.to_string()).unwrap_or_else(|| addr.ip().to_string());
    
    let default_password = bcrypt::hash(&payload.new_password, bcrypt::DEFAULT_COST).unwrap();
    sqlx::query("UPDATE admins SET password = ?, is_first_login = 0 WHERE id = ?").bind(default_password).bind(claims.sub).execute(&state.pool).await.unwrap();
    sqlx::query("INSERT INTO action_logs (admin_id, action, severity, remote_ip, animal_id) VALUES (?, ?, 'WARNING', ?, NULL)").bind(claims.sub).bind("Redefiniu a senha").bind(&ip).execute(&state.pool).await.unwrap();
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
// HANDLER PROTEGIDO: Resumo do Admin
// Note o extrator 'claims: Claims'. Este é um extractor customizado (implementado em auth.rs).
// Se o token JWT não for enviado ou for inválido, o Axum nem chega a executar esta função!
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
// HANDLER MULTIPART: Cadastro de Animais
// Usa o extrator 'Multipart' para receber não apenas JSON, mas formulários com arquivos (fotos).
// *Pitfall*: Processamento de imagens pode ser custoso. Usamos uma função assíncrona externa.
pub async fn create_animal(
    claims: Claims,
    State(state): State<AppState>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    headers: axum::http::HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<String>, (StatusCode, Json<ErrorResponse>)> {
    let ip = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()).map(|s| s.to_string()).unwrap_or_else(|| addr.ip().to_string());
    let (mut name, mut species, mut birth_year, mut breed, mut is_vaccinated, mut is_dewormed) = (String::new(), String::new(), 0, String::from("Sem raça definida"), false, false);
    let (mut behavior_dogs, mut behavior_cats, mut behavior_humans, mut independence, mut size, mut coat_color, mut predominant_color, mut coat_length, mut description) = (String::new(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new());
    let mut photos: Vec<String> = Vec::new();
    let mut diseases: Vec<String> = Vec::new();
    let mut total_bytes: usize = 0;

    loop {
        match multipart.next_field().await {
            Ok(Some(mut field)) => {
                let field_name = field.name().unwrap_or("").to_string();
                if field_name == "photo" {
                    let mut data = Vec::new();
                    let mut chunk_error = false;
                    loop {
                        match field.chunk().await {
                            Ok(Some(chunk)) => data.extend_from_slice(&chunk),
                            Ok(None) => break,
                            Err(_) => { chunk_error = true; break; }
                        }
                    }
                    if chunk_error {
                        for path in &photos { let _ = std::fs::remove_file(path); }
                        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Conexão interrompida no envio da foto".to_string(), remaining_attempts: None })));
                    }
                    if !data.is_empty() { 
                        total_bytes += data.len();
                        let bytes_data = axum::body::Bytes::from(data);
                        if let Ok(Ok(path)) = tokio::task::spawn_blocking(move || crate::image_utils::process_and_save_image(bytes_data)).await { 
                            photos.push(path); 
                        } 
                    }
                } else {
                    match field.text().await {
                        Ok(text) => {
                            match field_name.as_str() {
                                "name" => name = text, "species" => species = text, "birth_year" => birth_year = text.parse().unwrap_or(0),
                                "breed" => if !text.trim().is_empty() { breed = text }, "is_vaccinated" => is_vaccinated = text == "true",
                                "is_dewormed" => is_dewormed = text == "true", "behavior_dogs" => behavior_dogs = text,
                                "behavior_cats" => behavior_cats = text, "behavior_humans" => behavior_humans = text, "independence" => independence = text, "size" => size = text,
                                "coat_color" => coat_color = text, "predominant_color" => predominant_color = text, "coat_length" => coat_length = text, "description" => description = text,
                                "diseases" => { if !text.trim().is_empty() { diseases = text.split(',').map(|s| s.trim().to_string()).collect(); } }
                                _ => {}
                            }
                        },
                        Err(_) => {
                            for path in &photos { let _ = std::fs::remove_file(path); }
                            return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Conexão interrompida no envio de texto".to_string(), remaining_attempts: None })));
                        }
                    }
                }
            },
            Ok(None) => break,
            Err(_) => {
                for path in &photos { let _ = std::fs::remove_file(path); }
                return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Conexão interrompida durante o envio".to_string(), remaining_attempts: None })));
            }
        }
    }

    let mut tx = state.pool.begin().await.unwrap();

    let animal_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO animals 
        (name, species, birth_year, breed, is_vaccinated, is_dewormed, behavior_dogs, behavior_cats, behavior_humans, independence, size, coat_color, predominant_color, coat_length, description) 
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id
        "#
    ).bind(&name).bind(&species).bind(birth_year).bind(&breed).bind(is_vaccinated).bind(is_dewormed).bind(&behavior_dogs).bind(&behavior_cats).bind(&behavior_humans).bind(&independence).bind(&size).bind(&coat_color).bind(&predominant_color).bind(&coat_length).bind(&description).fetch_one(&mut *tx).await.unwrap();

    for disease in &diseases { sqlx::query("INSERT INTO animal_diseases (animal_id, disease_name) VALUES (?, ?)").bind(animal_id).bind(disease).execute(&mut *tx).await.unwrap(); }
    for (index, path) in photos.iter().enumerate() { sqlx::query("INSERT INTO animal_photos (animal_id, file_path, is_primary, is_active) VALUES (?, ?, ?, 1)").bind(animal_id).bind(path).bind(index == 0).execute(&mut *tx).await.unwrap(); }
    
    sqlx::query("INSERT INTO animal_tutors (animal_id, admin_id) VALUES (?, ?)").bind(animal_id).bind(claims.sub).execute(&mut *tx).await.unwrap();
    let admin_name: String = sqlx::query_scalar("SELECT name FROM admins WHERE id = ?").bind(claims.sub).fetch_one(&mut *tx).await.unwrap_or_else(|_| "Desconhecido".to_string());
    let total_mb = total_bytes as f64 / 1024.0 / 1024.0;
    sqlx::query("INSERT INTO action_logs (admin_id, action, severity, remote_ip, animal_id) VALUES (?, ?, 'WARNING', ?, ?)")
        .bind(claims.sub)
        .bind(format!("O administrador {} (ID: {}) cadastrou o animal \"{}\" (ID: {}) com {:.2} MB de fotos.", admin_name, claims.sub, name, animal_id, total_mb))
        .bind(&ip)
        .bind(animal_id)
        .execute(&mut *tx).await.unwrap();
    
    tx.commit().await.unwrap();
    Ok(Json("Animal cadastrado!".to_string()))
}
// LISTAGEM DE ANIMAIS (ADMIN)
// Retorna a lista de animais formatada. O SQL usa subqueries para calcular 
// o número de tutores dinamicamente.
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
        behavior_cats: a.behavior_cats, behavior_humans: a.behavior_humans.unwrap_or_default(), independence: a.independence, size: a.size, coat_color: a.coat_color,
        predominant_color: a.predominant_color, coat_length: a.coat_length, description: a.description, is_active: a.is_active, diseases, photos,
    }))
}

pub async fn update_animal(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    headers: axum::http::HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<String>, (StatusCode, Json<ErrorResponse>)> {
    let ip = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()).map(|s| s.to_string()).unwrap_or_else(|| addr.ip().to_string());
    let (mut name, mut species, mut birth_year, mut breed, mut is_vaccinated, mut is_dewormed) = (String::new(), String::new(), 0, String::from("Sem raça definida"), false, false);
    let (mut behavior_dogs, mut behavior_cats, mut behavior_humans, mut independence, mut size, mut coat_color, mut predominant_color, mut coat_length, mut description) = (String::new(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new());
    let mut photos: Vec<String> = Vec::new();
    let mut diseases: Vec<String> = Vec::new();
    let mut active_photos: Vec<String> = Vec::new();
    let mut primary_photo = String::new();
    let mut total_bytes: usize = 0;

    loop {
        match multipart.next_field().await {
            Ok(Some(mut field)) => {
                let field_name = field.name().unwrap_or("").to_string();
                if field_name == "photo" {
                    let mut data = Vec::new();
                    let mut chunk_error = false;
                    loop {
                        match field.chunk().await {
                            Ok(Some(chunk)) => data.extend_from_slice(&chunk),
                            Ok(None) => break,
                            Err(_) => { chunk_error = true; break; }
                        }
                    }
                    if chunk_error {
                        for path in &photos { let _ = std::fs::remove_file(path); }
                        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Conexão interrompida no envio da foto".to_string(), remaining_attempts: None })));
                    }
                    if !data.is_empty() { 
                        total_bytes += data.len();
                        let bytes_data = axum::body::Bytes::from(data);
                        if let Ok(Ok(path)) = tokio::task::spawn_blocking(move || crate::image_utils::process_and_save_image(bytes_data)).await { 
                            photos.push(path); 
                        } 
                    }
                } else if field_name == "primary_photo" {
                    if let Ok(text) = field.text().await {
                        primary_photo = text.trim().to_string();
                    }
                } else {
                    match field.text().await {
                        Ok(text) => {
                            match field_name.as_str() {
                                "active_photos" => active_photos.push(text),
                                "name" => name = text, "species" => species = text, "birth_year" => birth_year = text.parse().unwrap_or(0),
                                "breed" => if !text.trim().is_empty() { breed = text }, "is_vaccinated" => is_vaccinated = text == "true",
                                "is_dewormed" => is_dewormed = text == "true", "behavior_dogs" => behavior_dogs = text,
                                "behavior_cats" => behavior_cats = text, "behavior_humans" => behavior_humans = text, "independence" => independence = text, "size" => size = text,
                                "coat_color" => coat_color = text, "predominant_color" => predominant_color = text, "coat_length" => coat_length = text, "description" => description = text,
                                "diseases" => { if !text.trim().is_empty() { diseases = text.split(',').map(|s| s.trim().to_string()).collect(); } }
                                _ => {}
                            }
                        },
                        Err(_) => {
                            for path in &photos { let _ = std::fs::remove_file(path); }
                            return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Conexão interrompida no envio de texto".to_string(), remaining_attempts: None })));
                        }
                    }
                }
            },
            Ok(None) => break,
            Err(_) => {
                for path in &photos { let _ = std::fs::remove_file(path); }
                return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Conexão interrompida durante o envio".to_string(), remaining_attempts: None })));
            }
        }
    }

    let mut tx = state.pool.begin().await.unwrap();
    
    sqlx::query(
        r#"
        UPDATE animals 
        SET name=?, species=?, birth_year=?, breed=?, is_vaccinated=?, is_dewormed=?, 
            behavior_dogs=?, behavior_cats=?, behavior_humans=?, independence=?, size=?, coat_color=?, predominant_color=?, 
            coat_length=?, description=?, updated_at=CURRENT_TIMESTAMP
        WHERE id=?
        "#
    )
    .bind(&name).bind(&species).bind(birth_year).bind(&breed).bind(is_vaccinated).bind(is_dewormed)
    .bind(&behavior_dogs).bind(&behavior_cats).bind(&behavior_humans).bind(&independence).bind(&size).bind(&coat_color).bind(&predominant_color)
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
    let total_mb = total_bytes as f64 / 1024.0 / 1024.0;
    sqlx::query("INSERT INTO action_logs (admin_id, action, severity, remote_ip, animal_id) VALUES (?, ?, 'WARNING', ?, ?)")
        .bind(claims.sub)
        .bind(format!("O administrador {} (ID: {}) editou o animal \"{}\" (ID: {}) com upload de {:.2} MB de fotos.", admin_name, claims.sub, name, id, total_mb))
        .bind(&ip)
        .bind(id)
        .execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    Ok(Json("Animal atualizado!".to_string()))
}

pub async fn update_animal_status(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<StatusPayload>,
) -> Result<Json<String>, (StatusCode, Json<ErrorResponse>)> {
    let ip = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()).map(|s| s.to_string()).unwrap_or_else(|| addr.ip().to_string());
    let admin_name: String = sqlx::query_scalar("SELECT name FROM admins WHERE id = ?").bind(claims.sub).fetch_one(&state.pool).await.unwrap_or_else(|_| "Desconhecido".to_string());
    let animal_name: String = sqlx::query_scalar("SELECT name FROM animals WHERE id = ?").bind(id).fetch_one(&state.pool).await.unwrap_or_else(|_| "Desconhecido".to_string());
    sqlx::query("UPDATE animals SET is_active = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?").bind(payload.is_active).bind(id).execute(&state.pool).await.unwrap();
    let action_str = if payload.is_active { "restaurou" } else { "inativou" };
    sqlx::query("INSERT INTO action_logs (admin_id, action, severity, remote_ip, animal_id) VALUES (?, ?, 'WARNING', ?, ?)")
        .bind(claims.sub).bind(format!("O administrador {} (ID: {}) {} o animal \"{}\" (ID: {})", admin_name, claims.sub, action_str, animal_name, id)).bind(&ip).bind(id).execute(&state.pool).await.unwrap();
    Ok(Json("Status alterado".to_string()))
}

pub async fn toggle_tutorship(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    headers: axum::http::HeaderMap,
) -> Result<Json<String>, (StatusCode, Json<ErrorResponse>)> {
    let ip = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()).map(|s| s.to_string()).unwrap_or_else(|| addr.ip().to_string());
    let mut tx = state.pool.begin().await.unwrap();
    let is_tutor: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM animal_tutors WHERE animal_id = ? AND admin_id = ?)").bind(id).bind(claims.sub).fetch_one(&mut *tx).await.unwrap_or(false);

    let admin_name: String = sqlx::query_scalar("SELECT name FROM admins WHERE id = ?").bind(claims.sub).fetch_one(&mut *tx).await.unwrap_or_else(|_| "Desconhecido".to_string());
    let animal_name: String = sqlx::query_scalar("SELECT name FROM animals WHERE id = ?").bind(id).fetch_one(&mut *tx).await.unwrap_or_else(|_| "Desconhecido".to_string());

    if is_tutor {
        let count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM animal_tutors WHERE animal_id = ?").bind(id).fetch_one(&mut *tx).await.unwrap_or(0);
        if count <= 1 { return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Você é o único responsável. Adicione outro tutor antes de sair.".to_string(), remaining_attempts: None }))); }
        sqlx::query("DELETE FROM animal_tutors WHERE animal_id = ? AND admin_id = ?").bind(id).bind(claims.sub).execute(&mut *tx).await.unwrap();
        sqlx::query("UPDATE animals SET updated_at = CURRENT_TIMESTAMP WHERE id = ?").bind(id).execute(&mut *tx).await.unwrap();
        sqlx::query("INSERT INTO action_logs (admin_id, action, severity, remote_ip, animal_id) VALUES (?, ?, 'WARNING', ?, ?)")
            .bind(claims.sub).bind(format!("O administrador {} (ID: {}) removeu tutoria do animal \"{}\" (ID: {})", admin_name, claims.sub, animal_name, id)).bind(&ip).bind(id).execute(&mut *tx).await.unwrap();
        tx.commit().await.unwrap();
        Ok(Json("Tutoria removida".to_string()))
    } else {
        sqlx::query("INSERT INTO animal_tutors (animal_id, admin_id) VALUES (?, ?)").bind(id).bind(claims.sub).execute(&mut *tx).await.unwrap();
        sqlx::query("UPDATE animals SET updated_at = CURRENT_TIMESTAMP WHERE id = ?").bind(id).execute(&mut *tx).await.unwrap();
        sqlx::query("INSERT INTO action_logs (admin_id, action, severity, remote_ip, animal_id) VALUES (?, ?, 'WARNING', ?, ?)")
            .bind(claims.sub).bind(format!("O administrador {} (ID: {}) assumiu tutoria do animal \"{}\" (ID: {})", admin_name, claims.sub, animal_name, id)).bind(&ip).bind(id).execute(&mut *tx).await.unwrap();
        tx.commit().await.unwrap();
        Ok(Json("Tutoria assumida".to_string()))
    }
}
// VITRINE PÚBLICA (SITE)
// Extrator 'Query<T>' transforma parâmetros da URL (ex: ?size=Pequeno&species=Gato)
// diretamente na struct 'PublicAnimalFilters'. Montamos uma query SQL dinâmica com o 'QueryBuilder'.
pub async fn get_public_animals(
    Query(filters): Query<PublicAnimalFilters>,
    State(state): State<AppState>,
) -> Result<Json<Vec<AnimalDetailResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let mut q = sqlx::QueryBuilder::new("SELECT * FROM animals WHERE is_active = 1");
    
    if let Some(n) = &filters.name { if !n.is_empty() { q.push(" AND name LIKE "); q.push_bind(format!("%{}%", n)); } }
    if let Some(s) = &filters.species { if !s.is_empty() { q.push(" AND species = "); q.push_bind(s); } }
    if let Some(c) = &filters.predominant_color { if !c.is_empty() { q.push(" AND predominant_color = "); q.push_bind(c); } }
    if let Some(sz) = &filters.size { if !sz.is_empty() { q.push(" AND size = "); q.push_bind(sz); } }
    if let Some(bd) = &filters.behavior_dogs { if !bd.is_empty() { q.push(" AND behavior_dogs = "); q.push_bind(bd); } }
    if let Some(bh) = &filters.behavior_cats { if !bh.is_empty() { q.push(" AND behavior_cats = "); q.push_bind(bh); } }
    if let Some(bhum) = &filters.behavior_humans { if !bhum.is_empty() { q.push(" AND behavior_humans = "); q.push_bind(bhum); } }
    
    if filters.age_min.is_some() || filters.age_max.is_some() {
        let current_year = chrono::Utc::now().naive_utc().date().format("%Y").to_string().parse::<i64>().unwrap_or(2026);
        if let Some(min) = filters.age_min { q.push(" AND birth_year <= "); q.push_bind(current_year - min); }
        if let Some(max) = filters.age_max { q.push(" AND birth_year >= "); q.push_bind(current_year - max); }
    }

    q.push(" ORDER BY (id * 97) % 100 DESC, updated_at DESC");

    if let Some(limit) = filters.limit {
        q.push(" LIMIT "); q.push_bind(limit);
        if let Some(page) = filters.page {
            q.push(" OFFSET "); q.push_bind((page - 1) * limit);
        }
    } else {
        q.push(" LIMIT 10 OFFSET 0");
    }

    let rows: Vec<AnimalRow> = q.build_query_as().fetch_all(&state.pool).await.unwrap_or_default();
    
    let mut results = Vec::new();
    for a in rows {
        let photos: Vec<PhotoInfo> = sqlx::query_as("SELECT file_path, is_active, is_primary FROM animal_photos WHERE animal_id = ? AND is_active = 1 ORDER BY is_primary DESC, id ASC")
            .bind(a.id).fetch_all(&state.pool).await.unwrap_or_default();
        let diseases: Vec<String> = sqlx::query_scalar("SELECT disease_name FROM animal_diseases WHERE animal_id = ?")
            .bind(a.id).fetch_all(&state.pool).await.unwrap_or_default();
            
        results.push(AnimalDetailResponse {
            id: a.id, name: a.name, species: a.species, birth_year: a.birth_year, breed: a.breed.unwrap_or_default(),
            is_vaccinated: a.is_vaccinated, is_dewormed: a.is_dewormed, behavior_dogs: a.behavior_dogs,
            behavior_cats: a.behavior_cats, behavior_humans: a.behavior_humans.unwrap_or_default(), independence: a.independence, size: a.size, coat_color: a.coat_color,
            predominant_color: a.predominant_color, coat_length: a.coat_length, description: a.description, is_active: a.is_active, diseases, photos,
        });
    }
    
    Ok(Json(results))
}

pub async fn get_public_animal(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<PublicAnimalDetail>, (StatusCode, Json<ErrorResponse>)> {
    let a: AnimalRow = match sqlx::query_as("SELECT * FROM animals WHERE id = ? AND is_active = 1").bind(id).fetch_optional(&state.pool).await {
        Ok(Some(animal)) => animal,
        _ => return Err((StatusCode::NOT_FOUND, Json(ErrorResponse { error: "Animal não encontrado".to_string(), remaining_attempts: None }))),
    };
    
    let photos: Vec<PhotoInfo> = sqlx::query_as("SELECT file_path, is_active, is_primary FROM animal_photos WHERE animal_id = ? AND is_active = 1 ORDER BY is_primary DESC, id ASC")
        .bind(a.id).fetch_all(&state.pool).await.unwrap_or_default();
    let diseases: Vec<String> = sqlx::query_scalar("SELECT disease_name FROM animal_diseases WHERE animal_id = ?")
        .bind(a.id).fetch_all(&state.pool).await.unwrap_or_default();
        
    let animal = AnimalDetailResponse {
        id: a.id, name: a.name, species: a.species, birth_year: a.birth_year, breed: a.breed.unwrap_or_default(),
        is_vaccinated: a.is_vaccinated, is_dewormed: a.is_dewormed, behavior_dogs: a.behavior_dogs,
        behavior_cats: a.behavior_cats, behavior_humans: a.behavior_humans.unwrap_or_default(), independence: a.independence, size: a.size, coat_color: a.coat_color,
        predominant_color: a.predominant_color, coat_length: a.coat_length, description: a.description, is_active: a.is_active, diseases, photos,
    };
    
    let tutors: Vec<PublicTutorContact> = sqlx::query_as(
        "SELECT a.name, a.phone, a.email FROM admins a JOIN animal_tutors t ON a.id = t.admin_id WHERE t.animal_id = ? AND a.is_active = 1"
    ).bind(id).fetch_all(&state.pool).await.unwrap_or_default();

    Ok(Json(PublicAnimalDetail { animal, tutors }))
}