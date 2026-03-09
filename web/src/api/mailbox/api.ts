/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import axiosInstance from "@/api/axiosInstance";


export interface MailboxData {
    attributes: { attr: string; extension: string | null }[];
    delimiter: string | null;
    exists: number;
    flags: { custom: null; flag: string }[];
    highest_modseq: number | null;
    id: number;
    name: string;
    permanent_flags: any[];
    uid_next: number | null;
    uid_validity: number | null;
    unseen: number | null;
}

export const list_account_mailboxes = async (accountId: number, remote: boolean) => {
    const response = await axiosInstance.get<MailboxData[]>(`/api/v1/list-mailboxes/${accountId}?remote=${remote}`);
    return response.data;
};