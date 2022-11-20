use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, test, test::TestRequest, App};
use freezer::{
    auth,
    auth::{LoggedUser, Role},
};
use json::json;
use tap::pipe::Pipe;

#[actix_web::test]
async fn test_index_get() {
    let app = test::init_service(
        App::new()
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .build(),
            )
            .service(auth::me)
            .service(auth::login)
            .service(auth::logout),
    )
    .await;

    // you need to login before `get` yourself
    assert!(
        test::call_service(&app, TestRequest::get().uri("/api/auth").to_request())
            .await
            .status()
            .is_client_error()
    );

    // nice
    let login = test::call_service(
        &app,
        TestRequest::post()
            .uri("/api/auth")
            .set_json(json!({
                // admin is reserved login
                "login": "Admin"
            }))
            .to_request(),
    )
    .await;
    assert!(login.status().is_success());

    // fixme: manual delegate all cookie
    //  it is base for testing???
    let req = TestRequest::get().pipe(|mut req| {
        for cookie in login.response().cookies() {
            req = req.cookie(cookie);
        }
        req
    });

    let user: LoggedUser =
        test::call_and_read_body_json(&app, req.uri("/api/auth").to_request()).await;
    assert_eq!(
        user,
        // is a reserved result for `Admin` login
        LoggedUser {
            login: "Admin".to_string(),
            role: Role::Admin
        }
    );
}
