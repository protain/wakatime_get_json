use std::{collections::HashMap, io::Write};

use anyhow::Result;
use chrono::{Local, TimeZone};
use comlib::*;
use config::Config;
use futures::executor::block_on;
use get_summary::url_encode::encode;
use once_cell::sync::Lazy;
use reqwest::{self, StatusCode};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use tokio_postgres::{Client, Error as PgError, NoTls};
use serde_json::json;

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
    log_dt: &chrono::NaiveDate,
    editors: &serde_json::Value,
    langs: &serde_json::Value,
    machines: &serde_json::Value,
    projects: &serde_json::Value,
    depends: &serde_json::Value,
    grand_total_sec: &Decimal,
) -> Result<u64, PgError> {
    let stmt = client
        .prepare(r#"
UPDATE wakatime_summary SET
    editors=$2,
    langs=$3,
    machine=$4,
    projects=$5,
    depends=$6,
    grand_total_sec=$7
where date=$1"#)
        .await
        .unwrap();
    client.execute(&stmt, &[
        &log_dt,
        &editors,
        &langs,
        &machines,
        &projects,
        &depends,
        &grand_total_sec
    ]).await
}

async fn register_wakatime(log_dt: &chrono::NaiveDate, summary: &SummariesAll) -> Result<()> {
    let db_url = SETTINGS.get_str("db_url")?;
    //println!("db_url: {}, date: {}", db_url, date);
    let (client, connection) = tokio_postgres::connect(&db_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            println!("connection error: {}", e);
        }
    });

    let json = serde_json::to_string(summary)?;
    let jval: serde_json::Value = serde_json::from_str(&json).unwrap();
    let null_obj = json!({});
    let null_arr = json!([{}]);
    let summary_data = jval.get("summaries")
        .unwrap_or(&null_obj)
        .get("data")
        .unwrap_or(&null_arr)
        .get(0)
        .unwrap_or(&null_obj);
    let depends = summary_data
        .get("dependencies")
        .unwrap_or(&null_arr);
    let editors = summary_data
        .get("editors")
        .unwrap_or(&null_arr);
    let langs = summary_data
        .get("languages")
        .unwrap_or(&null_arr);
    let machines = summary_data
        .get("machines")
        .unwrap_or(&null_arr);
    let projects = summary_data
        .get("projects")
        .unwrap_or(&null_arr);
    
    let grand_total_sec = match summary_data.get("grand_total") {
        Some(j) => {
            match j.get("total_seconds") {
                Some(v) => v.as_f64().unwrap_or(0.0),
                None => 0.0,
            }
        }
        None => 0.0,
    };

    let stmt = client
        .prepare(r#"
INSERT INTO wakatime_summary
    (date, editors, langs, machine, projects, depends, grand_total_sec)
VALUES
    ($1, $2, $3, $4, $5, $6, $7)"#)
        .await?;
    let grand_total_sec = Decimal::from_f64(grand_total_sec).unwrap();
    let _ = client
        .execute(&stmt, &[
            &log_dt,
            &editors,
            &langs,
            &machines,
            &projects,
            &depends,
            &grand_total_sec
        ])
        .await
        .unwrap_or_else(|_| block_on(update_log(
            &client,
            &log_dt,
            &editors,
            &langs,
            &machines,
            &projects,
            &depends,
            &grand_total_sec)).unwrap());

    Ok(())
}

async fn get_onedate_summary(
    dt_start: &chrono::DateTime<Local>,
    dt_end: &chrono::DateTime<Local>,
    save_file: bool,
) -> anyhow::Result<()> {
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
        let dt_start = dt_start.naive_local().date();
        match register_wakatime(&dt_start, &summary_all).await {
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut dt_end = chrono::Local::now();
    dt_end = dt_end - chrono::Duration::days(1);
    let mut dt_start = dt_end;
    let mut save_file = true;

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() >= 1 {
        if let Ok(tmp_dt_0) =
            chrono::NaiveDateTime::parse_from_str(
                &format!("{} 00:00", args[0]),
                "%Y/%m/%d %H:%M")
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

    let mut loop_cnt = 1;
    loop {
        let one_dt_end = dt_start + chrono::Duration::days(loop_cnt);
        get_onedate_summary(&one_dt_end, &one_dt_end, save_file).await?;

        println!("process => {}", one_dt_end);
        if one_dt_end > dt_end {
            break;
        }
        loop_cnt += 1;
    }

    Ok(())
}
