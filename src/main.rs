use std::sync::Arc;

use geo::GeoIndex;
use poem::{get, listener::TcpListener, middleware::Tracing, EndpointExt, Route, Server};

use clap::Parser;

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

mod api;
mod geo;

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
        .at("/reverse", get(api::reverse::handler))
        .at("/get", get(api::get::handler))
        .data(Arc::new(geo))
        .with(Tracing);

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("Fast-pbf-server")
        .run(app)
        .await
}
