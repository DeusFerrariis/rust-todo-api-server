#![feature(associated_type_defaults)]

mod handler;
mod model;
mod store;

use rocket::{fairing::{Fairing, Info, Kind}, http::{Header, Status}, serde::json::Json, Request, Response, State};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;


#[derive(Serialize, Deserialize)]
pub struct TaskData {
    id: usize,
    content: String,
}

#[derive(Serialize, Deserialize)]
pub struct NewTaskRequest {
    content: String,
}

#[derive(Serialize, Deserialize)]
pub struct NewTaskResponse {
    id: usize,
}

#[derive(Serialize, Deserialize)]
pub struct TaskResponse {
    id: usize,
    content: String,
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS, PUT"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[rocket::options("/<_..>")]
fn all_options() {
    /* Intentionally left empty */
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .attach(CORS)
        .manage(store::local::MutexVec::<(usize, String)>::new())
        .mount("/", rocket::routes![
            all_options,
            handler::task::create_task,
            handler::task::get_task,
            handler::task::get_tasks,
            handler::task::complete_task,
        ])
}