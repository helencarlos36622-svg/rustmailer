/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import axiosInstance from "@/api/axiosInstance";
import { MTARecord } from "@/features/mta/data/schema";

export const list_mta = async () => {
    const response = await axiosInstance.get<{ items: MTARecord[] }>("/api/v1/list-mta");
    return response.data;
};

export const create_mta = async (mta: Record<string, any>) => {
    const response = await axiosInstance.post("/api/v1/mta", mta);
    return response.data;
};

export const update_mta = async (id: number, mta: Record<string, any>) => {
    const response = await axiosInstance.post(`/api/v1/mta/${id}`, mta);
    return response.data;
};

export const delete_mta = async (id: number) => {
    const response = await axiosInstance.delete(`/api/v1/mta/${id}`);
    return response.data;
};

export const send_test_email = async (id: number, payload: Record<string, any>) => {
    const response = await axiosInstance.post(`/api/v1/mta-send-test/${id}`, payload);
    return response.data;
};