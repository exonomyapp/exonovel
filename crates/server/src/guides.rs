use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::{Path as AxumPath, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

use crate::{
    docs::{extract_title_and_headings, normalize_doc_locale, render_safe_gfm_html, DocHeading},
    AppState,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuideChapterSummary {
    pub slug: String,
    pub chapter_order: i32,
    pub locale: String,
    pub title: String,
    pub summary: String,
    pub audience: String,
    pub reading_minutes: usize,
    pub word_count: usize,
    pub translation_status: String,
    pub willow_namespace: String,
    pub willow_subspace: String,
    pub willow_path: String,
    pub willow_timestamp_us: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuideChapterDetail {
    pub slug: String,
    pub chapter_order: i32,
    pub locale: String,
    pub title: String,
    pub summary: String,
    pub audience: String,
    pub raw_markdown: String,
    pub html: String,
    pub headings: Vec<DocHeading>,
    pub reading_minutes: usize,
    pub word_count: usize,
    pub translation_status: String,
    pub willow_namespace: String,
    pub willow_subspace: String,
    pub willow_path: String,
    pub willow_timestamp_us: i64,
}

#[derive(Deserialize)]
pub struct GuideLocaleQuery {
    pub locale: Option<String>,
}

fn now_timestamp_us() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as i64)
        .unwrap_or(0)
}

pub async fn ensure_guides_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS dear_world_guides (
            slug TEXT PRIMARY KEY,
            chapter_order INTEGER NOT NULL,
            audience TEXT NOT NULL DEFAULT 'High School & University Writers',
            willow_namespace TEXT NOT NULL DEFAULT 'exonovel',
            willow_subspace TEXT NOT NULL DEFAULT 'public/dear-world',
            willow_path TEXT NOT NULL,
            willow_timestamp_us BIGINT NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS dear_world_guide_translations (
            slug TEXT NOT NULL REFERENCES dear_world_guides(slug) ON DELETE CASCADE,
            locale TEXT NOT NULL,
            title TEXT NOT NULL,
            summary TEXT NOT NULL DEFAULT '',
            raw_markdown TEXT NOT NULL,
            html TEXT NOT NULL,
            headings_json JSONB NOT NULL DEFAULT '[]'::jsonb,
            reading_minutes INTEGER NOT NULL DEFAULT 1,
            word_count INTEGER NOT NULL DEFAULT 0,
            willow_namespace TEXT NOT NULL DEFAULT 'exonovel',
            willow_subspace TEXT NOT NULL DEFAULT 'public/dear-world',
            willow_path TEXT NOT NULL,
            willow_timestamp_us BIGINT NOT NULL DEFAULT 0,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            PRIMARY KEY (slug, locale)
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

fn parse_chapter_order_and_slug(file_name: &str) -> (i32, String) {
    let stem = file_name.trim_end_matches(".md");
    let digits: String = stem.chars().take_while(|c| c.is_ascii_digit()).collect();
    let order = digits.parse::<i32>().unwrap_or(99);
    (order, stem.to_string())
}

fn extract_guide_summary(markdown: &str) -> String {
    for line in markdown.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') || t == "---" || t.starts_with("- ") || t.starts_with('>') {
            continue;
        }
        return t.trim_matches('*').trim().to_string();
    }
    String::new()
}

pub async fn seed_guides_from_disk(pool: &PgPool, root_dir: &Path) {
    let en_dir = root_dir.join("docs").join("guides");
    let Ok(entries) = fs::read_dir(&en_dir) else {
        return;
    };
    let mut files: Vec<(i32, String, String, String)> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".md") {
            continue;
        }
        if let Ok(md) = fs::read_to_string(entry.path()) {
            let (order, slug) = parse_chapter_order_and_slug(&name);
            files.push((order, slug, name, md));
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    let ts_us = now_timestamp_us();
    for (order, slug, file_name, en_md) in &files {
        let willow_guide_path = format!("/dear-world/{slug}");
        let _ = sqlx::query(
            r#"
            INSERT INTO dear_world_guides (
                slug, chapter_order, audience, willow_namespace, willow_subspace, willow_path, willow_timestamp_us
            )
            VALUES ($1, $2, 'High School & University Writers', 'exonovel', 'public/dear-world', $3, $4)
            ON CONFLICT (slug) DO UPDATE SET
                chapter_order = EXCLUDED.chapter_order,
                willow_path = EXCLUDED.willow_path,
                willow_timestamp_us = EXCLUDED.willow_timestamp_us,
                updated_at = NOW()
            "#,
        )
        .bind(slug)
        .bind(*order)
        .bind(&willow_guide_path)
        .bind(ts_us)
        .execute(pool)
        .await;

        let mut locale_variants: Vec<(&str, String)> = vec![("en", en_md.clone())];
        for loc in ["de", "pl"] {
            let loc_path = root_dir
                .join("docs")
                .join("i18n")
                .join(loc)
                .join("guides")
                .join(file_name);
            if let Ok(loc_md) = fs::read_to_string(&loc_path) {
                locale_variants.push((loc, loc_md));
            }
        }

        for (loc, md) in locale_variants {
            let (title, headings, word_count) = extract_title_and_headings(&md, file_name);
            let reading_minutes = ((word_count as f64) / 200.0).ceil().max(1.0) as i32;
            let summary = extract_guide_summary(&md);
            let html = render_safe_gfm_html(&md);
            let headings_json = serde_json::to_value(&headings).unwrap_or_else(|_| serde_json::json!([]));
            let willow_loc_path = format!("/dear-world/{loc}/{slug}");

            let _ = sqlx::query(
                r#"
                INSERT INTO dear_world_guide_translations (
                    slug, locale, title, summary, raw_markdown, html, headings_json,
                    reading_minutes, word_count, willow_namespace, willow_subspace, willow_path, willow_timestamp_us
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'exonovel', 'public/dear-world', $10, $11)
                ON CONFLICT (slug, locale) DO UPDATE SET
                    title = EXCLUDED.title,
                    summary = EXCLUDED.summary,
                    raw_markdown = EXCLUDED.raw_markdown,
                    html = EXCLUDED.html,
                    headings_json = EXCLUDED.headings_json,
                    reading_minutes = EXCLUDED.reading_minutes,
                    word_count = EXCLUDED.word_count,
                    willow_path = EXCLUDED.willow_path,
                    willow_timestamp_us = EXCLUDED.willow_timestamp_us,
                    updated_at = NOW()
                "#,
            )
            .bind(slug)
            .bind(loc)
            .bind(&title)
            .bind(&summary)
            .bind(&md)
            .bind(&html)
            .bind(&headings_json)
            .bind(reading_minutes)
            .bind(word_count as i32)
            .bind(&willow_loc_path)
            .bind(ts_us)
            .execute(pool)
            .await;
        }
    }
}

fn load_disk_guides(root_dir: &Path, req_locale: &str) -> Vec<GuideChapterDetail> {
    let en_dir = root_dir.join("docs").join("guides");
    let Ok(entries) = fs::read_dir(&en_dir) else {
        return Vec::new();
    };
    let mut raw_list = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".md") {
            continue;
        }
        let (order, slug) = parse_chapter_order_and_slug(&name);
        let mut chosen_md = None;
        let mut status = if req_locale == "en" {
            "canonical"
        } else {
            "en-fallback"
        };
        if req_locale != "en" {
            let loc_path = root_dir
                .join("docs")
                .join("i18n")
                .join(req_locale)
                .join("guides")
                .join(&name);
            if let Ok(md) = fs::read_to_string(&loc_path) {
                chosen_md = Some(md);
                status = "translated";
            }
        }
        let md = chosen_md.unwrap_or_else(|| fs::read_to_string(entry.path()).unwrap_or_default());
        let (title, headings, word_count) = extract_title_and_headings(&md, &name);
        let reading_minutes = ((word_count as f64) / 200.0).ceil().max(1.0) as usize;
        let summary = extract_guide_summary(&md);
        let html = render_safe_gfm_html(&md);
        let eff_locale = if status == "translated" { req_locale } else { "en" };
        raw_list.push(GuideChapterDetail {
            slug: slug.clone(),
            chapter_order: order,
            locale: eff_locale.to_string(),
            title,
            summary,
            audience: "High School & University Writers".to_string(),
            raw_markdown: md,
            html,
            headings,
            reading_minutes,
            word_count,
            translation_status: status.to_string(),
            willow_namespace: "exonovel".to_string(),
            willow_subspace: "public/dear-world".to_string(),
            willow_path: format!("/dear-world/{eff_locale}/{slug}"),
            willow_timestamp_us: 0,
        });
    }
    raw_list.sort_by(|a, b| a.chapter_order.cmp(&b.chapter_order).then(a.slug.cmp(&b.slug)));
    raw_list
}

pub async fn api_list_guides(
    State(state): State<AppState>,
    Query(query): Query<GuideLocaleQuery>,
) -> impl IntoResponse {
    let req_locale = normalize_doc_locale(query.locale.as_deref().unwrap_or("en"));

    if let Some(pool) = state.db.as_ref() {
        if let Ok(rows) = sqlx::query(
            r#"
            SELECT g.slug,
                   g.chapter_order,
                   g.audience,
                   COALESCE(loc.locale, en.locale, 'en') AS eff_locale,
                   COALESCE(loc.title, en.title, g.slug) AS title,
                   COALESCE(loc.summary, en.summary, '') AS summary,
                   COALESCE(loc.reading_minutes, en.reading_minutes, 1) AS reading_minutes,
                   COALESCE(loc.word_count, en.word_count, 0) AS word_count,
                   COALESCE(loc.willow_namespace, g.willow_namespace) AS willow_namespace,
                   COALESCE(loc.willow_subspace, g.willow_subspace) AS willow_subspace,
                   COALESCE(loc.willow_path, g.willow_path) AS willow_path,
                   COALESCE(loc.willow_timestamp_us, g.willow_timestamp_us) AS willow_timestamp_us,
                   CASE
                       WHEN $1 = 'en' THEN 'canonical'
                       WHEN loc.slug IS NOT NULL THEN 'translated'
                       ELSE 'en-fallback'
                   END AS translation_status
            FROM dear_world_guides g
            LEFT JOIN dear_world_guide_translations en
              ON en.slug = g.slug AND en.locale = 'en'
            LEFT JOIN dear_world_guide_translations loc
              ON loc.slug = g.slug AND loc.locale = $1
            ORDER BY g.chapter_order ASC, g.slug ASC
            "#,
        )
        .bind(req_locale)
        .fetch_all(pool)
        .await
        {
            if !rows.is_empty() {
                let guides: Vec<GuideChapterSummary> = rows
                    .into_iter()
                    .map(|r| GuideChapterSummary {
                        slug: r.try_get("slug").unwrap_or_default(),
                        chapter_order: r.try_get("chapter_order").unwrap_or(1),
                        locale: r.try_get("eff_locale").unwrap_or_else(|_| "en".to_string()),
                        title: r.try_get("title").unwrap_or_default(),
                        summary: r.try_get("summary").unwrap_or_default(),
                        audience: r
                            .try_get("audience")
                            .unwrap_or_else(|_| "High School & University Writers".to_string()),
                        reading_minutes: r.try_get::<i32, _>("reading_minutes").unwrap_or(1).max(1) as usize,
                        word_count: r.try_get::<i32, _>("word_count").unwrap_or(0).max(0) as usize,
                        translation_status: r
                            .try_get("translation_status")
                            .unwrap_or_else(|_| "canonical".to_string()),
                        willow_namespace: r
                            .try_get("willow_namespace")
                            .unwrap_or_else(|_| "exonovel".to_string()),
                        willow_subspace: r
                            .try_get("willow_subspace")
                            .unwrap_or_else(|_| "public/dear-world".to_string()),
                        willow_path: r.try_get("willow_path").unwrap_or_default(),
                        willow_timestamp_us: r.try_get("willow_timestamp_us").unwrap_or(0),
                    })
                    .collect();

                return (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "guides": guides,
                        "willowNamespace": "exonovel/public/dear-world",
                    })),
                )
                    .into_response();
            }
        }
    }

    let disk_guides: Vec<GuideChapterSummary> = load_disk_guides(&state.root_dir, req_locale)
        .into_iter()
        .map(|g| GuideChapterSummary {
            slug: g.slug,
            chapter_order: g.chapter_order,
            locale: g.locale,
            title: g.title,
            summary: g.summary,
            audience: g.audience,
            reading_minutes: g.reading_minutes,
            word_count: g.word_count,
            translation_status: g.translation_status,
            willow_namespace: g.willow_namespace,
            willow_subspace: g.willow_subspace,
            willow_path: g.willow_path,
            willow_timestamp_us: g.willow_timestamp_us,
        })
        .collect();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "guides": disk_guides,
            "willowNamespace": "exonovel/public/dear-world",
        })),
    )
        .into_response()
}

pub async fn api_get_guide(
    State(state): State<AppState>,
    AxumPath(slug): AxumPath<String>,
    Query(query): Query<GuideLocaleQuery>,
) -> impl IntoResponse {
    let clean_slug = slug.trim();
    if clean_slug.is_empty()
        || !clean_slug
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid guide slug." })),
        )
            .into_response();
    }

    let req_locale = normalize_doc_locale(query.locale.as_deref().unwrap_or("en"));

    if let Some(pool) = state.db.as_ref() {
        if let Ok(Some(r)) = sqlx::query(
            r#"
            SELECT g.slug,
                   g.chapter_order,
                   g.audience,
                   COALESCE(loc.locale, en.locale, 'en') AS eff_locale,
                   COALESCE(loc.title, en.title, g.slug) AS title,
                   COALESCE(loc.summary, en.summary, '') AS summary,
                   COALESCE(loc.raw_markdown, en.raw_markdown, '') AS raw_markdown,
                   COALESCE(loc.html, en.html, '') AS html,
                   COALESCE(loc.headings_json, en.headings_json, '[]'::jsonb) AS headings_json,
                   COALESCE(loc.reading_minutes, en.reading_minutes, 1) AS reading_minutes,
                   COALESCE(loc.word_count, en.word_count, 0) AS word_count,
                   COALESCE(loc.willow_namespace, g.willow_namespace) AS willow_namespace,
                   COALESCE(loc.willow_subspace, g.willow_subspace) AS willow_subspace,
                   COALESCE(loc.willow_path, g.willow_path) AS willow_path,
                   COALESCE(loc.willow_timestamp_us, g.willow_timestamp_us) AS willow_timestamp_us,
                   CASE
                       WHEN $2 = 'en' THEN 'canonical'
                       WHEN loc.slug IS NOT NULL THEN 'translated'
                       ELSE 'en-fallback'
                   END AS translation_status
            FROM dear_world_guides g
            LEFT JOIN dear_world_guide_translations en
              ON en.slug = g.slug AND en.locale = 'en'
            LEFT JOIN dear_world_guide_translations loc
              ON loc.slug = g.slug AND loc.locale = $2
            WHERE g.slug = $1
            "#,
        )
        .bind(clean_slug)
        .bind(req_locale)
        .fetch_optional(pool)
        .await
        {
            let headings_val: serde_json::Value =
                r.try_get("headings_json").unwrap_or_else(|_| serde_json::json!([]));
            let headings: Vec<DocHeading> =
                serde_json::from_value(headings_val).unwrap_or_default();

            let detail = GuideChapterDetail {
                slug: r.try_get("slug").unwrap_or_default(),
                chapter_order: r.try_get("chapter_order").unwrap_or(1),
                locale: r.try_get("eff_locale").unwrap_or_else(|_| "en".to_string()),
                title: r.try_get("title").unwrap_or_default(),
                summary: r.try_get("summary").unwrap_or_default(),
                audience: r
                    .try_get("audience")
                    .unwrap_or_else(|_| "High School & University Writers".to_string()),
                raw_markdown: r.try_get("raw_markdown").unwrap_or_default(),
                html: r.try_get("html").unwrap_or_default(),
                headings,
                reading_minutes: r.try_get::<i32, _>("reading_minutes").unwrap_or(1).max(1) as usize,
                word_count: r.try_get::<i32, _>("word_count").unwrap_or(0).max(0) as usize,
                translation_status: r
                    .try_get("translation_status")
                    .unwrap_or_else(|_| "canonical".to_string()),
                willow_namespace: r
                    .try_get("willow_namespace")
                    .unwrap_or_else(|_| "exonovel".to_string()),
                willow_subspace: r
                    .try_get("willow_subspace")
                    .unwrap_or_else(|_| "public/dear-world".to_string()),
                willow_path: r.try_get("willow_path").unwrap_or_default(),
                willow_timestamp_us: r.try_get("willow_timestamp_us").unwrap_or(0),
            };
            return (StatusCode::OK, Json(detail)).into_response();
        }
    }

    if let Some(found) = load_disk_guides(&state.root_dir, req_locale)
        .into_iter()
        .find(|g| g.slug == clean_slug)
    {
        return (StatusCode::OK, Json(found)).into_response();
    }

    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Guide chapter not found." })),
    )
        .into_response()
}
