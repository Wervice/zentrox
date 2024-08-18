use actix_files as afs;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::{get, http::StatusCode, middleware, post, web, App, HttpResponse, HttpServer};

fn is_admin(session: &Session) -> bool {
    return session.get::<bool>("is_admin").expect("Failed to get is_admin").unwrap_or(false);
}

// Page Routes (Routes that lead to the display of a static page)
#[get("/")]
async fn index(session: Session) -> HttpResponse {
    // is_admin session value is != true (None or false), the user is served the login screen
    // otherwise, the user is redirected to /
    if is_admin(&session) {
        HttpResponse::Found().append_header(("Location", "/dashboard")).finish()
    } else {
        return HttpResponse::build(StatusCode::OK).body(std::fs::read_to_string("static/index.html").expect("Failed to read file"))
    }
}

#[get("/dashboard")]
async fn dashboard(session: Session) -> HttpResponse {
    // is_admin session value is != true (None or false), the user is redirected to /
    // otherwise, the user is served the dashboard.html file
    if is_admin(&session) {
        return HttpResponse::build(StatusCode::OK).body(std::fs::read_to_string("static/dashboard.html").expect("Failed to read file"))
    } else {
        HttpResponse::Found().append_header(("Location", "/")).finish()
    }
}

// Blocks (Used to prevent users from accessing certain static resources)
#[get("/dashboard.html")]
async fn dashboard_asset_block(session: Session) -> HttpResponse {
    if !is_admin(&session) {
        HttpResponse::build(StatusCode::FORBIDDEN).finish()
    } else {
        HttpResponse::build(StatusCode::OK).body(std::fs::read_to_string("static/dashboard.html").expect("Failed to read file"))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Serving Zentrox on Port 8080");
    let secret_session_key = Key::generate();
    HttpServer::new(move || {
        App::new()
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_session_key.clone(),
            ))
            .wrap(middleware::Compress::default())
            .service(dashboard)
            .service(index)
            .service(dashboard_asset_block)
            .service(afs::Files::new("/", "static/"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
