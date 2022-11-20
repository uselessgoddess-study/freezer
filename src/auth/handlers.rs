use crate::{
    auth::{AuthData, LoggedUser, Role},
    Result,
};
use actix_identity::Identity;
use actix_web::{delete, get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use tracing::warn;

#[delete("/api/auth")]
pub async fn logout(id: Identity) -> impl Responder {
    id.logout();
    HttpResponse::Ok()
}

#[post("/api/auth")]
pub async fn login(
    req: HttpRequest,
    auth: web::Json<AuthData>,
    // fixme: store: web::Data<Mutex<RoleStore>>,
) -> Result<impl Responder> {
    let auth = auth.into_inner();

    let role = match &auth.login[..] {
        "Admin" => Role::Admin,
        "Moder" => Role::Moder,
        _ => Role::User,
    };

    if let Err(err) = Identity::login(
        &req.extensions(),
        json::to_string(&LoggedUser {
            login: auth.login,
            role,
        })?,
    ) {
        warn!("problem with login: `{err}`");
    }
    Ok(HttpResponse::Ok())
}

#[get("/api/auth")]
pub async fn me(user: LoggedUser) -> impl Responder {
    web::Json(user)
}
