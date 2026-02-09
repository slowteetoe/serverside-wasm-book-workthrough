use std::io::Read;
use wasmcloud_component::http::{self, Method, Response};
use wasmcloud_component::wasi::keyvalue::*;

wit_bindgen::generate!({ generate_all });

use thomastaylor312::ollama::generate::{generate, Request};

struct Component;

http::export!(Component);

impl http::Server for Component {
    fn handle(
        request: http::IncomingRequest,
    ) -> http::Result<http::Response<impl http::OutgoingBody>> {
        let (parts, mut body) = request.into_parts();

        match (parts.method, parts.uri.path()) {
            (Method::POST, "/api/create") => {
                let bucket = store::open("default").unwrap();
                let mut buf = Vec::new();
                body.read_to_end(&mut buf).unwrap();
                let body = String::from_utf8(buf).unwrap();

                let mut lines = body.lines();
                let story_name = lines
                    .next()
                    .unwrap_or("UnnamedStory")
                    .trim()
                    .replace(" ", "")
                    .to_string();
                let story_content = lines.collect::<Vec<_>>().join("\n");

                // Ok(Response::new(format!("{story_name}\n{story_content}")))

                match bucket.set(&story_name, story_content.as_bytes()) {
                    Ok(_) => Ok(Response::new(format!("Stored {story_name}\n"))),
                    Err(err) => Ok(Response::new(format!(
                        "Failed to store {story_name}: {err}\n"
                    ))),
                }
            }
            (Method::POST, "/api/retrieve") => {
                let bucket = store::open("default").unwrap();
                let mut buf = Vec::new();
                body.read_to_end(&mut buf).unwrap();
                let story_name = String::from_utf8(buf).unwrap().trim().to_string();
                match bucket.get(&story_name).unwrap() {
                    Some(content) => {
                        let story_content = String::from_utf8(content).unwrap();
                        Ok(http::Response::new(format!("{story_content}\n")))
                    }
                    None => Ok(http::Response::new("Story not found\n".to_string())),
                }
            }
            (_, "/api/generate") => {
                let prompt = "Once upon a time".to_string();
                let generated_story = generate(&Request {
                    prompt: prompt.clone(),
                    images: None,
                })
                .unwrap();
                let story = format!("{}{}", prompt, generated_story.response);
                let bucket = store::open("default").unwrap();
                let count = atomics::increment(&bucket, "counter", 1).unwrap();
                let story_name = format!("generated{}", count);
                bucket.set(&story_name, story.clone().as_bytes()).unwrap();
                Ok(http::Response::new(format!("{}\n{}\n", story_name, story)))
            }
            (_, _) => Ok(http::Response::new("Invalid route.\n".to_string())),
        }
    }
}
