/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { LogoutConfirmDialog } from '@/features/auth/sign-in/components/logout';
import { resetAccessToken } from '@/stores/authStore';
import { useNavigate } from '@tanstack/react-router';
import { useState } from 'react';

export function ProfileDropdown() {
  const navigate = useNavigate()
  const [isLogoutDialogOpen, setIsLogoutDialogOpen] = useState(false)

  const handleLogout = () => {
    resetAccessToken()
    navigate({ to: '/sign-in' })
  }

  return (
    <>
      <DropdownMenu modal={false}>
        <DropdownMenuTrigger asChild>
          <Button variant='ghost' className='relative h-8 w-8 rounded-full'>
            <Avatar className='h-8 w-8'>
              <AvatarFallback className='text-xs'>root</AvatarFallback>
            </Avatar>
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent className='w-56' align='end' forceMount>
          <DropdownMenuItem onClick={() => setIsLogoutDialogOpen(true)}>
            Log out
            <DropdownMenuShortcut>⇧⌘Q</DropdownMenuShortcut>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
      <LogoutConfirmDialog
        open={isLogoutDialogOpen}
        onOpenChange={setIsLogoutDialogOpen}
        handleConfirm={handleLogout}
      />
    </>
  )
}
