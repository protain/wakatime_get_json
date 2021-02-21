use anyhow;
use oauth2::{
    AuthorizationCode,
    AuthUrl,
    ClientId,
    ClientSecret,
    CsrfToken,
    RedirectUrl,
    Scope,
    TokenResponse,
    TokenUrl
};
use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
use url::Url;
use std::{ io::{BufRead, BufReader, Write}, net::TcpListener };
use once_cell::sync::Lazy;
use config::{Config};

static SETTINGS: Lazy<Config> = Lazy::new(|| {
    let mut settings = Config::default();
    settings.merge(config::File::with_name("Settings.toml")).unwrap();
    settings
});

fn main() -> anyhow::Result<()> {
    let app_id = SETTINGS.get_str("api-id")?;
    let secret = SETTINGS.get_str("api-secret")?;
    let authorize_url = "https://wakatime.com/oauth/authorize";
    let token_url = "https://wakatime.com/oauth/token";

    let client  = BasicClient::new(
        ClientId::new(app_id.into()),
        Some(ClientSecret::new(secret.into())), 
        AuthUrl::new(authorize_url.into())?, 
        Some(TokenUrl::new(token_url.into())?)
    ).set_redirect_url(
        RedirectUrl::new("http://localhost:8081".into()).expect("Invalid redirect URL"),
    );

    //let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        // Set the desired scopes.
        .add_scope(Scope::new("email,read_logged_time,read_stats,read_orgs".to_string()))
        // Set the PKCE code challenge
        //.set_pkce_challenge(pkce_challenge)
        .url();

    println!("Browse to: {}", auth_url); 
    
    // A very naive implementation of the redirect server.
    let listener = TcpListener::bind("127.0.0.1:8081").unwrap();
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let code;
            let state;
            {
                let mut reader = BufReader::new(&stream);

                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();

                let redirect_url = request_line.split_whitespace().nth(1).unwrap();
                let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

                let code_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "code"
                    })
                    .unwrap();

                let (_, value) = code_pair;
                code = AuthorizationCode::new(value.into_owned());

                let state_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "state"
                    })
                    .unwrap();

                let (_, value) = state_pair;
                state = CsrfToken::new(value.into_owned());
            }

            let message = "Go back to your terminal :)";
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                message.len(),
                message
            );
            stream.write_all(response.as_bytes()).unwrap();

            println!("Github returned the following code:\n{}\n", code.secret());
            println!(
                "Github returned the following state:\n{} (expected `{}`)\n",
                state.secret(),
                csrf_state.secret()
            );

            // Exchange the code with a token.
            let token_res = client.exchange_code(code).request(http_client);

            println!("Github returned the following token:\n{:?}\n", token_res);

            if let Ok(token) = token_res {
                // NB: Github returns a single comma-separated "scope" parameter instead of multiple
                // space-separated scopes. Github-specific clients can parse this scope into
                // multiple scopes by splitting at the commas. Note that it's not safe for the
                // library to do this by default because RFC 6749 allows scopes to contain commas.
                let scopes = if let Some(scopes_vec) = token.scopes() {
                    scopes_vec
                        .iter()
                        .map(|comma_separated| comma_separated.split(','))
                        .flatten()
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                };
                println!("Github returned the following scopes:\n{:?}\n", scopes);
            }

            // The server will terminate itself after collecting the first code.
            break;
        }
    }



    Ok(())
}
