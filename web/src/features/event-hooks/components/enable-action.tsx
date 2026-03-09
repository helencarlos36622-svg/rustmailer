/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { Row } from '@tanstack/react-table'
import { Switch } from '@/components/ui/switch'
import { useState } from 'react'
import { ConfirmDialog } from '@/components/confirm-dialog'
import { ToastAction } from '@/components/ui/toast'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from '@/hooks/use-toast'
import { AxiosError } from 'axios'
import { update_event_hook } from '@/api/hook/api'
import { EventHook } from '../data/schema'

interface DataTableRowActionsProps {
  row: Row<EventHook>
}

export function EnableAction({ row }: DataTableRowActionsProps) {
  const [open, setOpen] = useState(false);
  const queryClient = useQueryClient();

  const updateMutation = useMutation({
    mutationFn: (enabled: boolean) =>
      update_event_hook(row.original.id, { enabled }),
    onSuccess: () => {
      setOpen(false);
      toast({
        title: 'EventHook Updated',
        description: `EventHook has been successfully ${row.original.enabled ? 'disabled' : 'enabled'}.`,
        action: <ToastAction altText="Close">Close</ToastAction>,
      })
      queryClient.invalidateQueries({ queryKey: ['event-hook-list'] })
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
        title={`${row.original.enabled ? 'Disable' : 'Enable'} EventHook`}
        desc={
          `Are you sure you want to ${row.original.enabled ? 'disable' : 'enable'} this hook?` +
          (row.original.enabled ? ' This will prevent the event hook from being used.' : '')
        }
        destructive={row.original.enabled}
        confirmText={row.original.enabled ? 'Disable' : 'Enable'}
        isLoading={updateMutation.isPending}
        handleConfirm={handleConfirm}
      />
    </>
  )
}
