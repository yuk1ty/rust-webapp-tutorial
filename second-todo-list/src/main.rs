#[macro_use]
extern crate log;
use actix_web::web::{Data, Json};
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

#[derive(Deserialize)]
struct RegisterTodo {
    description: String,
}

struct SqliteTodo {
    id: String,
    description: String,
    done: u8,
    datetime: String,
}

impl From<SqliteTodo> for Todo {
    fn from(st: SqliteTodo) -> Self {
        Todo {
            id: Uuid::parse_str(st.id.as_str()).unwrap(),
            description: st.description,
            done: matches!(st.done, 1),
            datetime: Utc
                .from_local_datetime(
                    &NaiveDateTime::parse_from_str(st.datetime.as_str(), DATETIME_FORMAT).unwrap(),
                )
                .unwrap(),
        }
    }
}

#[derive(Serialize)]
struct Todo {
    id: Uuid,
    description: String,
    done: bool,
    datetime: DateTime<Utc>,
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
        .query_map([], |row| {
            Ok(SqliteTodo {
                id: row.get_unwrap(0),
                description: row.get_unwrap(1),
                done: row.get_unwrap(2),
                datetime: row.get_unwrap(3),
            })
        })
        .unwrap()
        .into_iter()
        .map(|r| Todo::from(r.unwrap()))
        .collect();

    HttpResponse::Ok().json(TodoList(results))
}

#[post("/todo")]
async fn register_todo(
    req: Json<RegisterTodo>,
    db: Data<Pool<SqliteConnectionManager>>,
) -> impl Responder {
    let id = Uuid::new_v4();

    let todo = SqliteTodo {
        id: id.to_string(),
        description: req.0.description,
        done: 0,
        datetime: Utc::now().format(DATETIME_FORMAT).to_string(),
    };

    let conn = db.get().unwrap();
    conn.execute(
        "insert into todo (id, description, done, datetime) values(?1, ?2, ?3, ?4)",
        params![todo.id, todo.description, todo.done, todo.datetime],
    )
    .unwrap();

    let mut stmt = conn
        .prepare("select id, description, done, datetime from todo where id = ?")
        .unwrap();

    let results: Vec<Todo> = stmt
        .query_map(params![id.to_string()], |row| {
            Ok(SqliteTodo {
                id: row.get_unwrap(0),
                description: row.get_unwrap(1),
                done: row.get_unwrap(2),
                datetime: row.get_unwrap(3),
            })
        })
        .unwrap()
        .into_iter()
        .map(|r| Todo::from(r.unwrap()))
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
