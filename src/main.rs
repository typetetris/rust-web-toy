use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use futures_util::stream::StreamExt;
use std::ops::Add;
use std::sync::Arc;
use std::sync::RwLock;
use structopt::StructOpt;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

#[derive(Debug, StructOpt)]
#[structopt()]
struct Opt {
    /// Socket addresses to listen on for requests
    /// e.g. 127.0.0.1:8080
    #[structopt(short, long, required(true))]
    socket_addrs: Vec<std::net::SocketAddr>,
}

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

async fn read_the_counter(data: web::Data<Arc<RwLock<u32>>>) -> impl Responder {
    let mut result = String::new();
    {
        let handle = data.read().unwrap();
        result.push_str(&format!("{}", *handle));
    }
    HttpResponse::Ok().content_type("text/plain").body(result)
}

async fn noop() -> impl Responder {
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();
    let guarded_counter: Arc<RwLock<u32>> = Arc::new(std::sync::RwLock::new(0));
    {
        let guarded_counter_for_background_thread = guarded_counter.clone();
        tokio::spawn(async move {
            loop {
                {
                    let mut handle = guarded_counter_for_background_thread.write().unwrap();
                    *handle += 1;
                }
                tokio::time::delay_until(
                    tokio::time::Instant::now().add(tokio::time::Duration::from_millis(250)),
                )
                .await;
            }
        });
    }
    let mut server = HttpServer::new(move || {
        App::new()
            .data(guarded_counter.clone())
            //            .service(web::resource("/upload/{file}").route(web::post().to(file_upload)))
            //            .route("/error", web::post().to(produce_error_response))
            .route("/counter", web::get().to(read_the_counter))
            .route("/noop", web::get().to(noop))
    });
    for addr in opt.socket_addrs.iter() {
        server = server.bind(addr).unwrap();
    }
    server
        .client_timeout(std::u64::MAX)
        .client_shutdown(std::u64::MAX)
        .max_connections(std::usize::MAX)
        .max_connection_rate(std::usize::MAX)
        .run()
        .await
}
