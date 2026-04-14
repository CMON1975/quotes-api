use crate::models::{CreateQuoteRequest, PaginatedQuotes, Quote, UpdateQuoteRequest};
use chrono::DateTime;
use chrono::Utc;
use sqlx::SqlitePool;

pub async fn list_quotes(
    pool: &SqlitePool,
    author: Option<String>,
    tag: Option<String>,
    page: i64,
    per_page: i64,
) -> Result<PaginatedQuotes, sqlx::Error> {
    let offset = (page - 1) * per_page;

    let author_filter = author.as_deref().unwrap_or("").to_lowercase();
    let tag_filter = tag.as_deref().unwrap_or("").to_lowercase();

    let all_quotes = sqlx::query_as!(
        Quote,
        r#"SELECT id, text, author, source, tags,
        created_at as "created_at: DateTime<Utc>",
        updated_at as "updated_at: DateTime<Utc>"
        FROM quotes"#,
    )
    .fetch_all(pool)
    .await?;

    let filtered: Vec<Quote> = all_quotes
        .into_iter()
        .filter(|q| {
            let author_match =
                author_filter.is_empty() || q.author.to_lowercase().contains(&author_filter);
            let tag_match = tag_filter.is_empty()
                || q.tags
                    .as_deref()
                    .unwrap_or("")
                    .split(',')
                    .any(|t| t.trim().to_lowercase() == tag_filter);
            author_match && tag_match
        })
        .collect();

    let total = filtered.len() as i64;

    let data = filtered
        .into_iter()
        .skip(offset as usize)
        .take(per_page as usize)
        .collect();

    Ok(PaginatedQuotes {
        data,
        total,
        page,
        per_page,
    })
}

pub async fn get_quote(pool: &SqlitePool, id: i64) -> Result<Option<Quote>, sqlx::Error> {
    sqlx::query_as!(
        Quote,
        r#"SELECT id, text, author, source, tags,
            created_at as "created_at: DateTime<Utc>",
            updated_at as "updated_at: DateTime<Utc>"
            FROM quotes WHERE id = ?"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn insert_quote(
    pool: &SqlitePool,
    req: CreateQuoteRequest,
) -> Result<Quote, sqlx::Error> {
    let now = Utc::now();

    sqlx::query_as!(
        Quote,
        r#"INSERT INTO quotes (text, author, source, tags, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id, text, author, source, tags,
            created_at as "created_at: DateTime<Utc>",
            updated_at as "updated_at: DateTime<Utc>""#,
        req.text,
        req.author,
        req.source,
        req.tags,
        now,
        now
    )
    .fetch_one(pool)
    .await
}

pub async fn update_quote(
    pool: &SqlitePool,
    id: i64,
    req: UpdateQuoteRequest,
) -> Result<Option<Quote>, sqlx::Error> {
    let existing = get_quote(pool, id).await?;
    let Some(existing) = existing else {
        return Ok(None);
    };

    let now = Utc::now();
    let text = req.text.unwrap_or(existing.text);
    let author = req.author.unwrap_or(existing.author);
    let source = req.source.or(existing.source);
    let tags = req.tags.or(existing.tags);

    sqlx::query_as!(
        Quote,
        r#"UPDATE quotes SET text = ?, author = ?, source = ?, tags = ?, updated_at = ? 
            WHERE id = ?
            RETURNING id, text, author, source, tags,
            created_at as "created_at: DateTime<Utc>",
            updated_at as "updated_at: DateTime<Utc>""#,
        text,
        author,
        source,
        tags,
        now,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn delete_quote(pool: &SqlitePool, id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM quotes WHERE id = ?", id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CreateQuoteRequest, UpdateQuoteRequest};
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("failed to create in-memory pool");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("failed to run migrations");

        pool
    }

    fn sample_create() -> CreateQuoteRequest {
        CreateQuoteRequest {
            text: "Test quote".to_string(),
            author: "Test Author".to_string(),
            source: Some("Test Source".to_string()),
            tags: Some("rust,test".to_string()),
        }
    }

    #[tokio::test]
    async fn insert_and_get_quote() {
        let pool = setup_pool().await;
        let created = insert_quote(&pool, sample_create()).await.unwrap();

        assert_eq!(created.text, "Test quote");
        assert_eq!(created.author, "Test Author");

        let fetched = get_quote(&pool, created.id).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().id, created.id);
    }

    #[tokio::test]
    async fn get_quote_returns_none_for_missing_id() {
        let pool = setup_pool().await;
        let result = get_quote(&pool, 9999).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn list_quotes_returns_all() {
        let pool = setup_pool().await;
        insert_quote(&pool, sample_create()).await.unwrap();
        insert_quote(&pool, sample_create()).await.unwrap();

        let result = list_quotes(&pool, None, None, 1, 10).await.unwrap();
        assert_eq!(result.total, 2);
        assert_eq!(result.data.len(), 2);
    }

    #[tokio::test]
    async fn list_quotes_filters_by_author() {
        let pool = setup_pool().await;
        insert_quote(&pool, sample_create()).await.unwrap();
        insert_quote(
            &pool,
            CreateQuoteRequest {
                author: "Other Author".to_string(),
                ..sample_create()
            },
        )
        .await
        .unwrap();

        let result = list_quotes(&pool, Some("other author".to_string()), None, 1, 10)
            .await
            .unwrap();

        assert_eq!(result.total, 1);
        assert_eq!(result.data[0].author, "Other Author");
    }

    #[tokio::test]
    async fn list_quotes_filters_by_tag() {
        let pool = setup_pool().await;
        insert_quote(&pool, sample_create()).await.unwrap();
        insert_quote(
            &pool,
            CreateQuoteRequest {
                tags: Some("philosophy,life".to_string()),
                ..sample_create()
            },
        )
        .await
        .unwrap();

        let result = list_quotes(&pool, None, Some("philosophy".to_string()), 1, 10)
            .await
            .unwrap();

        assert_eq!(result.total, 1);
        assert_eq!(result.data[0].tags, Some("philosophy,life".to_string()));
    }

    #[tokio::test]
    async fn list_quotes_paginates_correctly() {
        let pool = setup_pool().await;
        for _ in 0..5 {
            insert_quote(&pool, sample_create()).await.unwrap();
        }

        let page1 = list_quotes(&pool, None, None, 1, 2).await.unwrap();
        let page2 = list_quotes(&pool, None, None, 2, 2).await.unwrap();
        let page3 = list_quotes(&pool, None, None, 3, 2).await.unwrap();

        assert_eq!(page1.data.len(), 2);
        assert_eq!(page2.data.len(), 2);
        assert_eq!(page3.data.len(), 1);
        assert_eq!(page1.total, 5);
    }

    #[tokio::test]
    async fn update_quote_patches_fields() {
        let pool = setup_pool().await;
        let created = insert_quote(&pool, sample_create()).await.unwrap();

        let updated = update_quote(
            &pool,
            created.id,
            UpdateQuoteRequest {
                text: Some("Updated text".to_string()),
                author: None,
                source: None,
                tags: None,
            },
        )
        .await
        .unwrap();

        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.text, "Updated text");
        assert_eq!(updated.author, "Test Author"); // unchanged
    }

    #[tokio::test]
    async fn update_quote_returns_none_for_missing_id() {
        let pool = setup_pool().await;
        let result = update_quote(
            &pool,
            9999,
            UpdateQuoteRequest {
                text: Some("x".to_string()),
                author: None,
                source: None,
                tags: None,
            },
        )
        .await
        .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn delete_quote_removes_record() {
        let pool = setup_pool().await;
        let created = insert_quote(&pool, sample_create()).await.unwrap();

        let deleted = delete_quote(&pool, created.id).await.unwrap();
        assert!(deleted);

        let fetched = get_quote(&pool, created.id).await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn delete_quote_returns_false_for_missing_id() {
        let pool = setup_pool().await;
        let result = delete_quote(&pool, 9999).await.unwrap();
        assert!(!result);
    }
}
