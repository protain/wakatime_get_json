#[macro_use]
extern crate rocket;
use rocket::{http::Status, Config, State};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use core::f64;
use std::env;

use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
struct RankingItem {
    title: String,
    hours: f64,
}

impl RankingItem {
    pub async fn get_editors_ranking(
        from: &str,
        to: &str,
        pool: &Pool<Postgres>,
    ) -> anyhow::Result<Vec<RankingItem>, sqlx::Error> {
        let res = sqlx::query_as::<_, RankingItem>(
            r#"
select title#>>'{}' as title, (sum(total_seconds) / 3600) hours from
    (select
        jsonb_path_query(w.data, '$.summaries.data.editors.name') title,
        jsonb_path_query(w.data, '$.summaries.data.editors.total_seconds')::double precision total_seconds
    from wakatime_dat w
    where w.date >= $1 and w.date <= $2
    ) x
group by title
order by hours desc
                "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(&*pool)
        .await;
        res
    }

    pub async fn get_lang_ranking(
        from: &str,
        to: &str,
        pool: &Pool<Postgres>,
    ) -> anyhow::Result<Vec<RankingItem>, sqlx::Error> {
        let res = sqlx::query_as::<_, RankingItem>(
            r#"
select title#>>'{}' as title, (sum(total_seconds) / 3600) hours from
    (select
        jsonb_path_query(w.data, '$.summaries.data.languages.name') title,
        jsonb_path_query(w.data, '$.summaries.data.languages.total_seconds')::double precision total_seconds
    from wakatime_dat w
    where w.date >= $1 and w.date <= $2
    ) x
group by title
order by hours desc
                "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(&*pool)
        .await;
        res
    }

    pub async fn get_projects_ranking(
        from: &str,
        to: &str,
        pool: &Pool<Postgres>,
    ) -> anyhow::Result<Vec<RankingItem>, sqlx::Error> {
        let res = sqlx::query_as::<_, RankingItem>(
            r#"
select title#>>'{}' as title, (sum(total_seconds) / 3600) hours from
    (select
        jsonb_path_query(w.data, '$.summaries.data.projects.name') title,
        jsonb_path_query(w.data, '$.summaries.data.projects.total_seconds')::double precision total_seconds
    from wakatime_dat w
    where w.date >= $1 and w.date <= $2
    ) x
group by title
order by hours desc
                "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(&*pool)
        .await;
        res
    }
}

#[get("/editors/<from>/<to>")]
async fn editors(pool: &State<Pool<Postgres>>, from: &str, to: &str) -> Result<String, Status> {
    let rank = RankingItem::get_editors_ranking(from, to, &pool).await;
    match rank {
        Ok(rank) => Ok(serde_json::to_string(&rank).unwrap()),
        _ => Err(Status::NotFound),
    }
}

#[get("/langs/<from>/<to>")]
async fn langs(pool: &State<Pool<Postgres>>, from: &str, to: &str) -> Result<String, Status> {
    let rank = RankingItem::get_lang_ranking(from, to, &pool).await;
    match rank {
        Ok(rank) => Ok(serde_json::to_string(&rank).unwrap()),
        _ => Err(Status::NotFound),
    }
}

#[get("/projects/<from>/<to>")]
async fn projects(pool: &State<Pool<Postgres>>, from: &str, to: &str) -> Result<String, Status> {
    let rank = RankingItem::get_projects_ranking(from, to, &pool).await;
    match rank {
        Ok(rank) => Ok(serde_json::to_string(&rank).unwrap()),
        _ => Err(Status::NotFound),
    }
}

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    rocket::build()
        .mount("/api", routes![editors, langs, projects])
        .manage(pool)
        .configure(Config {
            port: 5005,
            ..Config::default()
        })
        .launch()
        .await?;

    Ok(())
}
