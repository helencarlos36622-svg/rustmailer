/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { createLazyFileRoute } from '@tanstack/react-router'
import GeneralError from '@/features/errors/general-error'

export const Route = createLazyFileRoute('/(errors)/500')({
  component: GeneralError,
})
