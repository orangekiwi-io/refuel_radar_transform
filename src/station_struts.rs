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
    // Last update (lu) date and time (ISO)
    // Shortened to lu to reduce file size
    pub lu: String,
}

/// Represents a fuel station's price information with last updated timestamp.
///
/// # Structure
///
/// Captures comprehensive data about a single fuel station, including:
/// - Unique site identification
/// - Brand information
/// - Location details
/// - Prices with their last updated timestamp
///
/// # Derive Attributes
///
/// - `Debug`: Enables convenient debugging and printing
/// - `Serialize`: Allows conversion to various formats (JSON, etc.)
/// - `Clone`: Enables deep copying of the entire station data
///
/// # Use Case
///
/// Designed to store enriched station pricing data with timestamp information,
/// useful for tracking historical pricing and data updates
#[derive(Debug, Serialize, Clone)]
pub struct StationPriceLastUpdated {
    pub site_id: String,
    pub brand: String,
    pub address: String,
    pub postcode: String,
    pub location: Location,
    pub prices: Vec<PriceLastUpdated>,
}

/// Custom price deserialization function with robust parsing and filtering.
///
/// # Deserialization Strategy
///
/// Transforms input data by:
/// - Converting various input types to floating-point prices
/// - Filtering out non-positive or invalid price values
/// - Handling different serialization formats flexibly
///
/// # Supported Input Types
///
/// Handles price inputs as:
/// - Numeric values
/// - String representations of numbers
///
/// # Filtering Criteria
///
/// - Converts input to f64
/// - Removes entries with:
///   * Non-numeric values
///   * Zero or negative prices
///
/// # Performance
///
/// - Uses iterator-based transformation
/// - Minimal memory allocation
/// - Efficient filtering and conversion
///
/// # Examples
///
/// ```rust
/// // Hypothetical JSON input
/// // {"unleaded": 1.50, "diesel": "1.75", "invalid": "not a number"}
/// // Result: {"unleaded": 1.50, "diesel": 1.75}
/// ```
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

/// Represents a fuel station's detailed information and pricing.
///
/// # Structure
///
/// Captures comprehensive data about a single fuel station, including:
/// - Unique site identification
/// - Brand information
/// - Location details
/// - Pricing data
///
/// # Derive Attributes
///
/// - `Serialize`: Allows the struct to be converted to various formats (JSON, etc.)
/// - `Clone`: Enables deep copying of the entire station data
///
/// # Visibility
///
/// All fields are `pub(crate)`, meaning they're accessible within the current crate,
/// providing a balance between encapsulation and internal flexibility
#[derive(Serialize, Clone)]
pub struct StationPrices {
    pub(crate) site_id: String,
    pub(crate) brand: String,
    pub(crate) address: String,
    pub(crate) postcode: String,
    pub(crate) location: Location,
    pub(crate) prices: PricesHashMap,
}

/// Custom Debug implementation for more controlled logging and debugging.
///
/// # Benefits
///
/// - Provides a clean, structured debug output
/// - Allows selective field representation
/// - Ensures sensitive data can be selectively displayed
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

/// Custom Deserialization implementation with advanced validation and transformation.
///
/// # Deserialization Strategy
///
/// 1. Use a temporary struct for initial deserialization
/// 2. Perform custom validation and transformation
/// 3. Handle optional fields and apply business logic during deserialization
///
/// # Key Features
///
/// - Validates brand is not null
/// - Applies brand name formatting during deserialization
/// - Provides robust error handling
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
            let brand_name = format_brand(temp.brand.unwrap());
            Ok(StationPrices {
                site_id: temp.site_id,
                brand: brand_name,
                address: temp.address,
                postcode: temp.postcode,
                location: temp.location,
                prices: temp.prices,
            })
        }
    }
}

/// Standardizes and formats brand names to a consistent representation.
///
/// This function performs brand name normalization by:
/// - Trimming whitespace
/// - Converting to lowercase for matching
/// - Applying predefined formatting rules
/// - Preserving original casing for known brands
///
/// # Parameters
///
/// - `brand`: A `String` containing the brand name to be formatted
///
/// # Returns
///
/// A `String` with the standardized brand name
///
/// # Formatting Rules
///
/// - Removes leading and trailing whitespace
/// - Converts input to lowercase for consistent matching
/// - Maps specific brand names to their preferred representation
/// - Maintains original input for unrecognized brands
///
/// # Examples
///
/// ```rust
/// assert_eq!(format_brand("bp".to_string()), "BP");
/// assert_eq!(format_brand("  Sainsbury's  ".to_string()), "Sainsbury's");
/// assert_eq!(format_brand("unknown brand".to_string()), "unknown brand");
/// ```
///
/// # Brand Mapping
///
/// Supports consistent formatting for various fuel station brands:
/// - "applegreen" → "Applegreen"
/// - "bp" → "BP"
/// - "esso" → "Esso"
/// - ... and many more predefined mappings
///
/// # Performance
///
/// - O(1) time complexity for brand matching
/// - Minimal overhead for string processing
fn format_brand(brand: String) -> String {
    let input_brand = brand.trim().to_lowercase();
    let output_brand = match input_brand.as_str() {
        "applegreen" => "Applegreen",
        "asda express" => "ASDA Express",
        "asda" => "ASDA",
        "bp" => "BP",
        "coop" => "Co Op",
        "essar" => "Essar",
        "esso" => "Esso",
        "gulf" => "Gulf",
        "harvest energy" => "Harvest Engery",
        "jet" => "JET",
        "morrisons" => "Morrisons",
        "murco" => "Murco",
        "sainsbury's" => "Sainsbury's",
        "shell" => "Shell",
        "tesco" => "Tesco",
        "texaco" => "Texaco",
        _ => brand.as_str(),
    };

    output_brand.to_string()
}
