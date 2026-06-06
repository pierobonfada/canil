pub mod auth;
pub mod handlers;
pub mod image_utils;
pub mod models;

use axum::{
    routing::{get, patch, post},
    Router,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::{env, net::SocketAddr, str::FromStr};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use crate::handlers::{
    change_password, create_animal, get_animal, get_animals, get_dashboard, login_handler,
    toggle_tutorship, update_animal, update_animal_status, update_preferences,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL não configurada no .env");
    
    if env::var("JWT_SECRET").is_err() {
        panic!("ERRO: Variável de ambiente JWT_SECRET não encontrada no arquivo .env!");
    }

    let connection_options = SqliteConnectOptions::from_str(&db_url)
        .expect("URL do banco inválida")
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connection_options)
        .await
        .expect("Falha ao conectar no SQLite");

    std::fs::create_dir_all("uploads").expect("Falha ao criar diretório de uploads");
    init_db(&pool).await;

    let state = models::AppState { pool };
    let cors = CorsLayer::new().allow_origin(Any).allow_headers(Any).allow_methods(Any);

    let app = Router::new()
        .nest_service("/uploads", ServeDir::new("uploads"))
        .route("/api/auth/login", post(login_handler))
        .route("/api/auth/password", patch(change_password))
        .route("/api/auth/preferences", patch(update_preferences))
        .route("/api/dashboard", get(get_dashboard))
        .route("/api/animais", get(get_animals).post(create_animal))
        .route("/api/animais/:id", get(get_animal).put(update_animal))
        .route("/api/animais/:id/status", patch(update_animal_status))
        .route("/api/animais/:id/tutores", post(toggle_tutorship))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("Servidor rodando em http://0.0.0.0:8000");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}

async fn init_db(pool: &sqlx::SqlitePool) {
    let schema = r#"
        CREATE TABLE IF NOT EXISTS admins (
            id INTEGER PRIMARY KEY AUTOINCREMENT, 
            name TEXT NOT NULL, 
            email TEXT NOT NULL UNIQUE, 
            phone TEXT NOT NULL, 
            password TEXT NOT NULL, 
            is_first_login INTEGER NOT NULL DEFAULT 1,
            pref_show_inactive INTEGER NOT NULL DEFAULT 0,
            pref_show_others INTEGER NOT NULL DEFAULT 0,
            pref_sort_by TEXT NOT NULL DEFAULT 'updated_desc'
        );

        CREATE TABLE IF NOT EXISTS login_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT NOT NULL,
            success INTEGER NOT NULL,
            remote_ip TEXT NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS security_warnings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            datetime DATETIME DEFAULT CURRENT_TIMESTAMP,
            msg TEXT NOT NULL,
            remote_ip TEXT NOT NULL,
            endpoint TEXT NOT NULL,
            user_agent TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS action_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            admin_id INTEGER NOT NULL,
            action TEXT NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(admin_id) REFERENCES admins(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS animals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            species TEXT NOT NULL,
            birth_year INTEGER NOT NULL,
            breed TEXT,
            is_vaccinated INTEGER DEFAULT 0,
            is_dewormed INTEGER DEFAULT 0,
            behavior_dogs TEXT,
            behavior_humans TEXT,
            independence TEXT,
            size TEXT,
            coat_color TEXT,
            coat_length TEXT,
            description TEXT,
            is_active INTEGER DEFAULT 1,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS animal_diseases (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            animal_id INTEGER NOT NULL,
            disease_name TEXT NOT NULL,
            FOREIGN KEY(animal_id) REFERENCES animals(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS animal_photos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            animal_id INTEGER NOT NULL,
            file_path TEXT NOT NULL,
            is_primary INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1,
            FOREIGN KEY(animal_id) REFERENCES animals(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS animal_tutors (
            animal_id INTEGER NOT NULL,
            admin_id INTEGER NOT NULL,
            PRIMARY KEY (animal_id, admin_id),
            FOREIGN KEY(animal_id) REFERENCES animals(id) ON DELETE CASCADE,
            FOREIGN KEY(admin_id) REFERENCES admins(id) ON DELETE CASCADE
        );
    "#;

    sqlx::query(schema).execute(pool).await.expect("Falha ao criar estrutura do banco");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admins").fetch_one(pool).await.unwrap_or(0);
    if count == 0 {
        let default_password = bcrypt::hash("master", bcrypt::DEFAULT_COST).unwrap();
        sqlx::query("INSERT INTO admins (name, email, phone, password, is_first_login) VALUES (?, ?, ?, ?, 1)")
            .bind("Administrador principal do sistema")
            .bind("master@master.master")
            .bind("")
            .bind(default_password)
            .execute(pool)
            .await
            .expect("Falha ao criar admin");
    }
}