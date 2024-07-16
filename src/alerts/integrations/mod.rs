pub mod discord;

use std::error::Error;

// crate imports
use crate::probe::model::ProbeAlert;

pub async fn alert_router(
    alert: &ProbeAlert
) -> Result<String, Box<dyn Error + Send>> {
    if alert.url.starts_with("https://discord.com/api/webhooks") {
        return Ok("discord".to_string());
        
    } else {
        return Ok("any".to_string());
    }
}