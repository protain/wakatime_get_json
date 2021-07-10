use anyhow::Result;
use config::Config;
use futures::executor::block_on;
use once_cell::sync::Lazy;
use tokio_postgres::{Client, Error as PgError, NoTls};

static SETTINGS: Lazy<Config> = Lazy::new(|| {
    let mut settings = Config::default();
    settings
        .merge(config::File::with_name("Settings.toml"))
        .unwrap();
    settings
});

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

        let json = std::fs::read_to_string(fentry.path().to_str().unwrap_or(""))?;
        let stmt = client
            .prepare("INSERT INTO wakatime_dat (date, data) VALUES ($1, $2)")
            .await?;

        let jval: serde_json::Value = serde_json::from_str(&json).unwrap();
        let _ = client
            .execute(&stmt, &[&log_dt, &jval])
            .await
            .unwrap_or_else(|_| block_on(update_log(&client, log_dt, &jval)).unwrap());
    }
    let rows = client
        .query("SELECT $1::TEXT as TXT", &[&"hello world"])
        .await?;
    let value: &str = rows[0].get("TXT");
    println!("{}", value);

    Ok(())
}
