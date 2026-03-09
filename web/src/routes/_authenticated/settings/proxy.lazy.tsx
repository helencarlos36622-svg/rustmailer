/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import ProxyManagerPage from '@/features/settings/proxy'
import { createLazyFileRoute } from '@tanstack/react-router'

export const Route = createLazyFileRoute('/_authenticated/settings/proxy')({
  component: ProxyManagerPage,
})
