use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Todo {
    id: Uuid,
    description: String,
    done: bool,
    datetime: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct TodoList(Vec<Todo>);

#[get("/health")]
async fn hc() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[get("/todo")]
async fn todo_list() -> impl Responder {
    let list = TodoList(vec![
        Todo {
            id: Uuid::new_v4(),
            description: "タスク1".to_string(),
            done: false,
            datetime: Utc::now(),
        },
        Todo {
            id: Uuid::new_v4(),
            description: "タスク2".to_string(),
            done: false,
            datetime: Utc::now(),
        },
    ]);
    HttpResponse::Ok().json(list)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hc).service(todo_list))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}