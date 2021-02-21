use std::io::Write;

use chrono::{Local, TimeZone};
use reqwest::{self, StatusCode};
use once_cell::sync::Lazy;
use config::{Config};

static SETTINGS: Lazy<Config> = Lazy::new(|| {
    let mut settings = Config::default();
    settings.merge(config::File::with_name("Settings.toml")).unwrap();
    settings
});

async fn request_wakatime(start: &str, end: &str) -> anyhow::Result<String> {
    // get the key from Settings.toml
    let sct_api_key = SETTINGS.get_str("secret-api-key")?;
    let client = reqwest::Client::new();

    let req_url = format!("https://wakatime.com/api/v1/users/current/summaries?start={}&end={}", start, end);
    let res = client.get(&req_url)
        .header("Authorization", format!("Basic {}", base64::encode(sct_api_key)))
        .send()
        .await?;
    if res.status() != StatusCode::OK {
        return Err(anyhow::anyhow!(format!("bad request => status code: {}", res.status())));
    }
    let body = res.text().await?;

    Ok(body)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut dt_end = chrono::Local::now();
    let mut dt_start = dt_end;
    let mut save_file = false;

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() >= 1 {
        if let Ok(tmp_dt_0) = chrono::NaiveDateTime::parse_from_str(&format!("{} 00:00", args[0]), "%Y/%m/%d %H:%M") {
            dt_start = Local.from_local_datetime(&tmp_dt_0).unwrap();
        }
        if args.len() >= 2 {
            if let Ok(tmp_dt_1) = chrono::NaiveDateTime::parse_from_str(&format!("{} 00:00", &args[1]), "%Y/%m/%d %H:%M") {
                dt_end = Local.from_local_datetime(&tmp_dt_1).unwrap();
            }
        }
    }
    for a in args {
        save_file = a == "save_file";
    }
    let body = request_wakatime(
        &dt_start.format("%Y-%m-%d").to_string(),
        &dt_end.format("%Y-%m-%d").to_string()).await;
    if let Ok(body_text) = body {
        if save_file {
            let str_dt_start = dt_start.format("%Y%m%d").to_string();
            let str_dt_end = dt_end.format("%Y%m%d").to_string();
            let file_name: String;
            if str_dt_start == str_dt_end {
                file_name = format!("res_{}", str_dt_start);
            }
            else {
                file_name = format!("res_{}-{}", str_dt_start, str_dt_end);
            }
            let mut f = std::fs::File::create(file_name)?;
            if let Err(e) = f.write(body_text.as_bytes()) {
                println!("error! : {:?}", e);
            }
        }
        else {
            println!("{}", body_text);
        }
    }
    else {
        println!("error! : {:?}", body);
    }

    Ok(())
}
