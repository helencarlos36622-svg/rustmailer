/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { CornerDownLeft, Mailbox } from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Skeleton } from '@/components/ui/skeleton';
import { TreeView } from '@/components/tree-view';
import { buildTree } from '../../../lib/build-tree';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { move_messages } from '@/api/mailbox/envelope/api';
import { useMutation } from '@tanstack/react-query';
import { toast } from '@/hooks/use-toast';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { MailboxData } from '@/api/mailbox/api';

interface MoveToProps {
    isMailboxesLoading: boolean;
    mailboxes?: MailboxData[];
    accountId: number,
    mailbox: string,
    selectedIds: string[],
    triggerUpdate: (mailbox: string) => void
    setSelectedIds: React.Dispatch<React.SetStateAction<string[]>>;
}

export function MoveTo({ isMailboxesLoading, mailboxes, accountId, mailbox, selectedIds, setSelectedIds, triggerUpdate }: MoveToProps) {
    const [open, setOpen] = useState(false);
    const moveMutation = useMutation({
        mutationFn: ({ accountId, payload }: { accountId: number, payload: Record<string, any> }) => move_messages(accountId, payload),
        retry: false,
        onSuccess: () => {
            triggerUpdate(mailbox);
            setOpen(false);
            setSelectedIds([])
            toast({
                title: 'Messages moved successfully',
                description: 'The messages have been moved.',
            });
        },
        onError: (error: any) => {
            toast({
                title: 'Failed to move messages',
                description: `${error.response.data.message}`,
                variant: 'destructive',
            });
        },
    });

    const handleMove = (target: string) => {
        if (target && mailbox && accountId) {
            const payload = {
                ids: selectedIds,
                current_mailbox: mailbox,
                target_mailbox: target
            }
            moveMutation.mutate({ accountId, payload })
        }
    }

    const action = (item: MailboxData) => {
        return (<TooltipProvider>
            <Tooltip>
                <TooltipTrigger asChild>
                    <CornerDownLeft className='h-4 w-4' onClick={(e) => {
                        e.stopPropagation();
                        handleMove(item.name)
                    }} />
                </TooltipTrigger>
                <TooltipContent>
                    <p>Click to move the envelope here</p>
                </TooltipContent>
            </Tooltip>
        </TooltipProvider>)
    }

    return (
        <Popover open={open} onOpenChange={setOpen}>
            <PopoverTrigger asChild>
                <Button variant="outline" className="flex items-center">
                    <Mailbox />
                    Move to
                </Button>
            </PopoverTrigger>
            <PopoverContent className="w-full md:w-96" align="start">
                <ScrollArea className="h-[20rem] w-full pr-4 -mr-4 py-1">
                    {isMailboxesLoading ? (
                        <div className="space-y-2 p-4">
                            {Array.from({ length: 5 }).map((_, index) => (
                                <div key={index} className="space-y-2">
                                    <div className="flex items-center space-x-2">
                                        <Skeleton className="h-4 w-4 rounded-full" />
                                        <Skeleton className="h-4 w-[200px]" />
                                    </div>
                                    <div className="pl-6 space-y-2">
                                        {Array.from({ length: 3 }).map((_, subIndex) => (
                                            <div key={subIndex} className="flex items-center space-x-2">
                                                <Skeleton className="h-4 w-4 rounded-full" />
                                                <Skeleton className="h-4 w-[150px]" />
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            ))}
                        </div>
                    ) : (
                        <TreeView data={buildTree(mailboxes ?? [], action)} />
                    )}
                </ScrollArea>
            </PopoverContent>
        </Popover>
    );
};
