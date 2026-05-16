use utoipa::OpenApi;

use crate::handlers;
use crate::models::address::{Address, CreateAddressRequest, UpdateAddressRequest};
use crate::models::booking::{Booking, BookingHouse, BookingPerson, BookingStatus, CreateBookingRequest, CreateBookingResponse, RecordPaymentRequest};
use crate::models::calendar::{CalendarEntry, CalendarStatus, CreateCalendarRequest, UpdateCalendarPriceRequest};
use crate::models::country::{Country, CreateCountryRequest, UpdateCountryRequest};
use crate::models::house::{CreateHouseRequest, House, UpdateHouseRequest};
use crate::models::manager::{CreateManagerRequest, Manager, UpdateManagerRequest};
use crate::models::person::{CreatePersonRequest, Person, UpdatePersonRequest};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "House Rental API",
        description = "Backend for managing rental houses (villas).",
        version = "0.1.0"
    ),
    paths(
        handlers::health::health,
        handlers::country::list,
        handlers::country::get,
        handlers::country::create,
        handlers::country::update,
        handlers::country::delete,
        handlers::manager::list,
        handlers::manager::get,
        handlers::manager::create,
        handlers::manager::update,
        handlers::manager::delete,
        handlers::person::list,
        handlers::person::get,
        handlers::person::create,
        handlers::person::update,
        handlers::person::delete,
        handlers::address::get,
        handlers::address::create,
        handlers::address::update,
        handlers::address::delete,
        handlers::house::list,
        handlers::house::get,
        handlers::house::create,
        handlers::house::update,
        handlers::house::delete,
        handlers::calendar::list,
        handlers::calendar::get,
        handlers::calendar::create,
        handlers::calendar::update_price,
        handlers::calendar::delete,
        handlers::booking::list,
        handlers::booking::get,
        handlers::booking::create,
        handlers::booking::cancel,
        handlers::booking::record_payment,
    ),
    components(schemas(
        Country,
        CreateCountryRequest,
        UpdateCountryRequest,
        Manager,
        CreateManagerRequest,
        UpdateManagerRequest,
        Person,
        CreatePersonRequest,
        UpdatePersonRequest,
        Address,
        CreateAddressRequest,
        UpdateAddressRequest,
        House,
        CreateHouseRequest,
        UpdateHouseRequest,
        CalendarStatus,
        CalendarEntry,
        CreateCalendarRequest,
        UpdateCalendarPriceRequest,
        BookingStatus,
        BookingHouse,
        BookingPerson,
        Booking,
        CreateBookingResponse,
        CreateBookingRequest,
        RecordPaymentRequest,
    )),
    tags(
        (name = "health", description = "Health check"),
        (name = "countries", description = "Countries"),
        (name = "managers", description = "House managers"),
        (name = "persons", description = "Renting persons"),
        (name = "addresses", description = "House addresses"),
        (name = "houses", description = "Rentable houses"),
        (name = "calendar", description = "Per-house calendar entries"),
        (name = "bookings", description = "House bookings"),
    )
)]
pub struct ApiDoc;
