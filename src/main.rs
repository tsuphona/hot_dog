use dioxus::prelude::*;
use env_logger;
use surrealdb::Surreal;
use surrealdb::engine::local::Db;

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

// Expose a `save_dog` endpoint on our server that takes an "image" parameter
#[server]
async fn save_dog(image: String) -> Result<(), ServerFnError<String>> {
    use std::io::Write;

    // Open the `dogs.txt` file in append-only mode, creating it if it doesn't exist;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("dogs.txt")
        .unwrap();

    // And then write a newline to it with the image url
    file.write_fmt(format_args!("{image}\n"))
        .map_err(|err| ServerFnError::ServerError(err.to_string()))?;

    Ok(())
}
