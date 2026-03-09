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
import { OAuth2Entity } from '../data/schema'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { delete_oauth2 } from '@/api/oauth2/api'
import { ToastAction } from '@/components/ui/toast'
import { AxiosError } from 'axios'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  currentRow: OAuth2Entity
}

export function TokenDeleteDialog({ open, onOpenChange, currentRow }: Props) {
  const [value, setValue] = useState(0)
  const queryClient = useQueryClient();

  function handleSuccess() {
    toast({
      title: 'Delete Success',
      description: `Your OAuth2 application has been successfully deleted.`,
      action: <ToastAction altText="Close">Close</ToastAction>,
    });

    queryClient.invalidateQueries({ queryKey: ['oauth2-list'] });
    onOpenChange(false);
  }

  function handleError(error: AxiosError) {
    const errorMessage = error.response?.data ||
      error.message ||
      `Delete failed, please try again later`;

    toast({
      variant: "destructive",
      title: `OAuth2 delete Failed`,
      description: errorMessage as string,
      action: <ToastAction altText="Try again">Try again</ToastAction>,
    });
    console.error(error);
  }

  const deleteMutation = useMutation({
    mutationFn: (id: number) => delete_oauth2(id),
    onSuccess: handleSuccess,
    onError: handleError
  })

  const handleDelete = () => {
    if (value !== currentRow.id) return
    deleteMutation.mutate(currentRow.id)
  }

  return (
    <ConfirmDialog
      open={open}
      onOpenChange={onOpenChange}
      handleConfirm={handleDelete}
      disabled={value !== currentRow.id}
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
            <span className='font-bold'>{currentRow.id}</span>?
            <br />
            This action will permanently remove the oauth2 record from the system. This cannot be undone.
          </p>

          <Label className='my-2'>
            OAuth2:
            <Input
              type="number"
              value={value}
              onChange={(e) => setValue(parseInt(e.target.value, 10))}
              placeholder='Enter id to confirm deletion.'
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
