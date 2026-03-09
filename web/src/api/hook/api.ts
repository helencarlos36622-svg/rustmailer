/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import axiosInstance from "@/api/axiosInstance";
import { EventHook, EventType } from "@/features/event-hooks/data/schema";
import { PaginatedResponse } from "..";

export const event_examples = async () => {
    const response = await axiosInstance.get<Record<EventType, any>>("/api/v1/event-examples");
    return response.data;
};

export const list_event_hook = async (page: number, page_size: number) => {
    const response = await axiosInstance.get<PaginatedResponse<EventHook>>(`/api/v1/event-hook-list?page=${page + 1}&page_size=${page_size}&desc=true`);
    return response.data;
};

export interface ResolveResult {
    result?: any;
    error?: string;
}


export const vrl_script_resolve = async (data: Record<string, any>) => {
    const response = await axiosInstance.post<ResolveResult>("/api/v1/vrl-script-resolve", data);
    return response.data;
};


export const create_event_hook = async (data: Record<string, any>) => {
    const response = await axiosInstance.post("/api/v1/event-hook", data);
    return response.data;
}

export const update_event_hook = async (id: number, data: Record<string, any>) => {
    const response = await axiosInstance.post(`/api/v1/event-hook/${id}`, data);
    return response.data;
}


export const delete_event_hook = async (id: number) => {
    const response = await axiosInstance.delete(`/api/v1/event-hook/${id}`);
    return response.data;
}