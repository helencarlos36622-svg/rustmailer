// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use itertools::Itertools;
use native_db::*;
use native_model::{native_model, Model};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::{
    id,
    modules::{
        database::{batch_delete_impl, insert_impl, list_all_impl, manager::DB_MANAGER},
        error::{code::ErrorCode, RustMailerResult},
    },
    raise_error,
};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
#[native_model(id = 14, version = 1)]
#[native_db]
pub struct DailyMetrics {
    #[primary_key]
    pub id: u64,
    pub metric: String,
    #[secondary_key]
    pub created_at: i64,
    pub value: u64,
    pub label: String,
}

impl DailyMetrics {
    pub async fn save(
        metric: String,
        value: u64,
        label: String,
        created_at: i64,
    ) -> RustMailerResult<()> {
        let item = DailyMetrics {
            id: id!(96),
            metric,
            created_at,
            value,
            label,
        };
        insert_impl(DB_MANAGER.meta_db(), item).await
    }

    pub async fn list_all() -> RustMailerResult<Vec<DailyMetrics>> {
        list_all_impl(&DB_MANAGER.meta_db()).await
    }

    pub async fn clean(cut: i64) -> RustMailerResult<()> {
        batch_delete_impl(DB_MANAGER.meta_db(), move |rw| {
            let to_delete: Vec<DailyMetrics> = rw
                .scan()
                .secondary(DailyMetricsKey::created_at)
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                .range(..cut)
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                .try_collect()
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;
            Ok(to_delete)
        })
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::modules::overview::metrics::DailyMetrics;

    #[tokio::test]
    async fn test_weekly_metrics_add_and_cleanup() {
        let metric_key = "metrics";
        let label_key = "label";

        // Insert 6 records with incremental values
        for i in 1..=6 {
            DailyMetrics::save(metric_key.into(), i * 100, label_key.into(), i as i64)
                .await
                .unwrap();
        }

        // Check initial insertion
        let all = DailyMetrics::list_all().await.unwrap();
        assert_eq!(all.len(), 6, "Expected 6 metrics before cleanup");

        // Clean records older than latest N = 3
        DailyMetrics::clean(3).await.unwrap();

        let all = DailyMetrics::list_all().await.unwrap();
        println!("{:#?}", all);
        println!("Remaining metrics after cleanup: {}", all.len());
        assert_eq!(all.len(), 4, "Expected 4 metrics after cleanup");
    }
}
