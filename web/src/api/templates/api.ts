/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import axiosInstance from "@/api/axiosInstance";
import { EmailTemplate } from "@/features/templates/data/schema";
import { PaginatedResponse } from "..";

export const list_email_templates = async (pageIndex: number, pageSize: number) => {
    const response = await axiosInstance.get<PaginatedResponse<EmailTemplate>>(`/api/v1/list-template?page=${pageIndex + 1}&page_size=${pageSize}&desc=true`);
    return response.data;
};


export const create_template = async (payload: Record<string, any>) => {
    const response = await axiosInstance.post("/api/v1/template", payload);
    return response.data;
}

export const update_template = async (id: number, payload: Record<string, any>) => {
    const response = await axiosInstance.post(`/api/v1/template/${id}`, payload);
    return response.data;
}


export const delete_template = async (id: number) => {
    const response = await axiosInstance.delete(`/api/v1/template/${id}`);
    return response.data;
}


export const send_test_email = async (id: number, payload: Record<string, any>) => {
    const response = await axiosInstance.post(`/api/v1/template-send-test/${id}`, payload);
    return response.data;
}