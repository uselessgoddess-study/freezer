mod handlers;
mod role;

pub use handlers::{login, logout, me};
pub use role::Role;

use crate::errors::{Error, Result};
use actix_identity::Identity;
use actix_web::{
    dev::{Payload, ServiceRequest}, FromRequest, HttpRequest,
};




use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::future::{ready, Ready};
use tap::Pipe;


pub const JWT_EXP: i64 = 24; // 24 hours
pub const SECRET: &[u8] = r#"
Bei Nacht im Dorf der Wächter rief:
    Elfe!
Ein ganz kleines Elfchen im Walde schlief,
    Wohl um die Elfe;
Und meint, es rief ihm aus dem Thal
Bei seinem Namen die Nachtigall,
Oder Silpelit[8] hätt' ihn gerufen.
Reibt sich der Elf' die Augen aus,
Begibt sich vor sein Schneckenhaus
Und ist als wie ein trunken Mann,
Sein Schläflein war nicht voll gethan,
Und humpelt also tippe tapp
Durch's Haselholz in's Thal hinab,
Schlupft an der Weinbergmauer hin,
Daran viel Feuerwürmchen glühn:[9]
"Was sind das helle Fensterlein?
Da drin wird eine Hochzeit seyn;
Die Kleinen sitzen beim Mahle,
Und treiben's in dem Saale.
Da guck' ich wohl ein wenig 'nein!"
— Pfui, stößt den Kopf an harten Stein!
Elfe, gelt, du hast genug?
    Gukuk! Gukuk!
"#
.as_bytes();

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthData {
    pub login: String,
}

#[derive(Serialize, Deserialize)]
pub struct SlimUser {
    login: String,
    role: Role,
}

pub type LoggedUser = SlimUser;

impl FromRequest for LoggedUser {
    type Error = Error;
    type Future = Ready<Result<LoggedUser>>;

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        fn json<T: DeserializeOwned>(str: String) -> Result<T, json::Error> {
            json::from_str(&str)
        }

        let res: Result<_, Box<dyn std::error::Error>> = try {
            Identity::from_request(req, pl)
                .into_inner()?
                .id()?
                .pipe(json)?
        };
        res.map_err(Error::Unauth).pipe(ready)
    }
}

pub async fn extractor(req: &mut ServiceRequest) -> Result<Vec<Role>, actix_web::Error> {
    if let Ok(user) = req.extract::<LoggedUser>().await {
        vec![user.role]
    } else {
        vec![]
    }
    .pipe(Ok)
}
