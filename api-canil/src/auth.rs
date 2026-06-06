use axum::{
    async_trait,
    extract::{ConnectInfo, FromRef, FromRequestParts},
    http::{
        header::{AUTHORIZATION, USER_AGENT},
        request::Parts,
        StatusCode,
    },
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::{env, net::SocketAddr};

use crate::models::{AppState, Claims, ErrorResponse};

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ErrorResponse>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        
        let app_state = AppState::from_ref(state);

        let ip = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(addr)| addr.ip().to_string())
            .unwrap_or_else(|| "Desconhecido".to_string());

        let user_agent = parts
            .headers
            .get(USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("Desconhecido")
            .to_string();

        let endpoint = parts.uri.path().to_string();

        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .filter(|value| value.starts_with("Bearer "))
            .map(|value| value[7..].to_string());

        let token = match auth_header {
            Some(token) => token,
            None => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        error: "Token ausente".to_string(),
                    }),
                ));
            }
        };

        let secret = env::var("JWT_SECRET").expect("JWT_SECRET não configurada no .env");

        let token_data = match decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        ) {
            Ok(data) => data,
            Err(err) => {
                let msg = format!(
                    "Falha de segurança JWT. Motivo: {:?}. Conteúdo do Token enviado: {}",
                    err, token
                );

                let _ = sqlx::query(
                    r#"
                    INSERT INTO security_warnings 
                    (msg, remote_ip, endpoint, user_agent) 
                    VALUES (?, ?, ?, ?)
                    "#
                )
                .bind(&msg)
                .bind(&ip)
                .bind(&endpoint)
                .bind(&user_agent)
                .execute(&app_state.pool)
                .await;

                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        error: "Token inválido ou expirado".to_string(),
                    }),
                ));
            }
        };

        Ok(token_data.claims)
    }
}