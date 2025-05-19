use actix::ActorContext;
use actix::{Actor, StreamHandler};
use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer, Result, web};
use actix_web_actors::ws;

const ADDR: &str = "localhost";
const PORT: u16 = 8080;

// Handler pour la connexion WebSocket
async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(MyWebSocket {}, &req, stream)
}

// Structure WebSocket
struct MyWebSocket;

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => ctx.text(text),
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

/// Providing WebSocket route and all root WEB folder.
#[actix_web::main]
pub async fn web() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(actix_files::Files::new("/", "./static/").index_file("index.html"))
            .route("/ws/", web::get().to(ws_index))
    })
    .bind((ADDR, PORT))?
    .run()
    .await
}
