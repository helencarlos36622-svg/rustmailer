/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { useState } from 'react'
import useDialogState from '@/hooks/use-dialog-state'
import { Button } from '@/components/ui/button'
import { Main } from '@/components/layout/main'
import { MTAActionDialog } from './components/action-dialog'
import { columns } from './components/columns'
import { MTADeleteDialog } from './components/delete-dialog'
import { MTATable } from './components/table'
import MTAProvider, {
  type MTADialogType,
} from './context'
import { Plus } from 'lucide-react'
import { MTARecord } from './data/schema'
import { SentTestEmailDialog } from './components/send-email-test-dialog'
import { useQuery } from '@tanstack/react-query'
import { list_mta } from '@/api/mta/api'
import { TableSkeleton } from '@/components/table-skeleton'
import { FixedHeader } from '@/components/layout/fixed-header'
import Logo from '@/assets/logo.svg'

export default function MTA() {
  // Dialog states
  const [currentRow, setCurrentRow] = useState<MTARecord | null>(null)
  const [open, setOpen] = useDialogState<MTADialogType>(null)

  const { data: mtaList, isLoading } = useQuery({
    queryKey: ['MTA-list'],
    queryFn: list_mta,
  })

  return (
    <MTAProvider value={{ open, setOpen, currentRow, setCurrentRow }}>
      {/* ===== Top Heading ===== */}
      <FixedHeader />

      <Main>
        <div className='mb-2 flex items-center justify-between space-y-2 flex-wrap gap-x-4'>
          <div>
            <h2 className='text-2xl font-bold tracking-tight'>MTA</h2>
            <p className='text-muted-foreground'>
              MTAs (Mail Transfer Agents) act as independent mail relays, not tied to specific email accounts. They can be hosted locally or through services like Amazon SES, SendGrid, or Mailgun.
            </p>
          </div>
          <div className='flex gap-2'>
            <Button className='space-x-1' onClick={() => setOpen('add')}>
              <span>Add</span> <Plus size={18} />
            </Button>
          </div>
        </div>
        <div className='-mx-4 flex-1 overflow-auto px-4 py-1 flex-row lg:space-x-12 space-y-0'>
          {isLoading ? <TableSkeleton columns={columns.length} rows={10} /> : mtaList?.items.length ? (
            <MTATable data={mtaList.items} columns={columns} />
          ) : (
            <div className="flex h-[450px] shrink-0 items-center justify-center rounded-md border border-dashed">
              <div className="mx-auto flex max-w-[420px] flex-col items-center justify-center text-center">
                <img
                  src={Logo}
                  className="max-h-[100px] w-auto opacity-20 saturate-0 transition-all duration-300 hover:opacity-100 hover:saturate-100 object-contain"
                  alt="RustMailer Logo"
                />
                <h3 className="mt-4 text-lg font-semibold">No MTA</h3>
                <p className="mb-4 mt-2 text-sm text-muted-foreground">
                  You haven't added any MTA yet. Add one to start using email sending features.
                </p>
                <Button onClick={() => setOpen('add')}>
                  Add MTA
                </Button>
              </div>
            </div>
          )}
        </div>
      </Main>

      <MTAActionDialog
        key='MTA-add'
        open={open === 'add'}
        onOpenChange={() => setOpen('add')}
      />

      {currentRow && (
        <>
          <MTAActionDialog
            key={`MTA-edit-${currentRow.id}`}
            open={open === 'edit'}
            onOpenChange={() => {
              setOpen('edit')
              setTimeout(() => {
                setCurrentRow(null)
              }, 500)
            }}
            currentRow={currentRow}
          />

          <MTADeleteDialog
            key={`MTA-delete-${currentRow.id}`}
            open={open === 'delete'}
            onOpenChange={() => {
              setOpen('delete')
              setTimeout(() => {
                setCurrentRow(null)
              }, 500)
            }}
            currentRow={currentRow}
          />

          <SentTestEmailDialog
            key={`MTA-send-test-${currentRow.id}`}
            open={open === 'send-test'}
            onOpenChange={() => {
              setOpen('send-test')
              setTimeout(() => {
                setCurrentRow(null)
              }, 500)
            }}
            currentRow={currentRow}
          />
        </>
      )}
    </MTAProvider>
  )
}
