/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

export type EventType =
  "AccountFirstSyncCompleted"
  | "EmailAddedToFolder"
  | "EmailBounce"
  | "EmailFeedBackReport"
  | "EmailFlagsChanged"
  | "EmailSendingError"
  | "EmailSentSuccess"
  | "MailboxCreation"
  | "MailboxDeletion"
  | "UIDValidityChange"
  | "EmailOpened"
  | "EmailLinkClicked";

export type HttpMethod = "Post" | "Put";

export type NatsAuthType = "None" | "Token" | "Password";
export type HookType = "Http" | "Nats";


export interface HttpConfig {
  target_url: string;
  http_method: HttpMethod;
  custom_headers?: Record<string, string>;
}

export interface NatsConfig {
  host: string;
  port: number;
  auth_type: NatsAuthType,
  token?: string;
  username?: string;
  password?: string;
  stream_name: string;
  namespace: string;
}

export interface EventHook {
  id: number,
  account_id?: number;
  email?: string;
  description?: string;
  created_at: number;
  updated_at: number;
  global: number,
  enabled: boolean;
  hook_type: HookType;
  http?: HttpConfig;
  nats?: NatsConfig;
  vrl_script: string;
  call_count: number;
  success_count: number;
  failure_count: number;
  last_error?: string;
  watched_events: EventType[];
  use_proxy?: number;
}
