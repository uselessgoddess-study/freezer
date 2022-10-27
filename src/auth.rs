use actix_identity::Identity;
use actix_web::{
    dev::{Payload, ServiceRequest},
    error::{self, Error},
    get, web, FromRequest, HttpMessage, HttpRequest, HttpResponse, Responder,
};
use actix_web_grants::{
    permissions::AuthDetails,
    proc_macro::{has_any_role, has_permissions},
    GrantsMiddleware,
};
use chrono::{Duration, Utc};
use jwt::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use tap::Pipe;

pub const JWT_EXP: i64 = 24; // 24 hours
pub const SECRET: &[u8] = b"SUPER_SECRET\
_________________________________________________________________________________________________";
//
// #[derive(Debug, Serialize, Deserialize)]
// pub struct Claims {
//     pub username: String,
//     pub permissions: Vec<String>,
//     exp: i64,
// }
//
// impl Claims {
//     pub fn new(username: String, permissions: Vec<String>) -> Self {
//         Self {
//             username,
//             permissions,
//             exp: (Utc::now() + Duration::hours(JWT_EXP)).timestamp(),
//         }
//     }
// }
//
// pub fn code_jwt(claims: Claims) -> Result<String, Error> {
//     jwt::encode(
//         &Header::default(),
//         &claims,
//         &EncodingKey::from_secret(SECRET),
//     )
//     .map_err(error::ErrorUnauthorized)
// }
//
// pub fn decode_jwt(token: &str) -> Result<Claims, Error> {
//     jwt::decode::<Claims>(
//         token,
//         &DecodingKey::from_secret(SECRET),
//         &Validation::default(),
//     )
//     .map(|data| data.claims)
//     .map_err(error::ErrorUnauthorized)
// }

#[derive(Debug, Serialize)]
pub struct AuthData {
    pub login: String,
}

#[derive(PartialEq, Eq, Clone)]
pub enum Role {
    Admin,
    User,
}

pub async fn login(
    auth_data: web::Json<AuthData>,
    req: HttpRequest,
) -> Result<impl Responder, actix_web::Error> {
    let _ = Identity::login(&req.extensions(), json::to_string(&auth_data)?);
    Ok(HttpResponse::Ok())
}

struct SlimUser {
    login: String,
    role: Role,
}

pub type LoggedUser = SlimUser;

impl FromRequest for LoggedUser {
    type Error = Error;
    type Future = Ready<Result<LoggedUser, Error>>;

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        fn json<'a, T: Deserialize<'a>>(str: String) -> Result<T, json::Error> {
            json::from_str(&str)
        }

        let res: Result<_, crate::errors::Error> = try {
            Identity::from_request(req, pl)
                .into_inner()?
                .id()?
                .pipe(json)?
        };
        ready(res.map_err(Into::into))
    }
}

#[get("/denied")]
#[has_any_role("ADMIN")]
// #[has_permissions("ROLE_ADMIN", secure = "user_id.into_inner() == user.id")]
async fn denied_endpoint(auth: AuthDetails) -> HttpResponse {
    HttpResponse::Ok().body("OK")
}

async fn extract(req: &mut ServiceRequest) -> Result<Vec<Role>, Error> {
    //let user:  = req.extract();

    // Stub example
    Ok(vec![Role::Admin])
}
