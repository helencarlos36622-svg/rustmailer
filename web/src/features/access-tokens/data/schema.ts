/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

interface AccountInfo {
  id: number;
  email: string;
}

interface RateLimit {
  quota: number;
  interval: number;
}

interface AccessControl {
  ip_whitelist?: string[];
  rate_limit?: RateLimit;
}

type AccessTokenScope = 'Api' | 'Metrics';
interface AccessToken {
  token: string;
  accounts: AccountInfo[];
  created_at: number;
  updated_at: number;
  description?: string;
  access_scopes: AccessTokenScope[];
  last_access_at: number;
  acl?: AccessControl;
}

export type { AccessToken, AccountInfo, AccessTokenScope, AccessControl, RateLimit };