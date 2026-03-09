/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import {
  Sidebar,
  SidebarContent,
  SidebarHeader,
  SidebarMenuButton,
  SidebarRail,
  useSidebar,
} from '@/components/ui/sidebar'
import { NavGroup } from '@/components/layout/nav-group'
import Logo from '@/assets/logo.svg'
import { sidebarData } from './data/sidebar-data'

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const { open } = useSidebar();
  return (
    <Sidebar collapsible='icon' variant='sidebar' {...props}>
      <SidebarHeader>
        <SidebarMenuButton
          size='lg'
          className='data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground'
        >
          <div className='flex aspect-square size-16 items-center justify-center rounded-lg text-sidebar-primary-foreground'>
            <img
              className={open ? "relative ml-[12px] mr-[12px]" : "mr-[30px]"}
              src={Logo}
              width={open ? 60 : 40}
              height={open ? 60 : 40}
              alt='Logo'
            />
          </div>
          <div className='grid flex-1 text-left text-lg leading-tight'>
            <span className='truncate font-semibold'>
              RustMailer
            </span>
          </div>
        </SidebarMenuButton>
      </SidebarHeader>
      <SidebarContent>
        {sidebarData.navGroups.map((props) => (
          <NavGroup key={props.title} {...props} />
        ))}
      </SidebarContent>
      {/* <SidebarFooter>
        <NavUser user={sidebarData.user} />
      </SidebarFooter> */}
      <SidebarRail />
    </Sidebar>
  )
}
