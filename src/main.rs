use dioxus::prelude::*;
use env_logger;
use log::{error, info};
use surrealdb::Surreal;
use surrealdb::engine::local::{Db, Mem};
use surrealdb::Error as SurrealError;
use thiserror::Error;
use futures::executor::block_on;
use once_cell::sync::OnceCell;

static CSS: Asset = asset!("/assets/main.css");

// Replace `static mut` with `OnceCell`
static DB: OnceCell<Surreal<Db>> = OnceCell::new();

// Create a new wrapper type.
#[derive(Clone)]
struct TitleState(String);

#[derive(serde::Deserialize)]
struct DogApi {
    message: String,
}

// Define a custom error type
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to initialize the database: {0}")]
    DatabaseInitError(String),
    #[error("Database operation failed: {0}")]
    DatabaseOpError(String),
    #[error("HTTP request failed: {0}")]
    HttpRequestError(String),
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

impl From<SurrealError> for AppError {
    fn from(err: SurrealError) -> Self {
        AppError::DatabaseOpError(err.to_string())
    }
}

fn main() {
    env_logger::init();
    info!("Starting HotDog application");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| TitleState("HotDog".to_string()));
    rsx! {
        document::Stylesheet { href: CSS }
        Title {}
        DogView {}
    }
}

#[component]
fn Title() -> Element {
    let title = use_context::<TitleState>();
    rsx! {
        h1 { "{title.0}" }
    }
}

#[component]
fn DogView() -> Element {
    let mut img_src = use_resource(|| async move {
        match reqwest::get("https://dog.ceo/api/breeds/image/random").await {
            Ok(resp) => match resp.json::<DogApi>().await {
                Ok(data) => data.message,
                Err(err) => {
                    error!("Failed to parse dog image response: {}", err);
                    String::new()
                }
            },
            Err(err) => {
                error!("Failed to fetch dog image: {}", err);
                String::new()
            }
        }
    });

    rsx! {
        div { id: "dogview",
            img { src: img_src.cloned().unwrap_or_default() }
        }
        div { id: "buttons",
             button {
                onclick: move |_| async move {
                    let current = img_src.cloned().unwrap();
                    img_src.restart();

                    if let Err(e) = save_dog(current).await {
                        error!("Failed to save dog: {:?}", e);
                    }
                },
                "save!"
            }
        }
    }
}

#[server]
async fn save_dog(image: String) -> Result<(), ServerFnError<String>> {
    let db = get_db();
    info!("Attempting to save dog URL: {}", image);

    db.create::<Option<surrealdb::Value>>("dogs")
        .content(serde_json::json!({
            "url": image,
        }))
        .await
        .map_err(|err| {
            error!("Failed to save dog URL to database: {}", err);
            ServerFnError::ServerError(format!("Database error: {}", err))
        })?;

    info!("Successfully saved dog URL: {}", image);

    Ok(())
}


pub fn get_db() -> &'static Surreal<Db> {
    DB.get_or_init(|| {
        let db = Surreal::<Db>::init(); // Explicitly specify `Surreal<Db>`
        log::info!("SurrealDB initialized");

        block_on(async {
            db.connect::<Mem>(()) // Use `Mem` explicitly for the in-memory database
                .await
                .expect("Failed to connect to in-memory database");
        });

        log::info!("Connected to in-memory database");

        block_on(async {
            db.query("DEFINE TABLE dogs SCHEMAFULL;")
                .await
                .expect("Failed to define schema");
        });

        log::info!("Database schema initialized");
        db
    })
}
