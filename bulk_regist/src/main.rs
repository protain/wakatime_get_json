use anyhow::Result;
use config::Config;
use futures::executor::block_on;
use once_cell::sync::Lazy;
use rust_decimal::{Decimal, prelude::FromPrimitive};
use serde::__private::de;
use tokio_postgres::{Client, Error as PgError, NoTls};
use serde_json::{json};
use rust_decimal_macros::dec;

static SETTINGS: Lazy<Config> = Lazy::new(|| {
    let mut settings = Config::default();
    settings
        .merge(config::File::with_name("Settings.toml"))
        .unwrap();
    settings
});

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
WHERE date=$1"#)
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_url = SETTINGS.get_str("db_url")?;
    let (client, connection) = tokio_postgres::connect(&db_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            println!("connection error: {}", e);
        }
    });

    let cdir = std::env::current_dir()?;
    let read_dir = std::fs::read_dir(cdir.as_os_str())?;
    for fnm in read_dir.into_iter() {
        if fnm.is_err() {
            continue;
        }
        let fentry = fnm.unwrap();
        let fnm = fentry.file_name();
        let fnm = fnm.to_string_lossy();
        let fnm_bara: Vec<&str> = fnm.split('.').collect();
        let ext = fnm_bara.last().unwrap();
        if *ext != "json" {
            continue;
        }
        let log_dt = fnm_bara
            .first()
            .unwrap_or(&"")
            .split('_')
            .last()
            .unwrap_or(&"");
        println!("{:?}", &fnm);

        let log_dt = chrono::NaiveDate::parse_from_str(&log_dt, "%Y%m%d")?;

        let json = std::fs::read_to_string(fentry.path().to_str().unwrap_or(""))?;
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
            .prepare("INSERT INTO wakatime_summary (date, editors, langs, machine, projects, depends, grand_total_sec) VALUES ($1, $2, $3, $4, $5, $6, $7)")
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
    }
    let rows = client
        .query("SELECT $1::TEXT as TXT", &[&"hello world"])
        .await?;
    let value: &str = rows[0].get("TXT");
    println!("{}", value);

    Ok(())
}
