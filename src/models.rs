use chrono::{Date, DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The canonical quote record as stored in and retrieved from the database.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Quote {
    pub id: i64,
    pub text: String,
    pub author: String,
    pub source: Option<String>,
    pub tags: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request body for POST /quotes
#[derive(Debug, Deserialize)]
pub struct CreateQuoteRequest {
    pub text: String,
    pub author: String,
    pub source: Option<String>,
    pub tags: Option<String>,
}

/// Request body for PUT /quotes/:id.
#[derive(Debug, Serialize)]
pub struct UpdateQuoteRequest {
    pub text: Option<String>,
    pub author: Option<String>,
    pub source: Option<String>,
    pub tage: Option<String>,
}

/// Query paramters for GET /quotes.
#[derive(Debug, Serialize)]
pub struct QuoteQuery {
    pub author: Option<String>,
    pub tag: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// Paginated response envelope for GET /quotes.
#[derive(Debug, Serialize)]
pub struct PaginatedQuotes {
    pub data: Vec<Quote>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}
