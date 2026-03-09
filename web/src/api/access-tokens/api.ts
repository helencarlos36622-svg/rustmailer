/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import axiosInstance from "@/api/axiosInstance";
import { AccessToken } from "@/features/access-tokens/data/schema";

export const login = async (password: string) => {
    const response = await axiosInstance.post(`/api/login`, password, {
        headers: {
            "Content-Type": "text/plain",
        },
    });
    return response.data;
};

export const reset_root_token = async () => {
    const response = await axiosInstance.post("/api/v1/reset-root-token");
    return response.data;
};

export const reset_root_password = async (password: string) => {
    const response = await axiosInstance.post("/api/v1/reset-root-password", password, {
        headers: {
            "Content-Type": "text/plain",
        },
    });
    return response.data;
};

export const list_access_tokens = async () => {
    const response = await axiosInstance.get<AccessToken[]>("/api/v1/access-token-list");
    return response.data;
};

export const create_access_token = async (data: Record<string, any>) => {
    const response = await axiosInstance.post("/api/v1/access-token", data);
    return response.data;
}

export const update_access_token = async (token: string, data: Record<string, any>) => {
    const response = await axiosInstance.post(`/api/v1/access-token/${token}`, data);
    return response.data;
}

export const delete_access_token = async (token: string) => {
    const response = await axiosInstance.delete(`/api/v1/access-token/${token}`);
    return response.data;
}