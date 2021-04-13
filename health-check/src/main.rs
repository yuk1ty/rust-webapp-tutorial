use actix_web::{get, App, HttpResponse, HttpServer, Responder};

#[get("/health")]
async fn hc() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hc))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
