use actix_web::{web, App, HttpResponse, HttpRequest, HttpServer, Responder};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use futures_util::stream::StreamExt;

async fn manual_hello(mut body: web::Payload,
                      web::Path(filename): web::Path<String>,
                      req: HttpRequest) -> impl Responder {
    println!("********************************************************************************");
    println!("                              NEXT REQUEST");
    println!("********************************************************************************");
    req.headers().iter().for_each(|v| {
        println!("{}: {:?}", v.0, v.1)
    });
    let mut file = OpenOptions::new().write(true).append(true).create(true).open(filename).await.unwrap();
    while let Some(item) = body.next().await {
        let bytes = item.unwrap();
        file.write_all(&bytes[..]).await.unwrap();
    }
    file.flush().await.unwrap();
    drop(file);
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(
            web::resource("/{file}")
                .route(web::post().to(manual_hello))
            )
    })
    .bind("127.0.0.1:8080")?
        .run()
        .await
}
