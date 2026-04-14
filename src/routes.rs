use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use sqlx::SqlitePool;

use crate::errors::ApiError;
use crate::models::{CreateQuoteRequest, QuoteQuery, UpdateQuoteRequest};
use crate::{auth, db};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub api_key: String,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/quotes", axum::routing::get(list_quotes))
        .route("/quotes/{id}", axum::routing::get(get_quote))
        .route("/quotes", axum::routing::post(create_quote))
        .route("/quotes/{id}", axum::routing::put(update_quote))
        .route("/quotes/{id}", axum::routing::delete(delete_quote))
        .with_state(state)
}

fn require_auth(headers: &HeaderMap, api_key: &str) -> Result<(), ApiError> {
    let header_value = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;

    let token = auth::extract_bearer_token(header_value).ok_or(ApiError::Unauthorized)?;

    if auth::verify_api_key(token, api_key) {
        Ok(())
    } else {
        Err(ApiError::Unauthorized)
    }
}

pub async fn list_quotes(
    State(state): State<AppState>,
    Query(params): Query<QuoteQuery>,
) -> Result<Response, ApiError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(10).clamp(1, 100);

    let result = db::list_quotes(&state.pool, params.author, params.tag, page, per_page).await?;

    Ok(Json(result).into_response())
}

pub async fn get_quote(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    let quote = db::get_quote(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(quote).into_response())
}

pub async fn create_quote(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateQuoteRequest>,
) -> Result<Response, ApiError> {
    todo!()
}

pub async fn update_quote(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<UpdateQuoteRequest>,
) -> Result<Response, ApiError> {
    todo!()
}

pub async fn delete_quote(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CreateQuoteRequest, Quote};
    use axum_test::TestServer;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup() -> (TestServer, SqlitePool) {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("failed to create pool");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("failed to run migrations");

        let state = AppState {
            pool: pool.clone(),
            api_key: "testkey".to_string(),
        };

        let router = build_router(state);
        let server = TestServer::new(router).expect("failed to build test server");
        (server, pool)
    }

    async fn insert_test_quote(pool: &SqlitePool) -> Quote {
        db::insert_quote(
            pool,
            CreateQuoteRequest {
                text: "Test quote".to_string(),
                author: "Test Author".to_string(),
                source: Some("Test Source".to_string()),
                tags: Some("rust,test".to_string()),
            },
        )
        .await
        .expect("failed to insert test quote")
    }

    // --- GET /quotes ---

    #[tokio::test]
    async fn list_quotes_returns_empty_array() {
        let (server, _pool) = setup().await;
        let response = server.get("/quotes").await;

        response.assert_status_ok();
        let body = response.json::<serde_json::Value>();
        assert_eq!(body["data"], serde_json::json!([]));
        assert_eq!(body["total"], 0);
    }

    #[tokio::test]
    async fn list_quotes_returns_inserted_quote() {
        let (server, pool) = setup().await;
        insert_test_quote(&pool).await;

        let response = server.get("/quotes").await;
        response.assert_status_ok();

        let body = response.json::<serde_json::Value>();
        assert_eq!(body["total"], 1);
        assert_eq!(body["data"][0]["author"], "Test Author");
    }

    #[tokio::test]
    async fn list_quotes_filters_by_author() {
        let (server, pool) = setup().await;
        insert_test_quote(&pool).await;

        let response = server
            .get("/quotes")
            .add_query_param("author", "Test Author")
            .await;
        response.assert_status_ok();
        let body = response.json::<serde_json::Value>();
        assert_eq!(body["total"], 1);

        let response = server
            .get("/quotes")
            .add_query_param("author", "Nobody")
            .await;
        let body = response.json::<serde_json::Value>();
        assert_eq!(body["total"], 0);
    }

    #[tokio::test]
    async fn list_quotes_filters_by_tag() {
        let (server, pool) = setup().await;
        insert_test_quote(&pool).await;

        let response = server.get("/quotes").add_query_param("tag", "rust").await;
        response.assert_status_ok();
        let body = response.json::<serde_json::Value>();
        assert_eq!(body["total"], 1);

        let response = server.get("/quotes").add_query_param("tag", "python").await;
        let body = response.json::<serde_json::Value>();
        assert_eq!(body["total"], 0);
    }

    #[tokio::test]
    async fn list_quotes_paginates() {
        let (server, pool) = setup().await;
        insert_test_quote(&pool).await;
        insert_test_quote(&pool).await;
        insert_test_quote(&pool).await;

        let response = server
            .get("/quotes")
            .add_query_param("page", "1")
            .add_query_param("per_page", "2")
            .await;

        response.assert_status_ok();
        let body = response.json::<serde_json::Value>();
        assert_eq!(body["data"].as_array().unwrap().len(), 2);
        assert_eq!(body["total"], 3);
        assert_eq!(body["page"], 1);
        assert_eq!(body["per_page"], 2);
    }

    // --- GET /quotes/:id ---

    #[tokio::test]
    async fn get_quote_returns_correct_quote() {
        let (server, pool) = setup().await;
        let created = insert_test_quote(&pool).await;

        let response = server.get(&format!("/quotes/{}", created.id)).await;
        response.assert_status_ok();

        let body = response.json::<serde_json::Value>();
        assert_eq!(body["id"], created.id);
        assert_eq!(body["author"], "Test Author");
    }

    #[tokio::test]
    async fn get_quote_returns_404_for_missing_id() {
        let (server, _pool) = setup().await;
        let response = server.get("/quotes/9999").await;
        response.assert_status(StatusCode::NOT_FOUND);
    }
}
