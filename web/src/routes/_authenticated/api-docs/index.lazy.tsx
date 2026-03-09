/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { createLazyFileRoute } from '@tanstack/react-router'
import APIDocs from '@/components/api-docs'

export const Route = createLazyFileRoute('/_authenticated/api-docs/')({
  component: APIDocs,
})
