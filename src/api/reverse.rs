use std::sync::Arc;

use poem::{
    handler,
    web::{Data, Json, Query},
};

use serde::Deserialize;
use serde_json::Value;

use crate::geo::GeoIndex;

use super::{ErrorInfo, Response};

#[derive(Debug, Deserialize)]
struct QueryParams {
    lat: f32,
    lon: f32,
}

#[handler]
pub fn handler(data: Data<&Arc<GeoIndex>>, Query(query): Query<QueryParams>) -> Json<Value> {
    if let Some(ways) = data.0.find(query.lat, query.lon) {
        Json(serde_json::to_value(ways).expect("Should convert to json"))
    } else {
        Json(
            serde_json::to_value(Response::<()> {
                data: None,
                error: Some(ErrorInfo {
                    code: 400,
                    message: "Address not found",
                }),
            })
            .expect("Should convert to json"),
        )
    }
}
