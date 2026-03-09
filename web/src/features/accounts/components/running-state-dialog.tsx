/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { AccountEntity, MailerType } from '../data/schema'
import { useQuery } from '@tanstack/react-query'
import { account_state } from '@/api/account/api'
import { formatDistanceToNow, formatDuration, intervalToDuration } from 'date-fns'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Skeleton } from '@/components/ui/skeleton'
import { CheckCircle, Clock, Loader2, PlayCircle, FolderSync, FolderCheck } from 'lucide-react'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  currentRow: AccountEntity
}

export function RunningStateDialog({ currentRow, open, onOpenChange }: Props) {
  const { data: state, isLoading } = useQuery({
    queryKey: ['running-state', currentRow.id],
    queryFn: () => account_state(currentRow.id),
    retry: 0,
    refetchOnWindowFocus: false,
    refetchInterval: 5000,
  })

  // Helper function to calculate duration
  const calculateDuration = (start?: number, end?: number) => {
    if (!start) {
      return (
        <span className="text-yellow-600 flex items-center gap-1">
          <Clock className="w-4 h-4" /> Not Started
        </span>
      );
    }
    if (!end) {
      return (
        <span className="text-blue-600 flex items-center gap-1">
          <PlayCircle className="w-4 h-4" /> In Progress
        </span>
      );
    }
    const duration = intervalToDuration({ start: new Date(start), end: new Date(end) })
    return (
      <span className="text-green-600 flex items-center gap-1">
        <CheckCircle className="w-4 h-4" /> {formatDuration(duration, { format: ['hours', 'minutes', 'seconds'] })}
      </span>
    );
  }

  // Helper function to render sync progress
  const renderSyncProgress = (current?: number | null, total?: number | null) => {
    if (current === null || current === undefined) {
      return <span className="text-muted-foreground">n/a</span>;
    }

    if (total === null || total === undefined) {
      return (
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium">
            Batch {current}
          </span>
        </div>
      );
    }

    const percentage = total > 0 ? Math.round((current / total) * 100) : 0;
    return (
      <div className="flex items-center gap-2">
        <span className="text-sm font-medium">
          {current}/{total} ({percentage}%)
        </span>
      </div>
    );
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-7xl">
        <DialogHeader className="text-left">
          <DialogTitle>
            <span className="ml-2 text-blue-500 font-medium">{currentRow.email}</span>
          </DialogTitle>
          <DialogDescription>
            Account synchronization status and detailed information
          </DialogDescription>
        </DialogHeader>

        {/* Loading State */}
        {isLoading && (
          <div className="space-y-4">
            <Skeleton className="h-6 w-1/2" />
            <Skeleton className="h-6 w-1/3" />
            <div className="flex justify-center items-center py-4">
              <Loader2 className="h-8 w-8 animate-spin text-primary" />
            </div>
          </div>
        )}

        {/* Loaded State */}
        {!isLoading && state && (
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Left Column - Sync Status */}
            <div className="lg:col-span-2 space-y-4">
              {/* Initial Sync Card */}
              <div className="p-4 border rounded-lg">
                <div className="flex items-center justify-between mb-3">
                  <h3 className="text-lg font-semibold flex items-center gap-2">
                    {state.is_initial_sync_completed ? (
                      <FolderCheck className="w-5 h-5 text-green-500" />
                    ) : (
                      <FolderSync className="w-5 h-5 text-blue-500" />
                    )}
                    Initial Sync
                  </h3>
                  {state.is_initial_sync_completed ? (
                    <span className="text-xs bg-green-100 text-green-800 px-2 py-1 rounded-full">
                      Completed
                    </span>
                  ) : (
                    <span className="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded-full">
                      In Progress
                    </span>
                  )}
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="space-y-3">
                    <div className="flex justify-between">
                      <span className="text-sm text-muted-foreground">Start Time:</span>
                      <span className="text-sm font-medium">
                        {state.initial_sync_start_time
                          ? formatDistanceToNow(new Date(state.initial_sync_start_time), { addSuffix: true })
                          : <span className="text-yellow-600">Not Started</span>}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm text-muted-foreground">End Time:</span>
                      <span className="text-sm font-medium">
                        {state.initial_sync_end_time
                          ? formatDistanceToNow(new Date(state.initial_sync_end_time), { addSuffix: true })
                          : state.initial_sync_start_time
                            ? <span className="text-blue-600">In Progress</span>
                            : <span className="text-yellow-600">Not Started</span>}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm text-muted-foreground">Duration:</span>
                      <span className="text-sm font-medium">
                        {calculateDuration(state.initial_sync_start_time, state.initial_sync_end_time)}
                      </span>
                    </div>
                  </div>

                  <div className="space-y-3">
                    <div className="flex justify-between">
                      <span className="text-sm text-muted-foreground">Current Folder:</span>
                      <span className="text-sm font-medium">
                        {state.current_syncing_folder || <span className="text-muted-foreground">n/a</span>}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm text-muted-foreground">Batch Progress:</span>
                      <span className="text-sm font-medium">
                        {renderSyncProgress(state.current_batch_number, state.current_total_batches)}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm text-muted-foreground">Folders to Sync:</span>
                      <span className="text-sm font-medium">
                        {state.initial_sync_folders?.length || 0} folders
                      </span>
                    </div>
                  </div>
                </div>

                {/* Folders List */}
                {state.initial_sync_folders?.length > 0 && (
                  <div className="mt-4">
                    <h4 className="text-sm font-medium mb-2 text-muted-foreground">Folders List:</h4>
                    <ScrollArea className="h-24 border rounded-md p-2">
                      <code className="rounded-md bg-muted/50 px-2 py-1 text-sm border overflow-x-auto inline-block">
                        {state.initial_sync_folders.join(', ')}
                      </code>
                    </ScrollArea>
                  </div>
                )}
              </div>
              {currentRow.mailer_type === MailerType.ImapSmtp && <div className="p-4 border rounded-lg">
                <h3 className="text-lg font-semibold mb-3">Full Sync</h3>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="space-y-3">
                    <div className="flex justify-between">
                      <span className="text-sm text-muted-foreground">Start Time:</span>
                      <span className="text-sm font-medium">
                        {state.last_full_sync_start
                          ? formatDistanceToNow(new Date(state.last_full_sync_start), { addSuffix: true })
                          : <span className="text-yellow-600">Not Started</span>}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm text-muted-foreground">End Time:</span>
                      <span className="text-sm font-medium">
                        {state.last_full_sync_end
                          ? formatDistanceToNow(new Date(state.last_full_sync_end), { addSuffix: true })
                          : state.last_full_sync_start
                            ? <span className="text-blue-600">In Progress</span>
                            : <span className="text-yellow-600">Not Started</span>}
                      </span>
                    </div>
                  </div>
                  <div className="space-y-3">
                    <div className="flex justify-between">
                      <span className="text-sm text-muted-foreground">Duration:</span>
                      <span className="text-sm font-medium">
                        {calculateDuration(state.last_full_sync_start, state.last_full_sync_end)}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm text-muted-foreground">Sync Interval:</span>
                      <span className="text-sm font-medium">
                        Every {currentRow.full_sync_interval_min} minutes
                      </span>
                    </div>
                  </div>
                </div>
              </div>}
              <div className="p-4 border rounded-lg">
                <h3 className="text-lg font-semibold mb-3">Incremental Sync</h3>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="space-y-3">
                    <div className="flex justify-between">
                      <span className="text-sm text-muted-foreground">Start Time:</span>
                      <span className="text-sm font-medium">
                        {state.last_incremental_sync_start
                          ? formatDistanceToNow(new Date(state.last_incremental_sync_start), { addSuffix: true })
                          : <span className="text-yellow-600">Not Started</span>}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm text-muted-foreground">End Time:</span>
                      <span className="text-sm font-medium">
                        {state.last_incremental_sync_end
                          ? formatDistanceToNow(new Date(state.last_incremental_sync_end), { addSuffix: true })
                          : state.last_incremental_sync_start
                            ? <span className="text-blue-600">In Progress</span>
                            : <span className="text-yellow-600">Not Started</span>}
                      </span>
                    </div>
                  </div>
                  <div className="space-y-3">
                    <div className="flex justify-between">
                      <span className="text-sm text-muted-foreground">Duration:</span>
                      <span className="text-sm font-medium">
                        {calculateDuration(state.last_incremental_sync_start, state.last_incremental_sync_end)}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm text-muted-foreground">Sync Interval:</span>
                      <span className="text-sm font-medium">
                        Every {currentRow.incremental_sync_interval_sec} seconds
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            <div className="p-4 border rounded-lg">
              <h3 className="text-lg font-semibold mb-3">Error Logs</h3>
              <ScrollArea className="h-[32rem]">
                <div className="space-y-3">
                  {state.errors.length ? (
                    state.errors.sort((a, b) => b.at - a.at).map((item, index) => (
                      <div
                        key={index}
                        className="flex flex-col items-start gap-2 rounded-lg border p-3 text-left text-sm transition-all hover:bg-accent"
                      >
                        <div className="flex w-full flex-col gap-1">
                          <div className="flex items-center justify-between">
                            <div className="text-xs font-medium text-muted-foreground">
                              {formatDistanceToNow(new Date(item.at), { addSuffix: true })}
                            </div>
                          </div>
                          <div className="text-xs font-medium break-words" style={{ wordBreak: 'break-word' }}>
                            {item.error}
                          </div>
                        </div>
                      </div>
                    ))
                  ) : (
                    <div className="h-full flex justify-center items-center py-8">
                      <p className="text-sm text-muted-foreground">No error logs available.</p>
                    </div>
                  )}
                </div>
              </ScrollArea>
            </div>
          </div>
        )}

        {!isLoading && !state && (
          <div className="h-full flex flex-col justify-center items-center py-8">
            <p className="text-sm text-muted-foreground">No active state available</p>
            <p className="text-xs text-muted-foreground">Account synchronization may not have started yet</p>
          </div>
        )}

        <DialogFooter>
          <DialogClose asChild>
            <Button variant="outline">Close</Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}