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

import { EmailTask, EventHookTask } from '../data/schema'
import AceEditor from '@/components/ace-editor'
import { useTheme } from '@/context/theme-context'


interface Props {
  currentRow: EmailTask | EventHookTask
  open: boolean
  onOpenChange: (open: boolean) => void
}


export function TaskDetailDialog({ currentRow, open, onOpenChange }: Props) {
  const { theme } = useTheme()

  return (
    <Dialog
      open={open}
      onOpenChange={(state) => {
        onOpenChange(state)
      }}
    >
      <DialogContent className='w-full md:max-w-2xl'>
        <DialogHeader className='text-left mb-4'>
          <DialogTitle>{currentRow.id}</DialogTitle>
        </DialogHeader>
        <AceEditor
          readOnly={true}
          value={JSON.stringify(currentRow, null, 2)}
          className="h-[35rem]"
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
