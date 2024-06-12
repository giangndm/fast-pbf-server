use std::sync::Arc;

use poem::{
    handler,
    web::{Data, Json, Query},
};

use serde::Deserialize;
use serde_json::Value;

use super::{ErrorInfo, Response};
use crate::geo::GeoIndex;

#[derive(Debug, Deserialize)]
struct GetParams {
    id: i64,
}

#[handler]
pub fn handler(data: Data<&Arc<GeoIndex>>, Query(query): Query<GetParams>) -> Json<Value> {
    if let Some(way) = data.0.get_by_id(query.id) {
        Json(serde_json::to_value(way).expect("Should convert to json"))
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
