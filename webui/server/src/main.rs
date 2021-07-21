#[macro_use]
extern crate rocket;
use rocket::{
    fs::NamedFile,
    http::{ContentType, Status},
    response::content::Custom,
    Config, Response, State,
};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use std::{borrow::Cow, collections::HashMap, env, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_json;

use once_cell::sync::Lazy;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../client/dist"]
struct Asset;

static ASSET_FILES: Lazy<HashMap<Cow<str>, Option<()>>> = Lazy::new(|| {
    let mut files = HashMap::new();
    Asset::iter().for_each(|v| {
        println!("file: {}", &v);
        files.insert(v.to_owned(), None);
    });
    files
});

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
select title, (sum(total_seconds) / 3600) hours from
    (select
        jsonb_path_query(w.data, '$.summaries.data.editors.name')#>>'{}' title,
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
select title, (sum(total_seconds) / 3600) hours from
    (select
        jsonb_path_query(w.data, '$.summaries.data.languages.name')#>>'{}' title,
        jsonb_path_query(w.data, '$.summaries.data.languages.total_seconds')::double precision total_seconds
    from wakatime_dat w
    where w.date >= $1 and w.date <= $2
    ) x
where title <> 'Other'
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
select title, (sum(total_seconds) / 3600) hours from
    (select
        jsonb_path_query(w.data, '$.summaries.data.projects.name')#>>'{}' title,
        jsonb_path_query(w.data, '$.summaries.data.projects.total_seconds')::double precision total_seconds
    from wakatime_dat w
    where w.date >= $1 and w.date <= $2
    ) x
where title <> 'Unknown Project'
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

#[get("/<filename..>")]
async fn statics(filename: PathBuf) -> Result<Custom<Vec<u8>>, Status> {
    let fpath = filename.to_str().unwrap();
    if !ASSET_FILES.contains_key(fpath) {
        return Err(Status::NotFound);
    }
    let dat: Vec<u8> = Asset::get(fpath).unwrap().into();
    let content_type = match filename
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .split(".")
        .last()
        .unwrap()
    {
        "html" => ContentType::HTML,
        "js" => ContentType::JavaScript,
        "css" => ContentType::CSS,
        "jpg" => ContentType::JPEG,
        "png" => ContentType::PNG,
        "ico" => ContentType::Icon,
        _ => ContentType::Plain,
    };
    let d = Custom(content_type, dat);
    Ok(d)
    //Ok(dat)
}

#[catch(404)]
fn index() -> Custom<Vec<u8>> {
    let dat: Vec<u8> = Asset::get("index.html").unwrap().into();
    Custom(ContentType::HTML, dat)
}

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    let prefix = match env::var("PREFIX") {
        Ok(pfx) => pfx,
        Err(_) => "".into(),
    };

    println!("prefix={}", prefix);
    rocket::build()
        .mount(
            format!("/{}api", &prefix),
            routes![editors, langs, projects],
        )
        .mount(format!("/{}", &prefix), routes![statics])
        .register(format!("/{}", &prefix), catchers![index])
        .manage(pool)
        .configure(Config {
            port: 5005,
            ..Config::default()
        })
        .launch()
        .await?;

    Ok(())
}
