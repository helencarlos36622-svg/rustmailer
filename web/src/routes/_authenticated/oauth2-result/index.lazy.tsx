/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { createLazyFileRoute } from '@tanstack/react-router'
import OAuth2Result from '@/features/oauth2-result'

export const Route = createLazyFileRoute('/_authenticated/oauth2-result/')({
  component: OAuth2Result,
})
