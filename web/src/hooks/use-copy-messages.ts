/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { copy_messages } from '@/api/mailbox/envelope/api';
import { useMutation } from '@tanstack/react-query';
import { toast } from './use-toast';

// Define the custom hook
export function useCopyMessageMutation() {
    return useMutation({
        mutationFn: ({ accountId, payload }: { accountId: number; payload: Record<string, any> }) => copy_messages(accountId, payload),
        retry: 0,
        onSuccess: () => {
            toast({
                title: 'Message copied successfully',
                description: 'The message has been copied.',
            });
        },
        onError: (error: Error) => {
            toast({
                title: 'Failed to copy message',
                description: `${error.message}`,
                variant: 'destructive',
            });
        },
    });
}
