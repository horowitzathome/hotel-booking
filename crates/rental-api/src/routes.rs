use actix_web::web;

use crate::handlers::{address, booking, calendar, country, house, manager, person};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/addresses", web::post().to(address::create))
            .route("/addresses/{id}", web::get().to(address::get))
            .route("/addresses/{id}", web::put().to(address::update))
            .route("/addresses/{id}", web::delete().to(address::delete))
            .route("/houses", web::get().to(house::list))
            .route("/houses", web::post().to(house::create))
            .route("/houses/{id}", web::get().to(house::get))
            .route("/houses/{id}", web::put().to(house::update))
            .route("/houses/{id}", web::delete().to(house::delete))
            .route("/houses/{house_id}/calendar", web::get().to(calendar::list))
            .route("/houses/{house_id}/calendar", web::post().to(calendar::create))
            .route("/houses/{house_id}/calendar", web::patch().to(calendar::update_price))
            .route("/houses/{house_id}/calendar", web::delete().to(calendar::delete))
            .route("/houses/{house_id}/calendar/{id}", web::get().to(calendar::get))
            .route("/countries", web::get().to(country::list))
            .route("/countries", web::post().to(country::create))
            .route("/countries/{id}", web::get().to(country::get))
            .route("/countries/{id}", web::put().to(country::update))
            .route("/countries/{id}", web::delete().to(country::delete))
            .route("/managers", web::get().to(manager::list))
            .route("/managers", web::post().to(manager::create))
            .route("/managers/{id}", web::get().to(manager::get))
            .route("/managers/{id}", web::put().to(manager::update))
            .route("/managers/{id}", web::delete().to(manager::delete))
            .route("/persons", web::get().to(person::list))
            .route("/persons", web::post().to(person::create))
            .route("/persons/{id}", web::get().to(person::get))
            .route("/persons/{id}", web::put().to(person::update))
            .route("/persons/{id}", web::delete().to(person::delete))
            .route("/bookings", web::get().to(booking::list))
            .route("/bookings", web::post().to(booking::create))
            .route("/bookings/{id}", web::get().to(booking::get))
            .route("/bookings/{id}/cancel", web::post().to(booking::cancel))
            .route("/bookings/{id}/payment", web::post().to(booking::record_payment)),
    );
}
