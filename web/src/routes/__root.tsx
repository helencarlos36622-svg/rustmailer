/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { QueryClient } from '@tanstack/react-query'
import { createRootRouteWithContext, Outlet } from '@tanstack/react-router'
// import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
// import { TanStackRouterDevtools } from '@tanstack/router-devtools'
import { Toaster } from '@/components/ui/toaster'
import GeneralError from '@/features/errors/general-error'
import NotFoundError from '@/features/errors/not-found-error'

export const Route = createRootRouteWithContext<{
  queryClient: QueryClient
}>()({
  component: () => {
    return (
      <>
        <Outlet />
        <Toaster />
        {/* {import.meta.env.MODE === 'development' && (
          <>
            <ReactQueryDevtools buttonPosition='bottom-left' />
            <TanStackRouterDevtools position='bottom-right' />
          </>
        )} */}
      </>
    )
  },
  notFoundComponent: NotFoundError,
  errorComponent: GeneralError,
})
