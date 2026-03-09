// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::cmp::min;

use crate::{
    modules::{
        database::Paginated,
        error::{code::ErrorCode, RustMailerResult},
    },
    raise_error,
};

pub fn  paginate_vec<T: Clone>(
    items: &Vec<T>,
    page: Option<u64>,
    page_size: Option<u64>,
) -> RustMailerResult<Paginated<T>> {
    let total_items = items.len() as u64;

    let (offset, total_pages) = match (page, page_size) {
        (Some(p), Some(s)) if p > 0 && s > 0 => {
            let offset = (p - 1) * s;
            let total_pages = if total_items > 0 {
                (total_items + s - 1) / s
            } else {
                0
            };
            (Some(offset), Some(total_pages))
        }
        (Some(0), _) | (_, Some(0)) => {
            return Err(raise_error!(
                "'page' and 'page_size' must be greater than 0.".into(),
                ErrorCode::InvalidParameter
            ));
        }
        _ => (None, None),
    };

    let data = match offset {
        Some(offset) if offset >= total_items => vec![],
        Some(offset) => {
            let end = min(offset + page_size.unwrap_or(total_items), total_items) as usize;
            items[offset as usize..end].to_vec()
        }
        None => items.clone(),
    };

    Ok(Paginated::new(
        page,
        page_size,
        total_items,
        total_pages,
        data,
    ))
}
