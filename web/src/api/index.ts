/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

export interface PaginatedResponse<S> {
    next_page_token: string | null;
    page_size: number | null;
    total_items: number;
    items: S[];
    total_pages: number | null;
}