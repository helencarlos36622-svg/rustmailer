/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { createFileRoute } from '@tanstack/react-router'
import SignIn from '@/features/auth/sign-in'

export const Route = createFileRoute('/(auth)/sign-in')({
  component: SignIn,
})
