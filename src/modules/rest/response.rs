// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::modules::database::Paginated;

/// Represents a paginated response containing a subset of items along with pagination metadata.
///
/// This generic structure is commonly used to return paged data from list or search endpoints.
/// The type `S` represents the individual item type within the result set.
///
/// # Type Parameters
/// - `S`: The type of each item in the `items` list. Must implement several traits for serialization
///   and OpenAPI documentation.
///
/// # Fields
/// - `current_page`: The current page number (1-based). `None` if unspecified or not applicable.
/// - `page_size`: The number of items per page. `None` if unspecified.
/// - `total_items`: The total number of items matching the query.
/// - `items`: The list of items returned for the current page.
/// - `total_pages`: The total number of pages available. `None` if not calculated.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
pub struct DataPage<S>
where
    S: Serialize
        + std::fmt::Debug
        + std::marker::Unpin
        + Send
        + Sync
        + poem_openapi::types::Type
        + poem_openapi::types::ParseFromJSON
        + poem_openapi::types::ToJSON,
{
    /// The current page number (starting from 1).
    pub current_page: Option<u64>,
    /// The number of items per page.
    pub page_size: Option<u64>,
    /// The total number of items across all pages.
    pub total_items: u64,
    /// The list of items returned on the current page.
    pub items: Vec<S>,
    /// The total number of pages. This is optional and may not be set if not calculated.
    pub total_pages: Option<u64>,
}

impl<
        S: Serialize
            + std::fmt::Debug
            + std::marker::Unpin
            + Send
            + Sync
            + poem_openapi::types::Type
            + poem_openapi::types::ParseFromJSON
            + poem_openapi::types::ToJSON,
    > DataPage<S>
{
    pub fn new(
        current_page: Option<u64>,
        page_size: Option<u64>,
        total_items: u64,
        total_pages: Option<u64>,
        items: Vec<S>,
    ) -> Self {
        Self {
            current_page,
            page_size,
            total_items,
            total_pages,
            items,
        }
    }
}

impl<
        S: Serialize
            + std::fmt::Debug
            + std::marker::Unpin
            + Send
            + Sync
            + poem_openapi::types::Type
            + poem_openapi::types::ParseFromJSON
            + poem_openapi::types::ToJSON,
    > From<Paginated<S>> for DataPage<S>
{
    fn from(paginated: Paginated<S>) -> Self {
        DataPage {
            current_page: paginated.page,
            page_size: paginated.page_size,
            total_items: paginated.total_items,
            total_pages: paginated.total_pages,
            items: paginated.items,
        }
    }
}

/// Represents a paginated response containing a subset of items along with pagination metadata.
///
/// This generic structure is commonly used to return paged data from list or search endpoints.
/// The type `S` represents the individual item type within the result set.
///
/// # Type Parameters
/// - `S`: The type of each item in the `items` list. Must implement several traits for serialization
///   and OpenAPI documentation.
///
/// # Fields
/// - `current_page`: The current page number (1-based). `None` if unspecified or not applicable.
/// - `page_size`: The number of items per page. `None` if unspecified.
/// - `total_items`: The total number of items matching the query.
/// - `items`: The list of items returned for the current page.
/// - `total_pages`: The total number of pages available. `None` if not calculated.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
pub struct CursorDataPage<S>
where
    S: Serialize
        + std::fmt::Debug
        + std::marker::Unpin
        + Send
        + Sync
        + poem_openapi::types::Type
        + poem_openapi::types::ParseFromJSON
        + poem_openapi::types::ToJSON,
{
    /// A cursor used for pagination (returned by the previous response).
    /// Pass this token to retrieve the next page of results.
    /// If `None`, there are no more pages available.
    pub next_page_token: Option<String>,
    /// The number of items per page.
    pub page_size: Option<u64>,
    /// The total number of items across all pages.
    pub total_items: u64,
    /// The list of items returned on the current page.
    pub items: Vec<S>,
    /// The total number of pages. This is optional and may not be set if not calculated.
    pub total_pages: Option<u64>,
}

impl<
        S: Serialize
            + std::fmt::Debug
            + std::marker::Unpin
            + Send
            + Sync
            + poem_openapi::types::Type
            + poem_openapi::types::ParseFromJSON
            + poem_openapi::types::ToJSON,
    > CursorDataPage<S>
{
    pub fn new(
        next_page_token: Option<String>,
        page_size: Option<u64>,
        total_items: u64,
        total_pages: Option<u64>,
        items: Vec<S>,
    ) -> Self {
        Self {
            next_page_token,
            page_size,
            total_items,
            total_pages,
            items,
        }
    }
}
