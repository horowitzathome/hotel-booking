use actix_web::web;

use crate::handlers::{country, manager};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/countries", web::get().to(country::list))
            .route("/countries", web::post().to(country::create))
            .route("/countries/{id}", web::get().to(country::get))
            .route("/countries/{id}", web::put().to(country::update))
            .route("/countries/{id}", web::delete().to(country::delete))
            .route("/managers", web::get().to(manager::list))
            .route("/managers", web::post().to(manager::create))
            .route("/managers/{id}", web::get().to(manager::get))
            .route("/managers/{id}", web::put().to(manager::update))
            .route("/managers/{id}", web::delete().to(manager::delete)),
    );
}
