use crate::models::{CreateQuoteRequest, PaginatedQuotes, Quote, UpdateQuoteRequest};
use chrono::DateTime;
use chrono::Utc;
use sqlx::{Sqlite, SqlitePool};

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
