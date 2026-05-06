use actix_web::web;

use crate::handlers::country;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/countries", web::get().to(country::list))
            .route("/countries", web::post().to(country::create))
            .route("/countries/{id}", web::get().to(country::get))
            .route("/countries/{id}", web::put().to(country::update))
            .route("/countries/{id}", web::delete().to(country::delete)),
    );
}
