/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

interface AccountInfo {
  id: number;
  email: string;
}

// Define the EmailTemplate interface
export interface EmailTemplate {
  id: number;
  description?: string; // Optional field
  account?: AccountInfo; // Optional field
  subject: string;
  html: string;
  text?: string;
  format: string;
  preview?: string; // Optional field
  created_at: number;
  updated_at: number;
  last_access_at: number;
}