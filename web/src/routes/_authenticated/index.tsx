/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { createFileRoute } from '@tanstack/react-router'
import Dashboard from '@/features/dashboard'

export const Route = createFileRoute('/_authenticated/')({
  component: Dashboard,
})
