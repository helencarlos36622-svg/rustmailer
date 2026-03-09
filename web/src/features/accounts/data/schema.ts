/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

type Encryption = 'Ssl' | 'StartTls' | 'None';
type AuthType = 'Password' | 'OAuth2';
type Unit = 'Days' | 'Months' | 'Years';

// Interface definitions
interface AuthConfig {
  auth_type: AuthType;
  password?: string;
}

export interface SmtpConfig {
  host: string;
  port: number; // integer, 0-65535
  encryption: Encryption;
  auth: AuthConfig;
  use_proxy?: number;
}

export interface ImapConfig {
  host: string;
  port: number; // integer, 0-65535
  encryption: Encryption;
  auth: AuthConfig;
  use_proxy?: number;
}

interface RelativeDate {
  unit: Unit;
  value: number; // integer, minimum 1
}

interface DateSelection {
  fixed?: string; // format: "YYYY-MM-DD"
  relative?: RelativeDate;
}

export interface AccountEntity {
  id: number;
  imap?: ImapConfig;
  smtp?: SmtpConfig;
  enabled: boolean;
  mailer_type: MailerType,
  deleted: boolean;
  name?: string,
  email: string;
  minimal_sync?: boolean;
  capabilities?: string[];
  date_since?: DateSelection;
  folder_limit?: number,
  sync_folders?: string[];
  full_sync_interval_min?: number;
  incremental_sync_interval_sec: number;
  created_at: number;
  updated_at: number;
  use_proxy?: number
}


// Represents the method used to access/manage emails
export enum MailerType {
  /** Use IMAP/SMTP protocol */
  ImapSmtp = "ImapSmtp",
  /** Use Gmail API */
  GmailApi = "GmailApi",
  /** Use Graph API */
  GraphApi = "GraphApi",
}