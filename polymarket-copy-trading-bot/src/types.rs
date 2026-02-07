use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserActivity {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub proxy_wallet: Option<String>,
    pub timestamp: Option<i64>,
    pub condition_id: Option<String>,
    #[serde(rename = "type")]
    pub activity_type: Option<String>,
    pub size: Option<f64>,
    pub usdc_size: Option<f64>,
    pub transaction_hash: Option<String>,
    pub price: Option<f64>,
    pub asset: Option<String>,
    pub side: Option<String>,
    pub outcome_index: Option<i32>,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub icon: Option<String>,
    pub event_slug: Option<String>,
    pub outcome: Option<String>,
    pub name: Option<String>,
    pub pseudonym: Option<String>,
    pub bio: Option<String>,
    pub profile_image: Option<String>,
    pub profile_image_optimized: Option<String>,
    pub bot: Option<bool>,
    #[serde(rename = "botExcutedTime")]
    pub bot_executed_time: Option<i64>,
    pub my_bought_size: Option<f64>,
}

impl UserActivity {
    pub fn side_buy(&self) -> bool {
        self.side.as_deref().unwrap_or("") == "BUY"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPosition {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub proxy_wallet: Option<String>,
    pub asset: Option<String>,
    pub condition_id: Option<String>,
    pub size: Option<f64>,
    pub avg_price: Option<f64>,
    pub initial_value: Option<f64>,
    pub current_value: Option<f64>,
    pub cash_pnl: Option<f64>,
    pub percent_pnl: Option<f64>,
    pub total_bought: Option<f64>,
    pub realized_pnl: Option<f64>,
    pub percent_realized_pnl: Option<f64>,
    pub cur_price: Option<f64>,
    pub redeemable: Option<bool>,
    pub mergeable: Option<bool>,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub icon: Option<String>,
    pub event_slug: Option<String>,
    pub outcome: Option<String>,
    pub outcome_index: Option<i32>,
    pub opposite_outcome: Option<String>,
    pub opposite_asset: Option<String>,
    pub end_date: Option<String>,
    pub negative_risk: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RtdsActivity {
    pub proxy_wallet: Option<String>,
    pub timestamp: Option<i64>,
    pub condition_id: Option<String>,
    #[serde(rename = "type")]
    pub activity_type: Option<String>,
    pub size: Option<f64>,
    pub price: Option<f64>,
    pub asset: Option<String>,
    pub side: Option<String>,
    pub outcome_index: Option<i32>,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub icon: Option<String>,
    pub event_slug: Option<String>,
    pub outcome: Option<String>,
    pub name: Option<String>,
    pub transaction_hash: Option<String>,
}

impl RtdsActivity {
    pub fn usdc_size(&self) -> f64 {
        self.size.unwrap_or(0.0) * self.price.unwrap_or(0.0)
    }
}
