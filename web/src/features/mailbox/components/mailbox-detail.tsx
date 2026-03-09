/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'

import AceEditor from '@/components/ace-editor'
import { useTheme } from '@/context/theme-context'
import { MailboxData } from '@/api/mailbox/api'


interface Props {
  currentMailbox?: MailboxData
  open: boolean
  onOpenChange: (open: boolean) => void
}

function convertMailboxData(raw: MailboxData): any {
  const flags: string[] = [];

  raw.flags.forEach(item => {
    flags.push(item.flag);
    if (item.flag.toLowerCase() === "Custom" && item.custom !== null) {
      flags.push(item.custom);
    }
  });

  const attributes: string[] = [];
  raw.attributes.forEach(item => {
    attributes.push(item.attr);
    if (item.attr.toLowerCase() === "Extension" && item.extension !== null) {
      attributes.push(item.extension);
    }
  });

  return {
    // ...raw,
    id: raw.id.toString(),
    flags,
    attributes
  };
}


export function MailboxDialog({ currentMailbox, open, onOpenChange }: Props) {
  const { theme } = useTheme()

  return (
    <Dialog
      open={open}
      onOpenChange={(state) => {
        onOpenChange(state)
      }}
    >
      <DialogContent className='w-full md:max-w-xl'>
        <DialogHeader className='text-left mb-4'>
          <DialogTitle>{currentMailbox?.name}</DialogTitle>
        </DialogHeader>
        <AceEditor
          readOnly={true}
          value={currentMailbox
            ? JSON.stringify(convertMailboxData(currentMailbox), null, 2)
            : 'null'}
          className="h-[27rem]"
          mode='json'
          theme={theme === "dark" ? 'monokai' : 'kuroir'}
        />
        <DialogFooter>
          <DialogClose asChild>
            <Button variant='outline' className="px-2 py-1 text-sm h-auto">Close</Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
