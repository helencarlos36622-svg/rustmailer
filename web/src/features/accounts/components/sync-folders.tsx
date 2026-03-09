/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription,
    DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'
import { useCallback, useMemo, useState } from 'react'
import { AccountEntity } from '../data/schema'
import { toast } from '@/hooks/use-toast'
import { list_account_mailboxes } from '@/api/mailbox/api'
import { buildTree } from '@/lib/build-tree'
import { TreeDataItem, TreeView } from '@/components/tree-view'
import { Skeleton } from '@/components/ui/skeleton'
import { update_account } from '@/api/account/api'
import { ToastAction } from '@/components/ui/toast'
import { AxiosError } from 'axios'
import { ScrollArea } from '@/components/ui/scroll-area'

interface Props {
    open: boolean
    onOpenChange: (open: boolean) => void
    currentRow: AccountEntity
}

export function SyncFoldersDialog({ currentRow, open, onOpenChange }: Props) {
    const [selectedFolders, setSelectedFolders] = useState<string[]>(currentRow.sync_folders || []);
    const [isSubmitting, setIsSubmitting] = useState(false);
    const queryClient = useQueryClient();
    const { data: mailboxes, isLoading } = useQuery({
        queryKey: ['account-mailboxes', currentRow.id],
        queryFn: () => list_account_mailboxes(currentRow.id, true),
        enabled: open,
    });

    // Convert mailbox names to IDs for initial selection
    const initialSelectedItemIds = useMemo(() => {
        if (!mailboxes) return [];
        return mailboxes
            .filter(mailbox => selectedFolders.includes(mailbox.name))
            .map(mailbox => mailbox.id.toString());
    }, [mailboxes, selectedFolders]);

    // Convert data to tree structure
    const treeData = useMemo(() => {
        if (!mailboxes) return [];
        return buildTree(mailboxes, undefined, true);
    }, [mailboxes]);

    const handleSelectItems = useCallback((selectedItems: TreeDataItem[]) => {
        const selected = selectedItems
            .map(item => mailboxes?.find(m => m.id === parseInt(item.id, 10))?.name)
            .filter(Boolean) as string[];

        setSelectedFolders(selected);
    }, [mailboxes]);


    const updateMutation = useMutation({
        mutationFn: (data: Record<string, any>) => update_account(currentRow?.id ?? '', data),
        onSuccess: handleSuccess,
        onError: handleError
    })

    function handleSuccess() {
        toast({
            title: 'Account Sync Folders Updated',
            description: 'Account has been successfully updated.',
            action: <ToastAction altText="Close">Close</ToastAction>,
        });

        queryClient.invalidateQueries({ queryKey: ['account-list'] });
        setIsSubmitting(false);
        onOpenChange(false);
    }
    function handleError(error: AxiosError) {
        const errorMessage = (error.response?.data as { message?: string })?.message ||
            error.message ||
            'Update failed, please try again later';

        toast({
            variant: "destructive",
            title: 'Account Sync Folders Update Failed',
            description: errorMessage as string,
            action: <ToastAction altText="Try again">Try again</ToastAction>,
        });
        setIsSubmitting(false);
        console.error(error);
    }


    const handleSubmit = async () => {
        if (selectedFolders.length === 0) {
            toast({
                title: 'Error',
                description: 'Please select at least one folder',
                variant: 'destructive',
            });
            return;
        }
        setIsSubmitting(true);
        updateMutation.mutate({
            sync_folders: selectedFolders,
        });
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-2xl">
                <DialogHeader>
                    <DialogTitle>Select Sync Folders</DialogTitle>
                    <DialogDescription>
                        Choose folders to sync for {currentRow.email}
                    </DialogDescription>
                </DialogHeader>

                <div className="space-y-4">
                    <div className="flex items-center justify-end">
                        <div className="text-sm text-muted-foreground">
                            {selectedFolders.length} folder(s) selected
                        </div>
                    </div>
                    <ScrollArea className="h-[30rem] w-full pr-4 -mr-4 py-1">
                        {isLoading && (
                            <div className="p-8 space-y-8">
                                <div className="flex flex-col items-center gap-3 text-muted-foreground">
                                    <Loader2 className="h-6 w-6 animate-spin" />
                                    <span className="text-sm font-medium">Loading mailbox folders…</span>
                                </div>

                                <div className="space-y-2">
                                    {[...Array(8)].map((_, i) => (
                                        <Skeleton key={i} className="h-8 w-full" />
                                    ))}
                                </div>
                            </div>
                        )}
                        {!isLoading && (
                            <TreeView
                                data={treeData}
                                multiple
                                expandAll
                                clickRowToSelect={false}
                                initialSelectedItemIds={initialSelectedItemIds}
                                onSelectItemsChange={handleSelectItems}
                            />
                        )}
                    </ScrollArea>
                </div>

                <DialogFooter>
                    <Button
                        variant="outline"
                        onClick={() => onOpenChange(false)}
                        disabled={isSubmitting}
                    >
                        Cancel
                    </Button>
                    <Button
                        onClick={handleSubmit}
                        disabled={isSubmitting || isLoading}
                    >
                        {isSubmitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                        Save Changes
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}