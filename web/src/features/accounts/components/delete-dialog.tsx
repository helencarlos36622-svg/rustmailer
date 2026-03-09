/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { useState } from 'react'
import { IconAlertCircle, IconAlertTriangle } from '@tabler/icons-react'
import { toast } from '@/hooks/use-toast'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { ConfirmDialog } from '@/components/confirm-dialog'
import { AccountEntity } from '../data/schema'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { ToastAction } from '@/components/ui/toast'
import { AxiosError } from 'axios'
import { remove_account } from '@/api/account/api'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  currentRow: AccountEntity
}

export function AccountDeleteDialog({ open, onOpenChange, currentRow }: Props) {
  const [value, setValue] = useState('')

  const queryClient = useQueryClient();
  function handleSuccess() {
    toast({
      title: 'Account Deleted',
      description: 'Your account has been successfully deleted.',
      action: <ToastAction altText="Close">Close</ToastAction>,
    });

    queryClient.invalidateQueries({ queryKey: ['account-list'] });
    onOpenChange(false);
  } 

  function handleError(error: AxiosError) {
    const errorMessage = error.response?.data ||
      error.message ||
      `Delete failed, please try again later`;

    toast({
      variant: "destructive",
      title: `Account delete Failed`,
      description: errorMessage as string,
      action: <ToastAction altText="Try again">Try again</ToastAction>,
    });
    console.error(error);
  }

  const deleteMutation = useMutation({
    mutationFn: (id: number) => remove_account(id),
    onSuccess: handleSuccess,
    onError: handleError
  })

  const handleDelete = () => {
    if (value.trim() !== currentRow.email) return
    deleteMutation.mutate(currentRow.id)
  }

  return (
    <ConfirmDialog
      open={open}
      onOpenChange={onOpenChange}
      handleConfirm={handleDelete}
      disabled={value.trim() !== currentRow.email}
      className="max-w-2xl"
      title={
        <span className='text-destructive'>
          <IconAlertTriangle
            className='mr-1 inline-block stroke-destructive'
            size={18}
          />{' '}
          Delete Account Permanently
        </span>
      }
      desc={
        <div className='space-y-4'>
          <p className='mb-2'>
            You are deleting <span className='font-bold'>{currentRow.email}</span>.
            This will permanently remove:
          </p>

          <ul className="list-disc pl-6 space-y-1 text-sm text-muted-foreground">
            <li>Account credentials and settings</li>
            <li>All email templates associated with this account</li>
            <li>Event hooks and webhook configurations</li>
            <li>Local cached metadata and sync status</li>
            <li>IMAP synchronization data</li>
            <li>OAuth tokens and API credentials</li>
          </ul>

          <div className="pt-2">
            <Label>
              Type the account email to confirm:
              <Input
                value={value}
                onChange={(e) => setValue(e.target.value)}
                placeholder={`Type "${currentRow.email}" to confirm`}
                className="mt-2"
              />
            </Label>
          </div>

          <Alert variant='destructive'>
            <IconAlertCircle className="h-4 w-4" />
            <AlertTitle>This action cannot be undone!</AlertTitle>
            <AlertDescription>
              All related resources will be permanently erased.
            </AlertDescription>
          </Alert>
        </div>
      }
      confirmText={
        deleteMutation.isPending ? 'Deleting...' : 'Permanently Delete Account'
      }
      isLoading={deleteMutation.isPending}
      destructive
    />
  )
}
