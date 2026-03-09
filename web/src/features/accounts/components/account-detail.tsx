/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { AccountEntity, MailerType } from '../data/schema'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  currentRow: AccountEntity
}

export function AccountDetailDrawer({ open, onOpenChange, currentRow }: Props) {
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
    >
      <DialogContent className='max-w-5xl'>
        <DialogHeader className='text-left mb-4'>
          <DialogTitle>{currentRow.email}</DialogTitle>
          <DialogDescription>
          </DialogDescription>
        </DialogHeader>
        <ScrollArea className="h-[35rem] w-full pr-4 -mr-4 py-1">
          <Tabs defaultValue="account" className="w-full">
            <TabsList className="grid w-full grid-cols-3">
              <TabsTrigger value="account">Account Details</TabsTrigger>
              <TabsTrigger value="server">Server Configurations</TabsTrigger>
              <TabsTrigger value="folders">Sync Folders</TabsTrigger>
            </TabsList>

            {/* Account Information Tab */}
            <TabsContent value="account">
              <Card className="mt-4">
                {/* <CardHeader>
                  <CardTitle>Account Details</CardTitle>
                </CardHeader> */}
                <CardContent className='mt-4'>
                  <div className="flex flex-col gap-2">
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="text-muted-foreground">ID:</span>
                      <span>{currentRow.id}</span>
                    </div>
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="text-muted-foreground">Email:</span>
                      <span>{currentRow.email}</span>
                    </div>
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="text-muted-foreground">Name:</span>
                      <span>{currentRow.name ?? "n/a"}</span>
                    </div>
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="text-muted-foreground">Enabled:</span>
                      <Checkbox checked={currentRow.enabled} disabled />
                    </div>
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="text-muted-foreground">Mailer Type:</span>
                      <span>{currentRow.mailer_type}</span>
                    </div>
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="text-muted-foreground">Minimal Sync:</span>
                      {currentRow.minimal_sync !== undefined ? (
                        <Checkbox checked={currentRow.minimal_sync} disabled />
                      ) : (
                        <span className="text-muted-foreground">n/a</span>
                      )}
                    </div>
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="text-muted-foreground">Full Sync Interval:</span>
                      <span>{currentRow.full_sync_interval_min ? `every ${currentRow.full_sync_interval_min} min` : "n/a"}</span>
                    </div>
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="text-muted-foreground">Incremental Sync Interval:</span>
                      <span>every {currentRow.incremental_sync_interval_sec} sec</span>
                    </div>
                    <div className="flex flex-col gap-2">
                      <span className="text-muted-foreground">Capabilities:</span>
                      <code className="rounded-md bg-muted/50 px-2 py-1 text-sm border overflow-x-auto inline-block">
                        {currentRow.capabilities ? currentRow.capabilities.join(', ') : "n/a"}
                      </code>
                    </div>
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="text-muted-foreground">Date Selection:</span>
                      <span>
                        {currentRow.date_since?.fixed
                          ? currentRow.date_since.fixed
                          : currentRow.date_since?.relative
                            ? `recent ${currentRow.date_since.relative.value} ${currentRow.date_since.relative.unit}`
                            : 'n/a'}
                      </span>
                    </div>
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="text-muted-foreground">Folder Limit:</span>
                      <span>{currentRow.folder_limit ? currentRow.folder_limit : "n/a"}</span>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </TabsContent>

            {/* Server Configurations Tab */}
            <TabsContent value="server">
              {currentRow.mailer_type === MailerType.ImapSmtp ? <div className="grid gap-4 mt-4 md:grid-cols-2">
                {/* IMAP Configuration Card */}
                <Card>
                  <CardHeader>
                    <CardTitle>IMAP Configuration</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="flex flex-col gap-2">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="text-muted-foreground">Host:</span>
                        <span>{currentRow.imap?.host}</span>
                      </div>
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="text-muted-foreground">Port:</span>
                        <span>{currentRow.imap?.port}</span>
                      </div>
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="text-muted-foreground">Encryption:</span>
                        <span>{currentRow.imap?.encryption}</span>
                      </div>
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="text-muted-foreground">Auth:</span>
                        {currentRow.imap?.auth.auth_type === 'OAuth2' ? (
                          <Badge variant="outline" className="bg-blue-100 text-blue-800">
                            OAuth2
                          </Badge>
                        ) : (
                          <Badge variant="outline" className="bg-blue-100 text-blue-800">
                            Password
                          </Badge>
                        )}
                      </div>
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="text-muted-foreground">Use Proxy:</span>
                        <span>{currentRow.imap?.use_proxy ? "true" : "false"}</span>
                      </div>
                    </div>
                  </CardContent>
                </Card>

                {/* SMTP Configuration Card */}
                <Card>
                  <CardHeader>
                    <CardTitle>SMTP Configuration</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="flex flex-col gap-2">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="text-muted-foreground">Host:</span>
                        <span>{currentRow.smtp?.host}</span>
                      </div>
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="text-muted-foreground">Port:</span>
                        <span>{currentRow.smtp?.port}</span>
                      </div>
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="text-muted-foreground">Encryption:</span>
                        <span>{currentRow.smtp?.encryption}</span>
                      </div>
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="text-muted-foreground">Auth:</span>
                        {currentRow.smtp?.auth.auth_type === 'OAuth2' ? (
                          <Badge variant="outline" className="bg-blue-100 text-blue-800">
                            OAuth2
                          </Badge>
                        ) : (
                          <Badge variant="outline" className="bg-blue-100 text-blue-800">
                            Password
                          </Badge>
                        )}
                      </div>
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="text-muted-foreground">Use Proxy:</span>
                        <span>{currentRow.smtp?.use_proxy ? "true" : "false"}</span>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </div> : <div className="mt-48 text-center text-muted-foreground">
                No IMAP/SMTP configuration required (using {currentRow.mailer_type}).
              </div>
              }
            </TabsContent>

            {/* Sync Folders Tab */}
            <TabsContent value="folders">
              <Card className="mt-4">
                {/* <CardHeader>
                  <CardTitle>Sync Folders</CardTitle>
                </CardHeader> */}
                <CardContent>
                  {currentRow.sync_folders?.length ? (
                    <div className="space-y-2">
                      <div className="text-sm mt-4 text-muted-foreground">
                        {currentRow.sync_folders.length} folder(s) configured for sync
                      </div>
                      <ScrollArea className="h-[300px] rounded-md border">
                        <div className="p-2">
                          {currentRow.sync_folders.map((folder, index) => (
                            <div
                              key={index}
                              className="flex items-center py-2 px-3 hover:bg-accent rounded-md transition-colors"
                            >
                              <span className="text-sm font-medium">{folder}</span>
                            </div>
                          ))}
                        </div>
                      </ScrollArea>
                    </div>
                  ) : (
                    <div className="text-center py-8 text-muted-foreground">
                      No folders configured for sync
                    </div>
                  )}
                </CardContent>
              </Card>
            </TabsContent>
          </Tabs>
        </ScrollArea>
      </DialogContent>
    </Dialog>
  )
}