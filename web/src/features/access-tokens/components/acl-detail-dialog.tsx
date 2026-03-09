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
import { AccessToken } from '../data/schema'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Textarea } from '@/components/ui/textarea'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'

interface Props {
  currentRow: AccessToken
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function AclDetailDialog({ currentRow, open, onOpenChange }: Props) {
  return (
    <Dialog
      open={open}
      onOpenChange={(state) => {
        onOpenChange(state)
      }}
    >
      <DialogContent className='sm:max-w-xl'>
        <DialogHeader className='text-left'>
          <DialogTitle>Acl</DialogTitle>
          <DialogDescription>
            ACL rules for access tokens include IP whitelist verification and rate limit enforcement.
          </DialogDescription>
        </DialogHeader>
        <ScrollArea className='h-[33rem] w-full pr-4 -mr-4 py-1'>
          <div className="space-y-4">
            {/* IP Whitelist */}
            <div className="grid w-full items-center">
              <Label className="mb-2">IP Whitelist</Label>
              <Textarea
                className="col-span-5 max-h-[240px] min-h-[300px]"
                value={currentRow.acl?.ip_whitelist?.join('\n')}
              />
            </div>

            {/* Quota */}
            <div className="grid w-full items-center">
              <Label className="mb-2">Quota</Label>
              <Input
                type="number"
                value={currentRow.acl?.rate_limit?.quota}
                className="col-span-5"
              />
            </div>

            {/* Interval (seconds) */}
            <div className="grid w-full items-center">
              <Label className="mb-2">Interval (seconds)</Label>
              <Input
                type="number"
                className="col-span-5"
                value={currentRow.acl?.rate_limit?.interval}
              />
            </div>
          </div>
        </ScrollArea>
        <DialogFooter>
          <DialogClose asChild>
            <Button variant='outline' className="px-2 py-1 text-sm h-auto">Close</Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
