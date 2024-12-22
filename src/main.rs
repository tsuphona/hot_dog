use dioxus::prelude::*;
use env_logger;
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use std::sync::Once;

static CSS: Asset = asset!("/assets/main.css");

// Create a new wrapper type.
#[derive(Clone)]
struct TitleState(String);

#[derive(serde::Deserialize)]
struct DogApi {
    message: String,
}

fn main() {
    env_logger::init();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Provide that type as a Context
    use_context_provider(|| TitleState("HotDog".to_string()));
    rsx! {
        document::Stylesheet { href: CSS }
        Title {}
        DogView {}
    }
}

#[component]
fn Title() -> Element {
    // Consume that type as a Context
    let title = use_context::<TitleState>();
    rsx! {
        h1 { "{title.0}" }
    }
}

#[component]
fn DogView() -> Element {
    let mut img_src = use_resource(|| async move {
        reqwest::get("https://dog.ceo/api/breeds/image/random")
            .await
            .unwrap()
            .json::<DogApi>()
            .await
            .unwrap()
            .message
    });

    rsx! {
        div { id: "dogview",
            img { src: img_src.cloned().unwrap_or_default() }
        }
        div { id: "buttons",
             button {
                onclick: move |_| async move {
                    // Clone the current image
                    let current = img_src.cloned().unwrap();

                    // Start fetching a new image
                    img_src.restart();

                    // And call the `save_dog` server function
                    if let Err(e) = save_dog(current).await {
                        eprintln!("Failed to save dog: {:?}", e);
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

    // Insert the dog image URL into the "dogs" table
    db.create::<Option<serde_json::Value>>("dogs") // Return type as `Option<serde_json::Value>`
        .content(serde_json::json!({
            "url": image, // Insert the URL field
        }))
        .await
        .map_err(|err| ServerFnError::ServerError(err.to_string()))?;

    Ok(())
}

pub fn get_db() -> &'static Surreal<Db> {
    static mut DB: Option<Surreal<Db>> = None;
    static INIT: Once = Once::new();

    unsafe {
        INIT.call_once(|| {
            let db = Surreal::init(); // Initialize the database
            println!("SurrealDB initialized");

            DB = Some(db);
            println!("Database ready for use");
        });

        DB.as_ref().expect("Database not initialized")
    }
}
