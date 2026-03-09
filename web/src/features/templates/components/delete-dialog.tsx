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
import { EmailTemplate } from '../data/schema'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { delete_template } from '@/api/templates/api'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  currentRow: EmailTemplate
}

export function TemplateDeleteDialog({ open, onOpenChange, currentRow }: Props) {
  const [value, setValue] = useState(0)
  const queryClient = useQueryClient();
  const deleteTemplateMutation = useMutation({
    mutationFn: () => delete_template(currentRow?.id!),
    retry: 0,
    onSuccess: () => {
      toast({
        title: 'Template delete successfully',
        description: 'The email template has been deleted.',
      });
      queryClient.invalidateQueries({ queryKey: ['email-templates-list'] });
      onOpenChange(false);
    },
    onError: (error: Error) => {
      toast({
        title: 'Failed to delete template',
        description: `${error.message}`,
        variant: 'destructive',
      });
    },
  });

  const handleDelete = () => {
    if (value !== currentRow.id) return
    deleteTemplateMutation.mutate();
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
          Delete Template
        </span>
      }
      desc={
        <div className='space-y-4'>
          <p className='mb-2'>
            Are you sure you want to delete{' '}
            <span className='font-bold'>{`${currentRow.id}`}</span>?
            <br />
            This action will permanently remove the template from the system. This cannot be undone.
          </p>

          <Label className='my-2'>
            Template:
            <Input
              type="number"
              value={`${value}`}
              onChange={(e) => setValue(parseInt(e.target.value, 10))}
              placeholder='Enter template id to confirm deletion.'
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
