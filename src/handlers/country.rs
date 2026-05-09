use actix_web::{web, HttpResponse};
use validator::Validate;

use crate::errors::AppError;
use crate::models::country::{CreateCountryRequest, UpdateCountryRequest};
use crate::services::country as svc;
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/countries",
    tag = "countries",
    operation_id = "list_countries",
    responses(
        (status = 200, description = "List of countries", body = [crate::models::country::Country])
    )
)]
pub async fn list(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let countries = svc::list(&state.pool).await?;
    Ok(HttpResponse::Ok().json(countries))
}

#[utoipa::path(
    get,
    path = "/api/v1/countries/{id}",
    tag = "countries",
    operation_id = "get_country",
    params(("id" = i64, Path, description = "Country id")),
    responses(
        (status = 200, description = "Country", body = crate::models::country::Country),
        (status = 404, description = "Country not found")
    )
)]
pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let country = svc::get(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(country))
}

#[utoipa::path(
    post,
    path = "/api/v1/countries",
    tag = "countries",
    operation_id = "create_country",
    request_body = CreateCountryRequest,
    responses(
        (status = 201, description = "Country created", body = crate::models::country::Country),
        (status = 409, description = "Duplicate iso_code")
    )
)]
pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<CreateCountryRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()?;
    let country = svc::create(&state.pool, &body).await?;
    let location = format!("/api/v1/countries/{}", country.id);
    Ok(HttpResponse::Created()
        .insert_header(("Location", location))
        .json(country))
}

#[utoipa::path(
    put,
    path = "/api/v1/countries/{id}",
    tag = "countries",
    operation_id = "update_country",
    params(("id" = i64, Path, description = "Country id")),
    request_body = UpdateCountryRequest,
    responses(
        (status = 200, description = "Country updated", body = crate::models::country::Country),
        (status = 404, description = "Country not found"),
        (status = 409, description = "Unique constraint conflict")
    )
)]
pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    body: web::Json<UpdateCountryRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()?;
    let country = svc::update(&state.pool, path.into_inner(), &body).await?;
    Ok(HttpResponse::Ok().json(country))
}

#[utoipa::path(
    delete,
    path = "/api/v1/countries/{id}",
    tag = "countries",
    operation_id = "delete_country",
    params(("id" = i64, Path, description = "Country id")),
    responses(
        (status = 204, description = "Country deleted"),
        (status = 404, description = "Country not found"),
        (status = 409, description = "Country is referenced by an address")
    )
)]
pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    svc::delete(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
