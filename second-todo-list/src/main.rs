use actix_web::web::{Data, Json};
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use derive_new::new;
use log::info;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Row};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

#[derive(Deserialize)]
struct RegisterTodo {
    description: String,
}

#[derive(Serialize)]
struct TaskId(Uuid);

#[derive(Serialize, new)]
struct Todo {
    id: TaskId,
    description: String,
    done: bool,
    datetime: DateTime<Utc>,
}

impl<'stmt> From<&Row<'stmt>> for Todo {
    fn from(row: &Row) -> Self {
        let uuid: String = row.get_unwrap(0);
        let datetime: String = row.get_unwrap(3);

        Todo::new(
            TaskId(Uuid::parse_str(uuid.as_str()).unwrap()),
            row.get_unwrap(1),
            matches!(row.get_unwrap(2), 1),
            Utc.from_local_datetime(
                &NaiveDateTime::parse_from_str(datetime.as_str(), DATETIME_FORMAT).unwrap(),
            )
            .unwrap(),
        )
    }
}

#[derive(Serialize)]
struct TodoList(Vec<Todo>);

#[get("/health")]
async fn hc() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[get("/todo")]
async fn todo_list(db: Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = db.get().unwrap();

    let mut stmt = conn
        .prepare("select id, description, done, datetime from todo")
        .unwrap();

    let results: Vec<Todo> = stmt
        .query_map([], |row| Ok(Todo::from(row)))
        .unwrap()
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    HttpResponse::Ok().json(TodoList(results))
}

#[post("/todo")]
async fn register_todo(
    req: Json<RegisterTodo>,
    db: Data<Pool<SqliteConnectionManager>>,
) -> impl Responder {
    let id = Uuid::new_v4();

    let todo = Todo::new(TaskId(id), req.0.description, false, Utc::now());

    let conn = db.get().unwrap();
    conn.execute(
        "insert into todo (id, description, done, datetime) values(?1, ?2, ?3, ?4)",
        params![todo.id.0, todo.description, todo.done, todo.datetime],
    )
    .unwrap();

    let mut stmt = conn
        .prepare("select id, description, done, datetime from todo where id = ?")
        .unwrap();

    let results: Vec<Todo> = stmt
        .query_map(params![id.to_string()], |row| Ok(Todo::from(row)))
        .unwrap()
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    HttpResponse::Ok().json(TodoList(results))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "INFO");
    env_logger::init();

    let manager = SqliteConnectionManager::file("test.db");
    let pool = Pool::new(manager).unwrap();

    info!("Bootstrapping the server...");

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(hc)
            .service(todo_list)
            .service(register_todo)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
