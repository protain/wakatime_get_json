use std::{collections::HashMap, io::Write};

use anyhow::Result;
use chrono::{Local, TimeZone};
use comlib::*;
use config::Config;
use futures::executor::block_on;
use get_summary::url_encode::encode;
use once_cell::sync::Lazy;
use reqwest::{self, StatusCode};
use tokio_postgres::{Client, Error as PgError, NoTls};

static SETTINGS: Lazy<Config> = Lazy::new(|| {
    let mut settings = Config::default();
    settings
        .merge(config::File::with_name("Settings.toml"))
        .unwrap();
    settings
});

async fn request_wakatime(
    start: &str,
    end: &str,
    proj_name: Option<&str>,
) -> anyhow::Result<Summaries> {
    // get the key from Settings.toml
    let sct_api_key = SETTINGS.get_str("secret-api-key")?;
    let client = reqwest::Client::new();

    let mut req_url = format!(
        "https://wakatime.com/api/v1/users/current/summaries?start={}&end={}",
        encode(start),
        encode(end)
    );
    if let Some(pjnm) = proj_name {
        req_url = format!("{}&project={}", req_url, encode(pjnm));
    }
    let res = client
        .get(&req_url)
        .header(
            "Authorization",
            format!("Basic {}", base64::encode(sct_api_key)),
        )
        .send()
        .await?;
    if res.status() != StatusCode::OK {
        return Err(anyhow::anyhow!(format!(
            "bad request => status code: {}",
            res.status()
        )));
    }
    let body = res.text().await?;
    let summaries: Summaries = serde_json::from_str(&body)?;

    Ok(summaries)
}

async fn update_log(
    client: &Client,
    log_dt: &str,
    jval: &serde_json::Value,
) -> Result<u64, PgError> {
    let stmt = client
        .prepare("UPDATE wakatime_dat SET data=$2 where date=$1")
        .await
        .unwrap();
    client.execute(&stmt, &[&log_dt, &jval]).await
}

async fn register_wakatime(date: &str, summary: &SummariesAll) -> Result<()> {
    let db_url = SETTINGS.get_str("db_url")?;
    //println!("db_url: {}, date: {}", db_url, date);
    let (client, connection) = tokio_postgres::connect(&db_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            println!("connection error: {}", e);
        }
    });

    let stmt = client
        .prepare("INSERT INTO wakatime_dat (date, data) VALUES ($1, $2)")
        .await?;

    let json = serde_json::to_string(summary)?;
    let jval: serde_json::Value = serde_json::from_str(&json).unwrap();
    let _ = client
        .execute(&stmt, &[&date, &jval])
        .await
        .unwrap_or_else(|_| block_on(update_log(&client, &date, &jval)).unwrap());

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut dt_end = chrono::Local::now();
    dt_end = dt_end - chrono::Duration::days(1);
    let mut dt_start = dt_end;
    let mut save_file = false;

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() >= 1 {
        if let Ok(tmp_dt_0) =
            chrono::NaiveDateTime::parse_from_str(&format!("{} 00:00", args[0]), "%Y/%m/%d %H:%M")
        {
            dt_start = Local.from_local_datetime(&tmp_dt_0).unwrap();
        }
        if args.len() >= 2 {
            if let Ok(tmp_dt_1) = chrono::NaiveDateTime::parse_from_str(
                &format!("{} 00:00", &args[1]),
                "%Y/%m/%d %H:%M",
            ) {
                dt_end = Local.from_local_datetime(&tmp_dt_1).unwrap();
            }
        }
    }
    for a in args {
        save_file = a == "save_file";
    }
    let summary = request_wakatime(
        &dt_start.format("%Y-%m-%d").to_string(),
        &dt_end.format("%Y-%m-%d").to_string(),
        None,
    )
    .await?;
    let mut proj_summary: HashMap<String, Summaries> = HashMap::new();
    for dat in &summary.data[0].projects {
        let proj = request_wakatime(
            &dt_start.format("%Y-%m-%d").to_string(),
            &dt_end.format("%Y-%m-%d").to_string(),
            Some(&dat.name),
        )
        .await?;
        proj_summary.insert(dat.name.clone(), proj);
    }

    let summary_all = SummariesAll {
        summaries: summary,
        projects: proj_summary,
    };

    if dt_start.format("%Y%m%d").to_string() == dt_end.format("%Y%m%d").to_string() {
        println!("db regist start.");
        match register_wakatime(&dt_start.format("%Y%m%d").to_string(), &summary_all).await {
            Ok(_) => {}
            Err(err) => {
                println!("db regist error! : {:?}", err);
            }
        }
    }

    if save_file {
        let str_dt_start = dt_start.format("%Y%m%d").to_string();
        let str_dt_end = dt_end.format("%Y%m%d").to_string();
        let file_name: String;
        if str_dt_start == str_dt_end {
            file_name = format!("res_{}.json", str_dt_start);
        } else {
            file_name = format!("res_{}-{}.json", str_dt_start, str_dt_end);
        }
        let mut f = std::fs::File::create(file_name)?;
        let body_txt = serde_json::to_string_pretty(&summary_all)?;
        if let Err(e) = f.write(body_txt.as_bytes()) {
            println!("error! : {:?}", e);
        }
    } else {
        println!("error! : {:?}", summary_all);
    }

    Ok(())
}
