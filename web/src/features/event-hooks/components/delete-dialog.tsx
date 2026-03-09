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
import { EventHook } from '../data/schema'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { delete_event_hook } from '@/api/hook/api'

interface Props {
    open: boolean
    onOpenChange: (open: boolean) => void
    currentRow: EventHook
}

export function EventHookDeleteDialog({ open, onOpenChange, currentRow }: Props) {
    const [value, setValue] = useState(0)
    const queryClient = useQueryClient();
    const deleteMutation = useMutation({
        mutationFn: () => delete_event_hook(currentRow?.id),
        retry: 0,
        onSuccess: () => {
            toast({
                title: 'Eventhook delete successfully',
                description: 'The event hook has been deleted.',
            });
            queryClient.invalidateQueries({ queryKey: ['event-hook-list'] });
            onOpenChange(false);
        },
        onError: (error: Error) => {
            toast({
                title: 'Failed to delete eventhook',
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
                    Delete Event hook
                </span>
            }
            desc={
                <div className='space-y-4'>
                    <p className='mb-2'>
                        Are you sure you want to delete{' '}
                        <span className='font-bold'>{`${currentRow.id}`}</span>?
                        <br />
                        This action will permanently remove the event hook from the system. This cannot be undone.
                    </p>

                    <Label className='my-2'>
                        Event Hook Id:
                        <Input
                            type="number"
                            value={`${value}`}
                            onChange={(e) => setValue(parseInt(e.target.value, 10))}
                            placeholder='Enter Event Hook Id to confirm deletion.'
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
