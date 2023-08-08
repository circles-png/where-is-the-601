use anyhow::{Ok, Result};
use dotenvy_macro::dotenv;
use reqwest::{
    header::{HeaderMap, ACCEPT, AUTHORIZATION},
    ClientBuilder, Method, Request,
};
use std::fs::write;

mod transit_realtime {
    #![allow(non_snake_case)]
    include!(concat!(env!("OUT_DIR"), "/transit_realtime.rs"));
}

#[tokio::main]
async fn main() -> Result<()> {
    const ENDPOINT: &str = "https://api.transport.nsw.gov.au/v1/gtfs/vehiclepos/buses";
    let client = ClientBuilder::new()
        .default_headers({
            let mut header_map = HeaderMap::new();
            header_map.append(ACCEPT, "application/x-google-protobuf".parse()?);
            header_map.append(
                AUTHORIZATION,
                format!("apikey {}", dotenv!("API_KEY")).parse()?,
            );
            header_map
        })
        .build()?;
    let request = Request::new(Method::GET, ENDPOINT.parse()?);
    let response = client.execute(request).await?;
    write("out/data", response.bytes().await?)?;
    Ok(())
}
