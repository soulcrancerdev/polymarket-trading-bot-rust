use anyhow::Result;
use mongodb::{
    bson::{doc, to_document},
    options::FindOneAndUpdateOptions,
    Client, Collection, Database,
};
use std::sync::Arc;

use crate::types::{UserActivity, UserPosition};

// MongoDB wrapper - stores trades & positions per trader
#[derive(Clone)]
pub struct Db {
    _client: Arc<Client>,
    db: Database,
}

impl Db {
    // Connect to MongoDB (default DB: polymarket_copytrading)
    pub async fn connect(uri: &str) -> Result<Self> {
        let client = Client::with_uri_str(uri).await?;
        let db = client.database("polymarket_copytrading");
        Ok(Self {
            _client: Arc::new(client),
            db: db.clone(),
        })
    }

    // Get collection for trader's activities (one per trader)
    pub fn activity_collection(&self, user_address: &str) -> Collection<UserActivity> {
        let name = format!("user_activities_{}", user_address.to_lowercase());
        self.db.collection(&name)
    }

    // Get collection for trader's positions
    pub fn position_collection(&self, user_address: &str) -> Collection<UserPosition> {
        let name = format!("user_positions_{}", user_address.to_lowercase());
        self.db.collection(&name)
    }

    // Count total activities for a trader
    pub async fn count_activities(&self, user_address: &str) -> Result<u64> {
        let coll = self.activity_collection(user_address);
        Ok(coll.estimated_document_count(None).await?)
    }

    // Insert new trade activity
    pub async fn insert_activity(&self, user_address: &str, activity: &UserActivity) -> Result<()> {
        let coll = self.activity_collection(user_address);
        coll.insert_one(activity, None).await?;
        Ok(())
    }

    // Find activity by tx hash (duplicate check)
    pub async fn find_activity_by_tx(
        &self,
        user_address: &str,
        transaction_hash: &str,
    ) -> Result<Option<UserActivity>> {
        let coll = self.activity_collection(user_address);
        let filter = doc! { "transactionHash": transaction_hash };
        Ok(coll.find_one(filter, None).await?)
    }

    // Find unprocessed trades (not executed by bot yet)
    pub async fn find_unprocessed_trades(&self, user_address: &str) -> Result<Vec<UserActivity>> {
        let coll = self.activity_collection(user_address);
        let filter = doc! {
            "type": "TRADE",
            "bot": false,
            "botExcutedTime": 0_i64
        };
        let mut cursor = coll.find(filter, None).await?;
        let mut out = Vec::new();
        while cursor.advance().await? {
            out.push(cursor.deserialize_current()?);
        }
        Ok(out)
    }

    pub async fn update_activity(
        &self,
        user_address: &str,
        id: &mongodb::bson::oid::ObjectId,
        update: &mongodb::bson::Document,
    ) -> Result<()> {
        let coll = self.activity_collection(user_address);
        let filter = doc! { "_id": id };
        coll.update_one(filter, doc! { "$set": update }, None)
            .await?;
        Ok(())
    }

    pub async fn mark_historical_processed(&self, user_address: &str) -> Result<u64> {
        let coll = self.activity_collection(user_address);
        let filter = doc! { "bot": false };
        let update = doc! { "$set": { "bot": true, "botExcutedTime": 999_i64 } };
        let result = coll.update_many(filter, update, None).await?;
        Ok(result.modified_count)
    }

    pub async fn upsert_position(&self, user_address: &str, position: &UserPosition) -> Result<()> {
        let coll = self.position_collection(user_address);
        let filter = doc! {
            "asset": position.asset.as_deref().unwrap_or(""),
            "conditionId": position.condition_id.as_deref().unwrap_or("")
        };
        let mut set_doc = to_document(position)?;
        set_doc.remove("_id");
        let update = doc! { "$set": set_doc };
        let opts = FindOneAndUpdateOptions::builder().upsert(true).build();
        coll.find_one_and_update(filter, update, opts).await?;
        Ok(())
    }

    pub async fn get_positions(&self, user_address: &str) -> Result<Vec<UserPosition>> {
        let coll = self.position_collection(user_address);
        let mut cursor = coll.find(doc! {}, None).await?;
        let mut out = Vec::new();
        while cursor.advance().await? {
            out.push(cursor.deserialize_current()?);
        }
        Ok(out)
    }

    pub fn config_collection(&self) -> Collection<mongodb::bson::Document> {
        self.db.collection("configs")
    }

    async fn get_next_sequence_number(&self, key_prefix: &str) -> Result<u32> {
        let coll = self.config_collection();
        let filter = doc! {
            "key": { "$regex": format!("^{}_\\d+$", key_prefix) }
        };
        let mut cursor = coll.find(filter, None).await?;
        let mut numbers = Vec::new();
        while cursor.advance().await? {
            let doc = cursor.deserialize_current()?;
            if let Ok(key) = doc.get_str("key") {
                if let Some(underscore_pos) = key.rfind('_') {
                    if let Ok(num) = key[underscore_pos + 1..].parse::<u32>() {
                        numbers.push(num);
                    }
                }
            }
        }
        Ok(if numbers.is_empty() {
            1
        } else {
            numbers.iter().max().unwrap() + 1
        })
    }

    pub async fn set_config(&self, key: &str, value: &str) -> Result<()> {
        let coll = self.config_collection();
        let mut final_key = key.to_string();

        if key == "PRIVATE_KEY" {
            let filter = doc! {
                "key": { "$regex": "^PRIVATE_KEY_\\d+$" },
                "value": value
            };
            let existing = coll.find_one(filter, None).await?;
            if existing.is_some() {
                return Ok(());
            }

            let sequence_number = self.get_next_sequence_number("PRIVATE_KEY").await?;
            final_key = format!("PRIVATE_KEY_{}", sequence_number);
        }

        let doc = doc! {
            "key": &final_key,
            "value": value,
            "timestamp": mongodb::bson::DateTime::now(),
        };
        coll.insert_one(doc, None).await?;
        Ok(())
    }

    pub async fn find_all_buy_activities_for_asset(
        &self,
        user_address: &str,
        asset: &str,
        condition_id: &Option<String>,
    ) -> Result<Vec<UserActivity>> {
        let coll = self.activity_collection(user_address);
        let mut filter = doc! {
            "asset": asset,
            "side": "BUY",
            "bot": true,
            "myBoughtSize": { "$exists": true, "$gt": 0.0 }
        };
        if let Some(ref cid) = condition_id {
            filter.insert("conditionId", cid);
        }
        let mut cursor = coll.find(filter, None).await?;
        let mut out = Vec::new();
        while cursor.advance().await? {
            out.push(cursor.deserialize_current()?);
        }
        Ok(out)
    }

    pub async fn update_many_activities(
        &self,
        user_address: &str,
        asset: &str,
        condition_id: &Option<String>,
        update: &mongodb::bson::Document,
    ) -> Result<u64> {
        let coll = self.activity_collection(user_address);
        let mut filter = doc! {
            "asset": asset,
            "side": "BUY",
            "bot": true,
            "myBoughtSize": { "$exists": true, "$gt": 0.0 }
        };
        if let Some(ref cid) = condition_id {
            filter.insert("conditionId", cid);
        }
        let result = coll.update_many(filter, doc! { "$set": update }, None).await?;
        Ok(result.modified_count)
    }

    pub async fn close(&self) -> Result<()> {
        Ok(())
    }
}
