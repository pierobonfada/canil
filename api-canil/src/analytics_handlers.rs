use axum::{
    extract::{State, ConnectInfo},
    http::{StatusCode, HeaderMap},
    Json,
};
use serde_json::Value;
use std::net::SocketAddr;

use crate::models::{
    AppState, ErrorResponse, AnalyticsEventRequest, AnalyticsDashboardResponse,
    AnalyticsSessionResponse, AnalyticsEventDetail, TopAnimalStats, StatCount, SessionFilters
};

pub async fn register_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<AnalyticsEventRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Unknown")
        .to_string();

    // Use X-Forwarded-For if available, else socket addr
    let ip = headers
        .get("x-forwarded-for")
        .map(|v| v.to_str().unwrap_or("").to_string())
        .unwrap_or_else(|| addr.ip().to_string());

    let payload_str = match payload.payload {
        Some(p) => Some(serde_json::to_string(&p).unwrap_or_default()),
        None => None,
    };

    sqlx::query(
        r#"
        INSERT INTO site_analytics (visitor_id, ip_address, user_agent, event_type, path, animal_id, payload)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&payload.visitor_id)
    .bind(&ip)
    .bind(&user_agent)
    .bind(&payload.event_type)
    .bind(&payload.path)
    .bind(payload.animal_id)
    .bind(payload_str)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Erro ao registrar evento: {}", e),
                remaining_attempts: None,
            }),
        )
    })?;

    Ok(StatusCode::CREATED)
}

pub async fn get_dashboard_stats(
    State(state): State<AppState>,
) -> Result<Json<AnalyticsDashboardResponse>, (StatusCode, Json<ErrorResponse>)> {
    let pool = &state.pool;

    let visitors_today: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT visitor_id) FROM site_analytics WHERE created_at >= datetime('now', '-1 day')").fetch_one(pool).await.unwrap_or(0);
    let visitors_week: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT visitor_id) FROM site_analytics WHERE created_at >= datetime('now', '-7 days')").fetch_one(pool).await.unwrap_or(0);
    let visitors_month: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT visitor_id) FROM site_analytics WHERE created_at >= datetime('now', '-30 days')").fetch_one(pool).await.unwrap_or(0);
    let visitors_year: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT visitor_id) FROM site_analytics WHERE created_at >= datetime('now', '-365 days')").fetch_one(pool).await.unwrap_or(0);

    let top_animals: Vec<TopAnimalStats> = sqlx::query_as(
        r#"
        SELECT a.id as animal_id, a.name, COUNT(s.id) as count 
        FROM site_analytics s 
        JOIN animals a ON s.animal_id = a.id 
        WHERE s.event_type = 'page_view' 
        GROUP BY a.id, a.name 
        ORDER BY count DESC
        "#
    ).fetch_all(pool).await.unwrap_or_default();

    let most_contacted_animals: Vec<TopAnimalStats> = sqlx::query_as(
        r#"
        SELECT a.id as animal_id, a.name, COUNT(s.id) as count 
        FROM site_analytics s 
        JOIN animals a ON s.animal_id = a.id 
        WHERE s.event_type = 'message_sent' 
        GROUP BY a.id, a.name 
        ORDER BY count DESC
        "#
    ).fetch_all(pool).await.unwrap_or_default();

    // To parse payload we can extract specific properties if we use JSON functions in SQLite 3.38+
    // But since sqlite json path might be tricky, we'll extract the raw payloads and count in Rust.
    let searches: Vec<String> = sqlx::query_scalar(
        "SELECT payload FROM site_analytics WHERE event_type = 'search' AND payload IS NOT NULL"
    ).fetch_all(pool).await.unwrap_or_default();

    let mut species_counts = std::collections::HashMap::new();
    let mut color_counts = std::collections::HashMap::new();
    let mut size_counts = std::collections::HashMap::new();
    let mut age_counts = std::collections::HashMap::new();

    for s in searches {
        if let Ok(json) = serde_json::from_str::<Value>(&s) {
            if let Some(v) = json.get("species").and_then(|v| v.as_str()) {
                *species_counts.entry(v.to_string()).or_insert(0) += 1;
            }
            if let Some(v) = json.get("predominant_color").and_then(|v| v.as_str()) {
                *color_counts.entry(v.to_string()).or_insert(0) += 1;
            }
            if let Some(v) = json.get("size").and_then(|v| v.as_str()) {
                *size_counts.entry(v.to_string()).or_insert(0) += 1;
            }
            if let Some(v) = json.get("age_category").and_then(|v| v.as_str()) {
                *age_counts.entry(v.to_string()).or_insert(0) += 1;
            }
        }
    }

    let to_stat_vec = |mut map: std::collections::HashMap<String, i64>| -> Vec<StatCount> {
        let mut v: Vec<StatCount> = map.into_iter().map(|(k, c)| StatCount { name: k, count: c }).collect();
        v.sort_by(|a, b| b.count.cmp(&a.count));
        v.truncate(5);
        v
    };

    Ok(Json(AnalyticsDashboardResponse {
        visitors_today,
        visitors_week,
        visitors_month,
        visitors_year,
        top_animals,
        most_searched_species: to_stat_vec(species_counts),
        most_searched_color: to_stat_vec(color_counts),
        most_searched_size: to_stat_vec(size_counts),
        most_searched_age: to_stat_vec(age_counts),
        most_contacted_animals,
    }))
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct SessionListResponse {
    pub visitor_id: String,
    pub ip_address: String,
    pub user_agent: String,
    pub start_time: String,
    pub end_time: String,
    pub total_events: i64,
}

pub async fn get_sessions(
    axum::extract::Query(filters): axum::extract::Query<SessionFilters>,
    State(state): State<AppState>,
) -> Result<Json<Vec<SessionListResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let pool = &state.pool;

    let mut q = sqlx::QueryBuilder::new(
        r#"
        SELECT 
            s.visitor_id, 
            MAX(s.ip_address) as ip_address, 
            MAX(s.user_agent) as user_agent, 
            MIN(s.created_at) as start_time, 
            MAX(s.created_at) as end_time, 
            COUNT(s.id) as total_events 
        FROM site_analytics s
        "#
    );

    let mut has_where = false;
    
    // Filter by animals
    if let Some(animals_str) = &filters.animals {
        if !animals_str.trim().is_empty() {
            let names: Vec<&str> = animals_str.split_whitespace().collect();
            if !names.is_empty() {
                if !has_where { q.push(" WHERE "); has_where = true; } else { q.push(" AND "); }
                q.push(" s.visitor_id IN (SELECT s2.visitor_id FROM site_analytics s2 JOIN animals a ON s2.animal_id = a.id WHERE ");
                for (i, name) in names.iter().enumerate() {
                    if i > 0 { q.push(" OR "); }
                    q.push("a.name LIKE ");
                    q.push_bind(format!("%{}%", name));
                }
                q.push(") ");
            }
        }
    }

    // Filter by date
    if let Some(date_str) = &filters.date {
        if !date_str.trim().is_empty() {
            if !has_where { q.push(" WHERE "); has_where = true; } else { q.push(" AND "); }
            q.push(" DATE(s.created_at) = ");
            q.push_bind(date_str);
        }
    }

    // Filter by IP
    if let Some(ip_str) = &filters.ip {
        if !ip_str.trim().is_empty() {
            if !has_where { q.push(" WHERE "); has_where = true; } else { q.push(" AND "); }
            q.push(" s.ip_address LIKE ");
            q.push_bind(format!("%{}%", ip_str));
        }
    }

    q.push(" GROUP BY s.visitor_id ORDER BY end_time DESC ");

    let limit = filters.limit.unwrap_or(20);
    q.push(" LIMIT ");
    q.push_bind(limit);

    if let Some(page) = filters.page {
        q.push(" OFFSET ");
        q.push_bind((page - 1) * limit);
    }

    println!("Executing query: {}", q.sql());

    let sessions: Vec<SessionListResponse> = q.build_query_as().fetch_all(pool).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string(), remaining_attempts: None }),
        )
    })?;

    Ok(Json(sessions))
}

pub async fn get_session_details(
    axum::extract::Path(visitor_id): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> Result<Json<AnalyticsSessionResponse>, (StatusCode, Json<ErrorResponse>)> {
    let pool = &state.pool;

    let events: Vec<AnalyticsEventDetail> = sqlx::query_as(
        r#"
        SELECT s.id, s.event_type, s.path, s.animal_id, a.name as animal_name, s.payload, s.created_at 
        FROM site_analytics s
        LEFT JOIN animals a ON s.animal_id = a.id
        WHERE s.visitor_id = ? 
        ORDER BY s.created_at ASC
        "#
    ).bind(&visitor_id).fetch_all(pool).await.unwrap_or_default();

    if events.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: "Sessão não encontrada".into(), remaining_attempts: None })
        ));
    }

    // get generic ip / ua
    let meta: (String, String) = sqlx::query_as("SELECT ip_address, user_agent FROM site_analytics WHERE visitor_id = ? LIMIT 1")
        .bind(&visitor_id).fetch_one(pool).await.unwrap_or(("".into(), "".into()));

    Ok(Json(AnalyticsSessionResponse {
        visitor_id,
        ip_address: Some(meta.0),
        user_agent: Some(meta.1),
        created_at: events[0].created_at.clone(),
        events,
    }))
}
