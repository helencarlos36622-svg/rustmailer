/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { createLazyFileRoute } from '@tanstack/react-router'
import Settings from '@/features/settings'

export const Route = createLazyFileRoute('/_authenticated/settings')({
  component: Settings,
})
