/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import axiosInstance from "@/api/axiosInstance";
import { AccountEntity, MailerType } from "@/features/accounts/data/schema";
import { PaginatedResponse } from "..";

export interface MinimalAccount {
    id: number;
    email: string;
    mailer_type: MailerType;
}

export const minimal_account_list = async () => {
    const response = await axiosInstance.get<MinimalAccount[]>("/api/v1/minimal-account-list");
    return response.data;
};


export interface ErrorMessage {
    error: string;
    at: number; // milliseconds timestamp
}

export interface AccountRunningState {
    account_id: number;
    last_full_sync_start: number;
    last_full_sync_end?: number,
    last_incremental_sync_start: number;
    last_incremental_sync_end?: number,
    errors: ErrorMessage[];
    is_initial_sync_completed: boolean;
    initial_sync_folders: string[];
    current_syncing_folder?: string | null;
    current_batch_number?: number | null;
    current_total_batches?: number | null;
    initial_sync_start_time?: number;
    initial_sync_end_time?: number;
}

export const account_state = async (account_id: number) => {
    const response = await axiosInstance.get<AccountRunningState>(`/api/v1/account-state/${account_id}`);
    return response.data;
};

export const create_account = async (data: Record<string, any>) => {
    const response = await axiosInstance.post("/api/v1/account", data);
    return response.data;
};

export const list_accounts = async () => {
    const response = await axiosInstance.get<PaginatedResponse<AccountEntity>>("/api/v1/list-accounts?desc=true");
    return response.data;
};

export const update_account = async (account_id: number, data: Record<string, any>) => {
    const response = await axiosInstance.post(`/api/v1/account/${account_id}`, data);
    return response.data;
};

export const remove_account = async (account_id: number) => {
    const response = await axiosInstance.delete(`/api/v1/account/${account_id}`);
    return response.data;
};

export interface AutoConfigResult {
    imap: ServerConfig;
    smtp: ServerConfig;
    oauth2?: OAuth2Config;
}

export interface ServerConfig {
    host: string;
    port: number;
    encryption: 'None' | 'Ssl' | 'StartTls';
}

export interface OAuth2Config {
    issuer: string;
    scope: string;
    auth_url: string;
    token_url: string;
}

export const autoconfig = async (email: string) => {
    const response = await axiosInstance.get<AutoConfigResult>(`/api/v1/autoconfig/${email}`);
    return response.data;
};
