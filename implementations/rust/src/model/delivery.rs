//! `delivery-media-available.schema.json`: `delivery.media_available`.

use serde::{Deserialize, Serialize};

use super::common::{Extensions, Locator, TimeRanges};

/// A media store announcing that media has arrived. The schema requires a
/// `source` or a `locator`; the validator enforces it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaAvailable {
    /// Always `delivery.media_available`.
    pub message_type: MediaAvailableType,
    /// UUID of the delivery.
    pub delivery_id: String,
    /// The asset whose media arrived.
    pub asset_id: String,
    /// Fully-qualified TAMS Source URI, `tams://store/id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Time range or ranges: onto the Source timeline against a TAMS source,
    /// offset from the file's own zero against a locator.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_range: Option<TimeRanges>,
    /// Receiving system (`system_id`).
    pub arrived_in: String,
    /// RFC 3339.
    pub arrived_at: String,
    /// Vendor fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
    /// Non-TAMS store reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locator: Option<Locator>,
}

/// The `delivery.media_available` constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum MediaAvailableType {
    /// `delivery.media_available`.
    #[default]
    #[serde(rename = "delivery.media_available")]
    MediaAvailable,
}
