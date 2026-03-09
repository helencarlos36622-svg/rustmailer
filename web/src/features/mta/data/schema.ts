/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

interface SmtpCredentials {
  username: string;
  password: string;
}

interface SmtpServerConfig {
  host: string;
  port: number;
  encryption: 'StartTls' | 'None' | 'Ssl';
}

export interface MTARecord {
  id: number;
  description?: string;
  credentials: SmtpCredentials;
  server: SmtpServerConfig;
  dsn_capable: boolean;
  created_at: number;
  updated_at: number;
  last_access_at: number;
  use_proxy?: number;
}
