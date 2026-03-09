/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { createLazyFileRoute } from '@tanstack/react-router'
import UnauthorisedError from '@/features/errors/unauthorized-error'

export const Route = createLazyFileRoute('/(errors)/401')({
  component: UnauthorisedError,
})
