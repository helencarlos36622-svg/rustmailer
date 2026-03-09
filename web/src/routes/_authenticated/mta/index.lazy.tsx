/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { createLazyFileRoute } from '@tanstack/react-router'
import MTA from '@/features/mta'

export const Route = createLazyFileRoute('/_authenticated/mta/')({
  component: MTA,
})
