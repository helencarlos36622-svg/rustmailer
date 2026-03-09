/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { Row } from '@tanstack/react-table'
import { AccountEntity } from '../data/schema'
import { Switch } from '@/components/ui/switch'
import { useState } from 'react'
import { ConfirmDialog } from '@/components/confirm-dialog'
import { ToastAction } from '@/components/ui/toast'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { update_account } from '@/api/account/api'
import { toast } from '@/hooks/use-toast'
import { AxiosError } from 'axios'

interface DataTableRowActionsProps {
  row: Row<AccountEntity>
}

export function EnableAction({ row }: DataTableRowActionsProps) {
  const [open, setOpen] = useState(false);
  const queryClient = useQueryClient();

  const updateMutation = useMutation({
    mutationFn: (enabled: boolean) =>
      update_account(row.original.id, { enabled }),
    onSuccess: () => {
      setOpen(false);
      toast({
        title: 'Account Updated',
        description: `Account has been successfully ${row.original.enabled ? 'disabled' : 'enabled'}.`,
        action: <ToastAction altText="Close">Close</ToastAction>,
      })
      queryClient.invalidateQueries({ queryKey: ['account-list'] })
    },
    onError: (error: AxiosError) => {
      const errorMessage =
        (error.response?.data as { message?: string })?.message ||
        error.message ||
        'Status update failed, please try again later'

      toast({
        variant: "destructive",
        title: 'Update Failed',
        description: errorMessage,
        action: <ToastAction altText="Try again">Try again</ToastAction>,
      })
    }
  })

  const handleConfirm = () => {
    updateMutation.mutate(!row.original.enabled)
  }

  return (
    <>
      <Switch
        checked={row.original.enabled}
        onCheckedChange={() => setOpen(true)}
        disabled={updateMutation.isPending}
      />
      <ConfirmDialog
        open={open}
        onOpenChange={setOpen}
        title={`${row.original.enabled ? 'Disable' : 'Enable'} Account`}
        desc={
          `Are you sure you want to ${row.original.enabled ? 'disable' : 'enable'} this account?` +
          (row.original.enabled ? ' This will prevent the account from being used.' : '')
        }
        destructive={row.original.enabled}
        confirmText={row.original.enabled ? 'Disable' : 'Enable'}
        isLoading={updateMutation.isPending}
        handleConfirm={handleConfirm}
      />
    </>
  )
}
