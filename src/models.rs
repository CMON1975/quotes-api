use chrono::{DateTime, Utc};
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
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateQuoteRequest {
    pub text: Option<String>,
    pub author: Option<String>,
    pub source: Option<String>,
    pub tags: Option<String>,
}

/// Query paramters for GET /quotes.
#[derive(Debug, Deserialize, Serialize)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_quote() -> Quote {
        Quote {
            id: 1,
            text: "The only way to do great work is to love what you do.".to_string(),
            author: "Steve Jobs".to_string(),
            source: Some("Stanford Commencement Speech".to_string()),
            tags: Some("work,passion".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn quote_serializes_all_fields() {
        let quote = sample_quote();
        let json = serde_json::to_value(&quote).unwrap();

        assert_eq!(json["id"], 1);
        assert_eq!(json["author"], "Steve Jobs");
        assert_eq!(json["source"], "Stanford Commencement Speech");
        assert_eq!(json["tags"], "work,passion");
    }

    #[test]
    fn quote_serializes_none_fields_as_null() {
        let quote = Quote {
            source: None,
            tags: None,
            ..sample_quote()
        };
        let json = serde_json::to_value(&quote).unwrap();

        assert!(json["source"].is_null());
        assert!(json["tags"].is_null());
    }

    #[test]
    fn create_request_deserializes_full() {
        let json = serde_json::json!({
            "text": "Hello world",
            "author": "Someone",
            "source": "A book",
            "tags": "rust,api"
        });

        let req: CreateQuoteRequest = serde_json::from_value(json).unwrap();

        assert_eq!(req.text, "Hello world");
        assert_eq!(req.author, "Someone");
        assert_eq!(req.source, Some("A book".to_string()));
        assert_eq!(req.tags, Some("rust,api".to_string()));
    }

    #[test]
    fn create_request_deserializes_minimal() {
        let json = serde_json::json!({
            "text": "Hellow world",
            "author": "Someone"
        });

        let req: CreateQuoteRequest = serde_json::from_value(json).unwrap();

        assert!(req.source.is_none());
        assert!(req.tags.is_none());
    }

    #[test]
    fn create_request_fails_without_required_fields() {
        let json = serde_json::json!({ "text": "Missing author" });
        let result: Result<CreateQuoteRequest, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn update_request_all_fields_optional() {
        let json = serde_json::json!({});
        let req: UpdateQuoteRequest = serde_json::from_value(json).unwrap();

        assert!(req.text.is_none());
        assert!(req.author.is_none());
        assert!(req.source.is_none());
        assert!(req.tags.is_none());
    }

    #[test]
    fn quote_query_all_fields_optional() {
        let json = serde_json::json!({});
        let req: QuoteQuery = serde_json::from_value(json).unwrap();

        assert!(req.author.is_none());
        assert!(req.tag.is_none());
        assert!(req.page.is_none());
        assert!(req.per_page.is_none());
    }
}
