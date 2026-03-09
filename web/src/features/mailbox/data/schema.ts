/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

export function formatAddressList(list?: Addr[]): string[] {
  if (!list) return [];

  return list
    .filter(addr => addr.address)
    .map(addr => {
      const name = addr.name?.trim();
      const address = addr.address!.trim();
      return name ? `${name} <${address}>` : `<${address}>`;
    });
}

export function isCustomFlag(flag: EmailFlag): boolean {
  return flag === 'Custom';
}

export function getBadgeVariantFromFlag(flag: EmailFlag): "default" | "secondary" | "destructive" | "outline" | null | undefined {
  switch (flag) {
    case 'Deleted':
      return "destructive";
    case 'Draft':
      return "secondary";
    default:
      return "outline";
  }
}
type EmailFlag = 'Seen' | 'Answered' | 'Flagged' | 'Deleted' | 'Draft' | 'Recent' | 'MayCreate' | 'Custom';

export interface EmailEnvelope {
  account_id: number;
  mailbox_id: number;
  mailbox_name: string;
  id: string;
  internal_date?: number;
  size: number;
  flags?: EnvelopeFlag[];
  flags_hash?: number;
  bcc?: Addr[];
  cc?: Addr[];
  date?: number;
  from?: Addr;
  in_reply_to?: string;
  sender?: Addr;
  return_address?: string;
  message_id?: string;
  subject?: string;
  thread_id: number,
  thread_name?: string;
  mime_version?: string;
  references?: string[];
  reply_to?: Addr[];
  to?: Addr[];
  attachments?: Attachment[];
  body_meta?: EmailBodyPart[];
  received?: Received;
  labels: string[];
  is_read: boolean
}



interface EnvelopeFlag {
  flag: EmailFlag;
  custom?: string;
}

export interface Addr {
  name?: string;
  address?: string;
}

export interface Attachment {
  id: string;
  path: SegmentPath;
  filename?: string;
  inline: boolean;
  content_id?: string;
  size: number;
  file_type: string;
  transfer_encoding: Encoding;
}

type Encoding = 'None' | 'QuotedPrintable' | 'Base64';


interface SegmentPath {
  segments: number[];
}


export interface EmailBodyPart {
  id: string;
  part_type: 'Plain' | 'Html';
  path: SegmentPath;
  params?: Param[];
  size: number;
  transfer_encoding: Encoding;
}


interface Param {
  key: string;
  value: string;
}

interface Received {
  from?: string;
  by?: string;
  with?: string;
  date?: number;
}