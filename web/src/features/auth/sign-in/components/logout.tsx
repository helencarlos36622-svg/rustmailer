/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { IconLogout } from '@tabler/icons-react'
import { ConfirmDialog } from '@/components/confirm-dialog'

interface Props {
    open: boolean
    onOpenChange: (open: boolean) => void
    handleConfirm: () => void
}

export function LogoutConfirmDialog({ open, onOpenChange, handleConfirm }: Props) {
    const handleLogout = () => {
        handleConfirm();
        onOpenChange(false);
    };

    return (
        <ConfirmDialog
            open={open}
            onOpenChange={onOpenChange}
            handleConfirm={handleLogout}
            className="max-w-md"
            title={
                <span className='text-destructive'>
                    <IconLogout
                        className='mr-1 inline-block stroke-destructive'
                        size={18}
                    />{' '}
                    Log out
                </span>
            }
            desc={
                <p>
                    Are you sure you want to log out of RustMailer?
                    <br />
                    You will need to log in again to access your account.
                </p>
            }
            confirmText='Log out'
            cancelBtnText='Cancel'
        />
    )
}