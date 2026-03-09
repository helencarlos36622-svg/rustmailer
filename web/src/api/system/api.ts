/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import axiosInstance from "@/api/axiosInstance";
import { Proxy } from "@/features/settings/proxy/data/schema";

export interface Release {
    tag_name: string;
    published_at: string;
    body: string;
    html_url: string;
}

export interface ReleaseNotification {
    latest: Release | null;  // `latest` can be null if the release information is not available
    is_newer: boolean;
    error_message: string | null;  // New field to store error message when the request fails
}

interface LicenseCheckResult {
    expired: boolean;
    days: number;
}

interface Notifications {
    release: ReleaseNotification;
    license: LicenseCheckResult;
}

export const get_notifications = async () => {
    const response = await axiosInstance.get<Notifications>(`/api/v1/notifications`);
    return response.data;
};


// TimeSeriesPoint.ts
export interface TimeSeriesPoint {
    timestamp: number;  // i64 in Rust becomes number in TypeScript
    value: number;      // u64 in Rust becomes number in TypeScript (TS doesn't have unsigned)
}

// MetricsTimeSeries.ts
export interface MetricsTimeSeries {
    imap_traffic_sent: TimeSeriesPoint[];
    imap_traffic_received: TimeSeriesPoint[];
    email_sent_success: TimeSeriesPoint[];
    email_sent_failure: TimeSeriesPoint[];
    email_sent_bytes: TimeSeriesPoint[];
    new_email_arrival: TimeSeriesPoint[];
    mail_flag_change: TimeSeriesPoint[];
    email_opens: TimeSeriesPoint[];
    email_clicks: TimeSeriesPoint[];
    event_dispatch_success_http: TimeSeriesPoint[];
    event_dispatch_success_nats: TimeSeriesPoint[];
    event_dispatch_failure_http: TimeSeriesPoint[];
    event_dispatch_failure_nats: TimeSeriesPoint[];
    email_task_queue_length: TimeSeriesPoint[];
    hook_task_queue_length: TimeSeriesPoint[];
}

// Overview.ts
export interface Overview {
    pending_email_task_num: number;    // usize -> number
    pending_hook_task_num: number;    // usize -> number
    account_num: number;             // usize -> number
    uptime: number;
    rustmailer_version: string;
    time_series: MetricsTimeSeries;
}



export const get_overview = async () => {
    const response = await axiosInstance.get<Overview>(`/api/v1/overview`);
    return response.data;
};

export const list_proxy = async () => {
    const response = await axiosInstance.get<Proxy[]>(`/api/v1/list-proxy`);
    return response.data;
};

export const delete_proxy = async (id: number) => {
    const response = await axiosInstance.delete(`/api/v1/proxy/${id}`);
    return response.data;
};

export const update_proxy = async (id: number, url: string) => {
    const response = await axiosInstance.post(`/api/v1/proxy/${id}`, url, {
        headers: {
            "Content-Type": "text/plain",
        },
    });
    return response.data;
};

export const add_proxy = async (url: string) => {
    const response = await axiosInstance.post(`/api/v1/proxy`, url, {
        headers: {
            "Content-Type": "text/plain",
        },
    });
    return response.data;
};