/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { PaginatedResponse } from "@/api";
import axiosInstance from "@/api/axiosInstance";
import { EmailEnvelope } from "@/features/mailbox/data/schema";
import { saveAs } from 'file-saver';

export const list_messages = async (accountId: number, mailbox: string, page_size: number, remote: boolean, next_page_token?: string) => {
    const params = new URLSearchParams({
        mailbox,
        page_size: String(page_size),
        desc: "true",
        remote: String(remote),
    });
    if (next_page_token) {
        params.append("next_page_token", next_page_token);
    }

    const response = await axiosInstance.get<PaginatedResponse<EmailEnvelope>>(
        `/api/v1/list-messages/${accountId}?${params.toString()}`
    );
    return response.data;
};

export const search_messages = async (accountId: number, page_size: number, remote: boolean, data: Record<string, any>, next_page_token?: string) => {
    const params = new URLSearchParams({
        page_size: String(page_size),
        desc: "true",
        remote: String(remote),
    });

    if (next_page_token) {
        params.append("next_page_token", next_page_token);
    }

    const response = await axiosInstance.post<PaginatedResponse<EmailEnvelope>>(
        `/api/v1/search-message/${accountId}?${params.toString()}`,
        data
    );

    return response.data;
};

export const download_attachment = async (accountId: number, attachmentFileName: string | undefined, data: Record<string, any>) => {
    const response = await axiosInstance.post(`/api/v1/message-attachment/${accountId}`, data, { responseType: 'blob' });
    const contentDisposition = response.headers['content-disposition'];
    let filename = attachmentFileName ?? "download-file";
    if (contentDisposition && contentDisposition.includes('filename=')) {
        filename = contentDisposition
            .split('filename=')[1]
            .split(';')[0]
            .replace(/['"]/g, '');
    }
    const blob = new Blob([response.data]);
    saveAs(blob, filename);
};


export interface PlainText {
    content: string;
    truncated: boolean;
}


export interface AttachmentInfo {
    /** MIME content type of the attachment (e.g., `image/png`, `application/pdf`). */
    file_type: string;
    /** Content transfer encoding (usually `"base64"`). */
    transfer_encoding: string;
    /** Content-ID, used for inline attachments (referenced in HTML by `cid:` URLs). */
    content_id: string;
    /** Whether the attachment is marked as inline (true) or a regular file (false). */
    inline: boolean;
    /** Original filename of the attachment, if provided. */
    filename: string;
    /** Gmail-specific attachment ID, used to fetch the attachment via Gmail API. */
    id: string;
    /** Size of the attachment in bytes. */
    size: number;
}




export interface MessageContentResponse {
    plain?: PlainText;
    html?: string;
    attachments?: AttachmentInfo[]
}

export const getContent = (messageContent: MessageContentResponse): string | null => {
    if (messageContent.html) {
        return messageContent.html;
    } else if (messageContent.plain) {
        return messageContent.plain.content;
    }
    return null;
};

export const load_message = async (accountId: number, payload: Record<string, any>) => {
    const response = await axiosInstance.post<MessageContentResponse>(`/api/v1/message-content/${accountId}`, payload);
    return response.data;
};

export const flag_messages = async (accountId: number, payload: Record<string, any>) => {
    const response = await axiosInstance.post(`/api/v1/flag-messages/${accountId}`, payload);
    return response.data;
};

export const delete_messages = async (accountId: number, payload: Record<string, any>) => {
    const response = await axiosInstance.post(`/api/v1/delete-messages/${accountId}`, payload);
    return response.data;
};


export const move_messages = async (accountId: number, payload: Record<string, any>) => {
    const response = await axiosInstance.post(`/api/v1/move-messages/${accountId}`, payload);
    return response.data;
};

export const copy_messages = async (accountId: number, payload: Record<string, any>) => {
    const response = await axiosInstance.post(`/api/v1/copy-messages/${accountId}`, payload);
    return response.data;
};

export const get_full_message = async (accountId: number, mailbox: string, id: string, subject: string) => {
    const response = await axiosInstance.get(`/api/v1/raw-message/${accountId}?mailbox=${mailbox}&id=${id}`, { responseType: 'blob' });
    const blob = new Blob([response.data]);
    let filename = subject.replace(/\.eml$/i, '');
    saveAs(blob, `${filename}.eml`);
};


export const reply_mail = async (accountId: number, payload: Record<string, any>) => {
    const response = await axiosInstance.post(`/api/v1/reply-mail/${accountId}`, payload);
    return response.data;
};

export const forward_mail = async (accountId: number, payload: Record<string, any>) => {
    const response = await axiosInstance.post(`/api/v1/forward-mail/${accountId}`, payload);
    return response.data;
};
