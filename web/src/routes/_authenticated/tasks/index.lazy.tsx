/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */
 
import { createLazyFileRoute } from '@tanstack/react-router'
import Tasks from '@/features/tasks'

export const Route = createLazyFileRoute('/_authenticated/tasks/')({
  component: Tasks,
})
