use std::sync::Arc;

use poem::{
    handler,
    web::{Data, Json, Query},
};

use serde::Deserialize;

use crate::{geo::GeoIndex, AddressResponse, Response};

#[derive(Debug, Deserialize)]
struct GetParams {
    id: i64,
}

#[handler]
pub fn get_by_id(
    data: Data<&Arc<GeoIndex>>,
    Query(query): Query<GetParams>,
) -> Json<Response<AddressResponse>> {
    if let Some(way) = data.0.get_by_id(query.id) {
        Json(Response {
            success: true,
            data: Some(AddressResponse { ways: vec![way] }),
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
