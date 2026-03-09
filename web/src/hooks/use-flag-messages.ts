/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { flag_messages } from '@/api/mailbox/envelope/api';
import { useMutation } from '@tanstack/react-query';
import { toast } from './use-toast';

// Define the custom hook
export function useFlagMessageMutation() {
    return useMutation({
        mutationFn: ({ accountId, payload }: { accountId: number; payload: Record<string, any> }) => flag_messages(accountId, payload),
        retry: 0,
        onSuccess: () => {
            toast({
                title: 'Message flagged successfully',
                description: 'The message has been flagged.',
            });
        },
        onError: (error: Error) => {
            toast({
                title: 'Failed to flag message',
                description: `${error.message}`,
                variant: 'destructive',
            });
        },
    });
}
