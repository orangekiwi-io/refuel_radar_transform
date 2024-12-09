use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// Represents the raw input data structure for fuel station information
#[derive(Debug, Serialize, Deserialize)]
pub struct FuelStationData {
    pub(crate) last_updated: String,
    pub(crate) stations: Vec<serde_json::Value>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Location {
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub(crate) latitude: f64,
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub(crate) longitude: f64,
}

// Custom deserializer to handle latitude and longitude
fn deserialize_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let value: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;

    match value {
        serde_json::Value::String(s) => s
            .parse::<f64>()
            .map_err(|e| serde::de::Error::custom(format!("Invalid coordinate: {}", e))),

        serde_json::Value::Number(num) => num
            .as_f64()
            .ok_or_else(|| serde::de::Error::custom("Invalid number")),

        _ => Err(serde::de::Error::custom("Invalid type for coordinate")),
    }
}

type PricesHashMap = HashMap<String, f64>;

/// Represents a price object with fuel price data and when that data was last updated
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriceLastUpdated {
    #[serde(flatten)]
    pub prices: PricesHashMap,
    pub last_updated: String,
}

/// Represents a station entry with prices
#[derive(Serialize, Clone)]
pub struct StationPrices {
    pub(crate) site_id: String,
    pub(crate) brand: String,
    pub(crate) address: String,
    pub(crate) postcode: String,
    pub(crate) location: Location,
    pub(crate) prices: PricesHashMap,
}

/// Represents a station entry with updated prices date
#[derive(Debug, Serialize, Clone)]
pub struct StationPriceLastUpdated {
    pub site_id: String,
    pub brand: String,
    pub address: String,
    pub postcode: String,
    pub location: Location,
    pub prices: Vec<PriceLastUpdated>,
}

// Custom deserializer for prices
fn deserialize_prices<'de, D>(deserializer: D) -> Result<PricesHashMap, D::Error>
where
    D: Deserializer<'de>,
{
    let map: HashMap<String, Value> = HashMap::deserialize(deserializer)?;
    Ok(map
        .into_iter()
        .filter_map(|(key, value)| {
            match value {
                Value::Number(num) => num.as_f64(),
                Value::String(s) => s.parse::<f64>().ok(),
                _ => None,
            }
            .filter(|&v| v > 0.0)
            .map(|v| (key, v))
        })
        .collect())
}

impl std::fmt::Debug for StationPrices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StationPrices")
            .field("site_id", &self.site_id)
            .field("brand", &self.brand)
            .field("address", &self.address)
            .field("postcode", &self.postcode)
            .field("location", &self.location)
            .field("prices", &self.prices)
            .finish()
    }
}

impl<'de> Deserialize<'de> for StationPrices {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        struct TempStationPrices {
            site_id: String,
            brand: Option<String>,
            address: String,
            postcode: String,
            location: Location,
            #[serde(deserialize_with = "deserialize_prices")]
            prices: PricesHashMap,
        }

        let temp = TempStationPrices::deserialize(deserializer)?;

        if temp.brand.is_none() {
            Err(serde::de::Error::custom("brand is null"))
        } else {
            Ok(StationPrices {
                site_id: temp.site_id,
                brand: temp.brand.unwrap(),
                address: temp.address,
                postcode: temp.postcode,
                location: temp.location,
                prices: temp.prices,
            })
        }
    }
}
