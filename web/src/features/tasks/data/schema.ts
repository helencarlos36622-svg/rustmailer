/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

// TaskStatus enum
export enum TaskStatus {
  Scheduled = "Scheduled",  // Default value
  Running = "Running",
  Success = "Success",
  Failed = "Failed",
  Removed = "Removed",
  Stopped = "Stopped"
}

// MailEnvelope interface
interface MailEnvelope {
  from: string;
  recipients: string[];
}

// DSNConfig interface
interface DSNConfig {
  ret: ReturnContent;
  envid?: string;
  notify: NotifyOption[];
  orcpt?: string;
}

// ReturnContent enum
enum ReturnContent {
  FULL = "FULL",
  HDRS = "HDRS"  // Default value
}

// NotifyOption enum
enum NotifyOption {
  Success = "Success",
  Failure = "Failure",  // Default value
  Delay = "Delay",
  Never = "Never"
}

// Updated QueuedEmailTask interface with all types included
export interface EmailTask {
  id: number;
  created_at: number;
  status: TaskStatus;
  stopped_reason?: string;
  error?: string;
  last_duration_ms?: number;
  retry_count?: number;
  scheduled_at: number;
  account_id: number;
  account_email: string;
  subject?: string;
  message_id?: string;
  from: string;
  to: string[];
  cc?: string[];
  bcc?: string[];
  attachment_count: number;
  cache_key: string;
  envelope?: MailEnvelope;
  save_to_sent: boolean;
  sent_folder?: string;
  send_at?: number;
  mta?: number;
  dsn?: DSNConfig;
  reply?: boolean;
  mailbox?: string;
  uid?: number;
}



export interface EventHookTask {
  id: number;
  created_at: number;
  status: TaskStatus;
  stopped_reason: string | null;
  error: string | null;
  last_duration_ms: number | null;
  retry_count: number | null;
  scheduled_at: number;
  account_id: number;
  account_email: string;
  event: Record<string, any>;
  event_type: EventType;
}

export type EventType =
  | 'EmailAddedToFolder'
  | 'EmailFlagsChanged'
  | 'EmailSentSuccess'
  | 'EmailSendingError'
  | 'UIDValidityChange'
  | 'MailboxDeletion'
  | 'MailboxCreation'
  | 'AccountFirstSyncCompleted'
  | 'EmailBounce'
  | 'EmailFeedBackReport'
  | 'EmailOpened'
  | 'EmailLinkClicked';