use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use futures_util::stream::StreamExt;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

async fn file_upload(
    mut body: web::Payload,
    web::Path(filename): web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    println!("********************************************************************************");
    println!("                         NEXT FILE UPLOAD REQUEST");
    println!("********************************************************************************");
    req.headers()
        .iter()
        .for_each(|v| println!("{}: {:?}", v.0, v.1));
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(filename)
        .await
        .unwrap();
    while let Some(item) = body.next().await {
        let bytes = item.unwrap();
        file.write_all(&bytes[..]).await.unwrap();
    }
    file.flush().await.unwrap();
    drop(file);
    HttpResponse::Ok()
}

async fn produce_error_response() -> impl Responder {
    HttpResponse::BadRequest()
        .content_type("text/plain")
        .body("canned error response for testing purposes")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(web::resource("/upload/{file}").route(web::post().to(file_upload)))
            .route("/error", web::post().to(produce_error_response))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
