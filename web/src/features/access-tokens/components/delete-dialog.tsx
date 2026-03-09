/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { useState } from 'react'
import { IconAlertTriangle } from '@tabler/icons-react'
import { toast } from '@/hooks/use-toast'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { ConfirmDialog } from '@/components/confirm-dialog'
import { AccessToken } from '../data/schema'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { delete_access_token } from '@/api/access-tokens/api'
import { ToastAction } from '@/components/ui/toast'
import { AxiosError } from 'axios'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  currentRow: AccessToken
}

export function TokenDeleteDialog({ open, onOpenChange, currentRow }: Props) {
  const [value, setValue] = useState('')
  const queryClient = useQueryClient();

  const deleteMutation = useMutation({
    mutationFn: () => delete_access_token(currentRow.token),
    onSuccess: handleSuccess,
    onError: handleError
  });


  function handleSuccess() {
    toast({
      title: 'Access Token Deleted',
      description: 'Your access token has been successfully deleted.',
      action: <ToastAction altText="Close">Close</ToastAction>,
    });

    queryClient.invalidateQueries({ queryKey: ['access-tokens'] });
    onOpenChange(false);
  }

  function handleError(error: AxiosError) {
    const errorMessage = (error.response?.data as { message?: string })?.message ||
      error.message ||
      'Delete failed, please try again later';

    toast({
      variant: "destructive",
      title: `Access token delete Failed`,
      description: errorMessage as string,
      action: <ToastAction altText="Try again">Try again</ToastAction>,
    });
    console.error(error);
  }


  const handleDelete = () => {
    if (value.trim() !== currentRow.token) return
    deleteMutation.mutate()
  }

  return (
    <ConfirmDialog
      open={open}
      onOpenChange={onOpenChange}
      handleConfirm={handleDelete}
      disabled={value.trim() !== currentRow.token}
      className="max-w-2xl"
      title={
        <span className='text-destructive'>
          <IconAlertTriangle
            className='mr-1 inline-block stroke-destructive'
            size={18}
          />{' '}
          Delete Token
        </span>
      }
      desc={
        <div className='space-y-4'>
          <p className='mb-2'>
            Are you sure you want to delete{' '}
            <span className='font-bold'>{currentRow.token}</span>?
            <br />
            This action will permanently remove the token from the system. This cannot be undone.
          </p>

          <Label className='my-2'>
            Token:
            <Input
              value={value}
              onChange={(e) => setValue(e.target.value)}
              placeholder='Enter token to confirm deletion.'
              className="mt-2"
            />
          </Label>

          <Alert variant='destructive'>
            <AlertTitle>Warning!</AlertTitle>
            <AlertDescription>
              Please be carefull, this operation can not be rolled back.
            </AlertDescription>
          </Alert>
        </div>
      }
      confirmText='Delete'
      destructive
    />
  )
}
