#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use anyhow::{Ok, Result};
use chrono::NaiveTime;
use colored::Colorize;
use dotenvy_macro::dotenv;
use prost::Message;
use reqwest::{
    header::{HeaderMap, ACCEPT, AUTHORIZATION},
    ClientBuilder, Method, Request,
};
use std::{
    fs::{create_dir, remove_dir_all, write},
    path::Path,
};
use transit_realtime::FeedEntity;

use crate::transit_realtime::{vehicle_position::CongestionLevel, FeedMessage};

mod transit_realtime {
    #![allow(clippy::all, clippy::pedantic, clippy::nursery, non_snake_case)]
    include!(concat!(env!("OUT_DIR"), "/transit_realtime.rs"));
}

#[tokio::main]
async fn main() -> Result<()> {
    const ENDPOINT: &str = "https://api.transport.nsw.gov.au/v1/gtfs/vehiclepos/buses";
    clear_output_directory()?;
    let bytes = ClientBuilder::new()
        .default_headers({
            let mut header_map = HeaderMap::new();
            header_map.append(ACCEPT, "application/x-google-protobuf".parse()?);
            header_map.append(
                AUTHORIZATION,
                format!("apikey {}", dotenv!("API_KEY")).parse()?,
            );
            header_map
        })
        .build()?
        .execute(Request::new(Method::GET, ENDPOINT.parse()?))
        .await?
        .bytes()
        .await?;
    write("out/data", bytes.clone())?;
    let message: FeedMessage = FeedMessage::decode(bytes)?;
    write("out/feed_message", format!("{:#?}", message))?;
    let entities: Vec<_> = message
        .entity
        .iter()
        .filter(|entity| {
            entity
                .vehicle
                .as_ref()
                .unwrap()
                .trip
                .as_ref()
                .unwrap()
                .route_id
                .as_ref()
                .unwrap()
                .ends_with("601")
        })
        .collect();
    write("out/routes_with_601", format!("{:#?}", entities))?;
    print_entities(&entities)?;
    Ok(())
}

fn clear_output_directory() -> Result<()> {
    if Path::new("out").exists() {
        remove_dir_all("out")?;
    }
    create_dir("out")?;
    Ok(())
}

fn print_entities(entities: &Vec<&FeedEntity>) -> Result<()> {
    for entity in entities {
        println!(
            "{} {}
    {} {}
    {} {} {} {}
    {}
    {} {}
",
            "Found route".dimmed(),
            entity
                .vehicle
                .as_ref()
                .unwrap()
                .trip
                .as_ref()
                .unwrap()
                .route_id()
                .bold(),
            "ID".dimmed(),
            entity.id,
            "start".dimmed(),
            {
                let date = entity
                    .vehicle
                    .as_ref()
                    .unwrap()
                    .trip
                    .as_ref()
                    .unwrap()
                    .start_date();
                format!("{}/{}/{}", &date[6..], &date[4..6], &date[..4])
            },
            "at".dimmed(),
            {
                let time = entity
                    .vehicle
                    .as_ref()
                    .unwrap()
                    .trip
                    .as_ref()
                    .unwrap()
                    .start_time();
                let time = NaiveTime::from_hms_opt(
                    time[..2].parse()?,
                    time[3..5].parse()?,
                    time[6..].parse()?,
                )
                .unwrap();
                time.format("%-I:%M %p")
            },
            {
                let position = entity.vehicle.as_ref().unwrap().position.as_ref().unwrap();
                format!(
                    "{} ({}, {})
    {} {}˚ {}{}{}
    {} {} {}",
                    "position".dimmed(),
                    position.latitude,
                    position.longitude,
                    "heading".dimmed(),
                    position.bearing.unwrap(),
                    "(".dimmed(),
                    if position.bearing.unwrap() < 22.5 {
                        "N".bold()
                    } else if position.bearing.unwrap() < 67.5 {
                        "NE".bold()
                    } else if position.bearing.unwrap() < 112.5 {
                        "E".bold()
                    } else if position.bearing.unwrap() < 157.5 {
                        "SE".bold()
                    } else if position.bearing.unwrap() < 202.5 {
                        "S".bold()
                    } else if position.bearing.unwrap() < 247.5 {
                        "SW".bold()
                    } else if position.bearing.unwrap() < 292.5 {
                        "W".bold()
                    } else if position.bearing.unwrap() < 337.5 {
                        "NW".bold()
                    } else {
                        "N".bold()
                    },
                    ")".dimmed(),
                    "speed".dimmed(),
                    position.speed.unwrap(),
                    "km/h".dimmed()
                )
            },
            "congestion level".dimmed(),
            CongestionLevel::from_i32(entity.vehicle.as_ref().unwrap().congestion_level.unwrap())
                .unwrap()
                .as_str_name()
                .split('_')
                .map(str::to_lowercase)
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
    Ok(())
}
