use chrono::{DateTime, NaiveDateTime, ParseError, Utc};
use station_struts::{FuelStationData, PriceLastUpdated, StationPriceLastUpdated, StationPrices};
use thiserror::Error;

pub mod station_struts;
/// Converts fuel station data to the target structure
///
/// # Arguments
///
/// * `raw_data` - The fuel station data to be processed
///
/// # Returns
///
/// A `Result` containing the converted `StationData` or a `ConversionError`
// pub fn process_data(raw_data: FuelStationData) -> Result<Station, ConversionError> {
pub fn process_data(json_data: &str) -> Vec<StationPriceLastUpdated> {
    let data: FuelStationData = serde_json::from_str(json_data).expect("Invalid JSON");
    // println!("=== data:\n{:#?}", data);
    let FuelStationData {
        last_updated,
        stations,
    } = data;

    // TODO RL Convert to match for error handling etc
    if !stations.is_empty() {
        let last_updated_parsed = parse_datetime(&last_updated).unwrap(); // Add ? back to end of line when return a Result
                                                                          // println!("last_updated_parsed: {:?}", last_updated_parsed);
        let stations_json = serde_json::to_string(&stations).unwrap();

        let processed_stations = process_stations(&stations_json);
        // println!("bob: {:#?}", bob);

        let stations_with_last_updated: Vec<StationPriceLastUpdated> = processed_stations
            .into_iter()
            .map(|station| {
                StationPriceLastUpdated {
                    site_id: station.site_id,
                    brand: station.brand,
                    address: station.address,
                    postcode: station.postcode,
                    location: station.location,
                    prices: vec![PriceLastUpdated {
                        prices: station.prices,
                        lu: last_updated_parsed.to_string(), // Append the last_updated date.
                    }],
                }
            })
            .collect();

        // println!(
        //     "stations_with_last_updated: {:#?}",
        //     stations_with_last_updated
        // );

        stations_with_last_updated
    } else {
        let nothing: Vec<StationPriceLastUpdated> = vec![];
        nothing
    }
    // let filtered_stations: Vec<Station> = stations
    //     .iter()
    //     .filter(|entry| !entry.brand.is_empty())
    //     .cloned()
    //     .collect();

    // println!("=== filtered_stations:\n{:?}", filtered_stations);
    // let stations: Vec<Station> = process_stations(filtered_stations);
}

/// Errors that can occur during data conversion
#[derive(Debug, Error)]
pub enum ConversionError {
    /// Represents a failure to parse the datetime
    #[error("Failed to parse datetime: {0}")]
    DateTimeParseError(#[from] ParseError),

    /// Represents a JSON parsing error
    #[error("JSON parsing error: {0}")]
    JsonParseError(#[from] serde_json::Error),
}

fn process_stations(json_data: &str) -> Vec<StationPrices> {
    let result: Vec<Result<StationPrices, serde_json::Error>> =
        serde_json::from_str::<Vec<serde_json::Value>>(json_data)
            .unwrap_or_default()
            .into_iter()
            .map(serde_json::from_value::<StationPrices>)
            .collect();

    // println!("process_stations");
    // println!("{:#?}", result);
    // println!("last_updated {}", last_updated);
    result.into_iter().filter_map(Result::ok).collect()
}

/// Parses a datetime string into ISO 8601 format
///
/// # Arguments
///
/// * `dt_str` - A datetime string in the format "dd/MM/yyyy HH:mm:ss"
///
/// # Returns
///
/// A `Result` containing the ISO 8601 formatted datetime string or a `ParseError`
pub fn parse_datetime(dt_str: &str) -> Result<String, ParseError> {
    // Parse the input date format "dd/MM/yyyy HH:mm:ss"
    let naive_dt = NaiveDateTime::parse_from_str(dt_str, "%d/%m/%Y %H:%M:%S")?;

    // Convert to UTC DateTime and then to ISO 8601 format
    let utc_dt: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive_dt, Utc);
    Ok(utc_dt.to_rfc3339())
}

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let path = env::current_dir()?;
//     println!("The current directory is {}", path.display());

//     let mut combined_vec: Vec<StationPriceLastUpdated> = vec![];
//     let data_source_path = Path::new(".").join("src").join("bin").join("data");
//     println!("data_source_path: {}", data_source_path.display());
//     let paths = fs::read_dir(data_source_path).unwrap();

//     for path in paths {
//         let file_content = fs::read_to_string(path.unwrap().path()).unwrap();
//         let processed_data = process_data(&file_content);
//         combined_vec.extend(processed_data);
//     }

//     let output_data = serde_json::to_string_pretty(&combined_vec).unwrap();

//     println!("combined_vec length: {}", combined_vec.len());
//     let output_filename = "stations.json";
//     let mut file = File::create(output_filename)?;
//     file.write_all(output_data.as_bytes())?;

//     Ok(())
// }
