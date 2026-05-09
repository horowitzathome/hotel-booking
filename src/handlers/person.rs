use actix_web::{web, HttpResponse};
use validator::Validate;

use crate::errors::AppError;
use crate::models::person::{CreatePersonRequest, UpdatePersonRequest};
use crate::services::person as svc;
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/persons",
    tag = "persons",
    operation_id = "list_persons",
    responses((status = 200, description = "List of persons", body = [crate::models::person::Person]))
)]
pub async fn list(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let persons = svc::list(&state.pool).await?;
    Ok(HttpResponse::Ok().json(persons))
}

#[utoipa::path(
    get,
    path = "/api/v1/persons/{id}",
    tag = "persons",
    operation_id = "get_person",
    params(("id" = i64, Path, description = "Person id")),
    responses(
        (status = 200, description = "Person", body = crate::models::person::Person),
        (status = 404, description = "Person not found")
    )
)]
pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let person = svc::get(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(person))
}

#[utoipa::path(
    post,
    path = "/api/v1/persons",
    tag = "persons",
    operation_id = "create_person",
    request_body = CreatePersonRequest,
    responses(
        (status = 201, description = "Person created", body = crate::models::person::Person),
        (status = 409, description = "Duplicate email")
    )
)]
pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<CreatePersonRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()?;
    let person = svc::create(&state.pool, &body).await?;
    let location = format!("/api/v1/persons/{}", person.id);
    Ok(HttpResponse::Created()
        .insert_header(("Location", location))
        .json(person))
}

#[utoipa::path(
    put,
    path = "/api/v1/persons/{id}",
    tag = "persons",
    operation_id = "update_person",
    params(("id" = i64, Path, description = "Person id")),
    request_body = UpdatePersonRequest,
    responses(
        (status = 200, description = "Person updated", body = crate::models::person::Person),
        (status = 404, description = "Person not found"),
        (status = 409, description = "Unique constraint conflict")
    )
)]
pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    body: web::Json<UpdatePersonRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()?;
    let person = svc::update(&state.pool, path.into_inner(), &body).await?;
    Ok(HttpResponse::Ok().json(person))
}

#[utoipa::path(
    delete,
    path = "/api/v1/persons/{id}",
    tag = "persons",
    operation_id = "delete_person",
    params(("id" = i64, Path, description = "Person id")),
    responses(
        (status = 204, description = "Person deleted"),
        (status = 404, description = "Person not found"),
        (status = 409, description = "Person is referenced by a booking")
    )
)]
pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    svc::delete(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
