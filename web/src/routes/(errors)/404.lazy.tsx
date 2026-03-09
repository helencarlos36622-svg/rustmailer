/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { createLazyFileRoute } from '@tanstack/react-router'
import NotFoundError from '@/features/errors/not-found-error'

export const Route = createLazyFileRoute('/(errors)/404')({
  component: NotFoundError,
})
