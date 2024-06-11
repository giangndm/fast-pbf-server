use std::sync::Arc;

use geo::{GeoIndex, WayInfo};
use poem::{get, listener::TcpListener, middleware::Tracing, EndpointExt, Route, Server};

use clap::Parser;

mod api_get_id;
mod api_query;

/// Pbf query server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Cached geo-index for faster load time
    #[arg(short, long, env)]
    cache: Option<String>,

    /// Path to pbf file
    #[arg(short, long, env)]
    pbf: String,
}

mod geo;

#[derive(serde::Serialize)]
struct Response<T> {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(serde::Serialize)]
struct AddressResponse {
    ways: Vec<WayInfo>,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();

    let geo = match args.cache {
        Some(path) => {
            //check if file path exists
            match std::fs::File::open(&path) {
                Ok(file) => {
                    let start = std::time::Instant::now();
                    println!("load index from file");
                    let geo: GeoIndex = bincode::deserialize_from(file).unwrap();
                    println!("Loaded index in {}ms", start.elapsed().as_millis());
                    geo
                }
                Err(_e) => {
                    println!("cannot load index => rebuild");
                    let mut geo = GeoIndex::new();
                    geo.build(&args.pbf);
                    // save geo to file
                    std::fs::write(&path, bincode::serialize(&geo).unwrap())
                        .expect("Unable to write file");
                    geo
                }
            }
        }
        None => {
            let mut geo = GeoIndex::new();
            geo.build(&args.pbf);
            geo
        }
    };

    let app = Route::new()
        .at("/query", get(api_query::query))
        .at("/get", get(api_get_id::get_by_id))
        .data(Arc::new(geo))
        .with(Tracing);
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("Fast-pbf-server")
        .run(app)
        .await
}
