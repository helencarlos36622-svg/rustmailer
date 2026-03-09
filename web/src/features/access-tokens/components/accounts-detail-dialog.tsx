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
import { AccountsDetailTable } from './accounts-detail-table'
import { columns } from './accounts-detail-columns'

interface Props {
  currentRow: AccessToken
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function AccountDetailDialog({ currentRow, open, onOpenChange }: Props) {
  return (
    <Dialog
      open={open}
      onOpenChange={(state) => {
        onOpenChange(state)
      }}
    >
      <DialogContent className='sm:max-w-4xl'>
        <DialogHeader className='text-left'>
          <DialogTitle>Accounts</DialogTitle>
          <DialogDescription>
            The list of accounts that can be queried using an access token.
          </DialogDescription>
        </DialogHeader>
        <div className="h-[33rem] overflow-x-auto overflow-y-auto">
          <AccountsDetailTable data={currentRow.accounts} columns={columns} />
        </div>
        <DialogFooter>
          <DialogClose asChild>
            <Button variant='outline' className="px-2 py-1 text-sm h-auto">Close</Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
