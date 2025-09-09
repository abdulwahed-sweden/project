use actix_web::{dev::Payload, Error, FromRequest, HttpRequest, HttpResponse, Result};
use actix_session::Session;
use futures_util::future::{Ready, ready};
use serde_json;
use crate::models::SessionUser;

pub struct AuthUser(pub SessionUser);

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let session = Session::extract(req).into_inner();
        
        if let Ok(session) = session {
            if let Ok(Some(user_json)) = session.get::<String>("user") {
                if let Ok(user) = serde_json::from_str::<SessionUser>(&user_json) {
                    return ready(Ok(AuthUser(user)));
                }
            }
        }
        
        ready(Err(actix_web::error::ErrorUnauthorized("Not authenticated")))
    }
}

pub struct OptionalAuthUser(pub Option<SessionUser>);

impl FromRequest for OptionalAuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let session = Session::extract(req).into_inner();
        
        if let Ok(session) = session {
            if let Ok(Some(user_json)) = session.get::<String>("user") {
                if let Ok(user) = serde_json::from_str::<SessionUser>(&user_json) {
                    return ready(Ok(OptionalAuthUser(Some(user))));
                }
            }
        }
        
        ready(Ok(OptionalAuthUser(None)))
    }
}

pub async fn logout(session: Session) -> Result<HttpResponse> {
    session.clear();
    Ok(HttpResponse::Found()
        .insert_header(("location", "/login?msg=logged_out"))
        .finish())
}

pub fn login_user(session: &Session, user: SessionUser) -> Result<(), actix_web::Error> {
    let user_json = serde_json::to_string(&user)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Session serialization error: {}", e)))?;
    
    session.insert("user", user_json)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Session storage error: {}", e)))?;
    
    Ok(())
}