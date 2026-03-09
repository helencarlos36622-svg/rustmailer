/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { Outlet } from '@tanstack/react-router'
import { Separator } from '@/components/ui/separator'
import { Main } from '@/components/layout/main'
import SidebarNav from './components/sidebar-nav'
import { Scale, ShieldEllipsis, Waypoints } from 'lucide-react'
import { FixedHeader } from '@/components/layout/fixed-header'

export default function Settings() {
  return (
    <>
      {/* ===== Top Heading ===== */}
      <FixedHeader />

      <Main fixed>
        <h1 className='text-2xl font-bold tracking-tight md:text-3xl'>
          Settings
        </h1>
        <Separator className='my-4 lg:my-6' />
        <div className='flex flex-1 flex-col space-y-2 md:space-y-2 overflow-hidden lg:flex-row lg:space-x-12 lg:space-y-0'>
          <aside className='top-0 lg:sticky lg:w-1/5'>
            <SidebarNav items={sidebarNavItems} />
          </aside>
          <div className='flex w-full p-1 pr-4 overflow-y-hidden'>
            <Outlet />
          </div>
        </div>
      </Main>
    </>
  )
}

const sidebarNavItems = [
  {
    title: 'License',
    icon: <Scale size={18} />,
    href: '/settings/license',
  },
  {
    title: 'Root',
    icon: <ShieldEllipsis size={18} />,
    href: '/settings/root',
  },
  {
    title: 'Proxy',
    icon: <Waypoints size={18} />,
    href: '/settings/proxy',
  }
]
