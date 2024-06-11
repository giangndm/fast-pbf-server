use std::sync::Arc;

use poem::{
    handler,
    web::{Data, Json, Query},
};

use serde::Deserialize;

use crate::{geo::GeoIndex, AddressResponse, Response};

#[derive(Debug, Deserialize)]
struct QueryParams {
    lat: f32,
    lon: f32,
}

#[handler]
pub fn query(
    data: Data<&Arc<GeoIndex>>,
    Query(query): Query<QueryParams>,
) -> Json<Response<AddressResponse>> {
    if let Some(ways) = data.0.find(query.lat, query.lon) {
        Json(Response {
            success: true,
            data: Some(AddressResponse { ways }),
            error: None,
        })
    } else {
        Json(Response {
            success: false,
            data: None,
            error: Some("No address found".to_string()),
        })
    }
}
