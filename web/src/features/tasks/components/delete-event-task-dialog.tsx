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
import { EventHookTask } from '../data/schema'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { delete_hook_task } from '@/api/tasks/api'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  currentRow: EventHookTask
}

export function EventTaskRemoveDialog({ open, onOpenChange, currentRow }: Props) {
  const [value, setValue] = useState(0)
  const queryClient = useQueryClient();
  const deleteMutation = useMutation({
    mutationFn: () => delete_hook_task(currentRow?.id!),
    retry: 0,
    onSuccess: () => {
      toast({
        title: 'Task successfully removed',
        description: 'The task has been removed.',
      });
      queryClient.invalidateQueries({ queryKey: ['hook-tasks'] });
      onOpenChange(false);
    },
    onError: (error: Error) => {
      toast({
        title: 'Failed to delete task',
        description: `${error.message}`,
        variant: 'destructive',
      });
    },
  });

  const handleDelete = () => {
    if (value !== currentRow.id) return
    deleteMutation.mutate();
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
          Remove Event Hook Task
        </span>
      }
      desc={
        <div className='space-y-4'>
          <p className='mb-2'>
            Are you sure you want to remove the event hook task{' '}
            <span className='font-bold'>{`${currentRow.id}`}</span> from the task queue?
            <br />
            This action will permanently remove the scheduled event hook task from the system. This cannot be undone.
          </p>

          <Label className='my-2'>
            Task ID:
            <Input
              type='number'
              value={`${value}`}
              onChange={(e) => setValue(parseInt(e.target.value, 10))}
              placeholder='Enter the task ID to confirm remove.'
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
      confirmText='Remove'
      destructive
    />
  )
}
