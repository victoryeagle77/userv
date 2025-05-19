use actix::ActorContext;
use actix::{Actor, StreamHandler};
use actix_files::NamedFile;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Result};
use actix_web_actors::ws;

// Handler pour servir la page HTML
async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("./static/index.html")?)
}

// Handler pour la connexion WebSocket
async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(MyWebSocket {}, &req, stream)
}

// Struct WebSocket
struct MyWebSocket;

// Implémentation de l'acteur
impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;
}

// Implémentation du handler de messages WebSocket
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => ctx.text(format!("Echo: {}", text)),
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Pong(_)) => {}
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

#[actix_web::main]
pub async fn web() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/ws/", web::get().to(ws_index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
