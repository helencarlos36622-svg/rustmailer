/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { useState } from 'react'
import { IconPlus } from '@tabler/icons-react'
import useDialogState from '@/hooks/use-dialog-state'
import { Button } from '@/components/ui/button'
import { Main } from '@/components/layout/main'
import { columns } from './components/columns'
import { EventHookTable } from './components/data-table'
import { EventHooksMutateDrawer } from './components/mutate-drawer'
import EventHooksContextProvider, { EventHooksDialogType } from './context/tasks-context'
import { EventHook } from './data/schema'
import { FixedHeader } from '@/components/layout/fixed-header'
import { EventHookDeleteDialog } from './components/delete-dialog'


export default function EventHooks() {
  // Local states
  const [currentRow, setCurrentRow] = useState<EventHook | null>(null)
  const [open, setOpen] = useDialogState<EventHooksDialogType>(null)

  return (
    <EventHooksContextProvider value={{ open, setOpen, currentRow, setCurrentRow }}>
      {/* ===== Top Heading ===== */}
      <FixedHeader />

      <Main>
        <div className='mb-2 flex items-center justify-between space-y-2 flex-wrap gap-x-4'>
          <div>
            <h2 className='text-2xl font-bold tracking-tight'>EventHooks</h2>
            <p className='text-muted-foreground'>
              Delivers IMAP/SMTP events to webhooks or message queues (Nats)
            </p>
          </div>
          <Button className='space-x-1' onClick={() => setOpen('create')}>
            <span>Add</span> <IconPlus size={18} />
          </Button>
        </div>
        <div className='-mx-4 flex-1 overflow-auto px-4 py-1 lg:flex-row lg:space-x-12 lg:space-y-0'>
          <EventHookTable columns={columns} onCallback={() => setOpen('create')} />
        </div>
      </Main>

      <EventHooksMutateDrawer
        key='hook-create'
        open={open === 'create'}
        onOpenChange={() => setOpen('create')}
      />

      {currentRow && (
        <>
          <EventHooksMutateDrawer
            key={`hook-update-${currentRow.id}`}
            open={open === 'update'}
            onOpenChange={() => {
              setOpen('update')
              setTimeout(() => {
                setCurrentRow(null)
              }, 500)
            }}
            currentRow={currentRow}
          />

          <EventHookDeleteDialog
            key={`eventhook-delete-${currentRow.id}`}
            open={open === 'delete'}
            onOpenChange={() => {
              setOpen('delete')
              setTimeout(() => {
                setCurrentRow(null)
              }, 500)
            }}
            currentRow={currentRow}
          />
        </>
      )}
    </EventHooksContextProvider>
  )
}
