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
} from '@/components/ui/dialog'

import { EnvelopeFilter, FilterForm } from './envelope-filter'
import { DialogDescription } from '@radix-ui/react-dialog'
import { ScrollArea } from '@/components/ui/scroll-area'


interface Props {
  handleSearch: (data: FilterForm) => Promise<void>,
  remote: boolean,
  isSearching: boolean,
  open: boolean,
  currentFilter?: FilterForm,
  onOpenChange: (open: boolean) => void
}

export function EnvelopeFilterDialog({ remote, currentFilter, handleSearch, isSearching, open, onOpenChange }: Props) {

  return (
    <Dialog
      open={open}
      onOpenChange={(state) => {
        onOpenChange(state)
      }}
    >
      <DialogContent className='w-full md:max-w-4xl'>
        <DialogHeader className='text-left mb-4'>
          <DialogTitle>Filters</DialogTitle>
          <DialogDescription>Add filters to refine your results.</DialogDescription>
        </DialogHeader>
        <ScrollArea className="max-h-[30rem] w-full pr-4 -mr-4 py-1">
          <EnvelopeFilter handleSearch={handleSearch} remote={remote} isSearching={isSearching} currentFilter={currentFilter} />
        </ScrollArea>
      </DialogContent>
    </Dialog>
  )
}
