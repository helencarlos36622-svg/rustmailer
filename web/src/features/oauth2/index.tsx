/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { useState } from 'react'
import useDialogState from '@/hooks/use-dialog-state'
import { Button } from '@/components/ui/button'
import { Main } from '@/components/layout/main'
import { ActionDialog } from './components/action-dialog'
import { columns } from './components/columns'
import { TokenDeleteDialog } from './components/delete-dialog'
import { Oauth2Table } from './components/oauth2-table'
import OAuth2Provider, {
  type OAuth2DialogType,
} from './context'
import { Plus } from 'lucide-react'
import Logo from '@/assets/logo.svg'
import { OAuth2Entity } from './data/schema'
import { useQuery } from '@tanstack/react-query'
import { get_oauth2_list } from '@/api/oauth2/api'
import { TableSkeleton } from '@/components/table-skeleton'
import { AuthorizeDialog } from './components/authorize-dialog'
import { FixedHeader } from '@/components/layout/fixed-header'

export default function OAuth2() {
  const [currentRow, setCurrentRow] = useState<OAuth2Entity | null>(null)
  const [open, setOpen] = useDialogState<OAuth2DialogType>(null)


  const { data: oauth2List, isLoading } = useQuery({
    queryKey: ['oauth2-list'],
    queryFn: get_oauth2_list,
  })

  return (
    <OAuth2Provider value={{ open, setOpen, currentRow, setCurrentRow }}>
      {/* ===== Top Heading ===== */}
      <FixedHeader />

      <Main>
        <div className='mb-2 flex items-center justify-between space-y-2 flex-wrap gap-x-4'>
          <div>
            <h2 className='text-2xl font-bold tracking-tight'>OAuth2</h2>
            <p className='text-muted-foreground'>
              Automatically handling access token retrieval, storage, and refreshing, ensuring a seamless and always-ready authentication experience.
            </p>
          </div>
          <div className='flex gap-2'>
            <Button className='space-x-1' onClick={() => setOpen('add')}>
              <span>Add</span> <Plus size={18} />
            </Button>
          </div>
        </div>
        <div className='-mx-4 flex-1 overflow-auto px-4 py-1 flex-row lg:space-x-12 space-y-0'>
          {isLoading ? <TableSkeleton columns={columns.length} rows={10} /> : oauth2List?.items.length ? (
            <Oauth2Table data={oauth2List.items} columns={columns} />
          ) : (
            <div className="flex h-[450px] shrink-0 items-center justify-center rounded-md border border-dashed">
              <div className="mx-auto flex max-w-[420px] flex-col items-center justify-center text-center">
                <img
                  src={Logo}
                  className="max-h-[100px] w-auto opacity-20 saturate-0 transition-all duration-300 hover:opacity-100 hover:saturate-100 object-contain"
                  alt="RustMailer Logo"
                />
                <h3 className="mt-4 text-lg font-semibold">No OAuth2 Configurations</h3>
                <p className="mb-4 mt-2 text-sm text-muted-foreground">
                  You haven't added any OAuth2 configurations yet. Add one to start using OAuth2 features.
                </p>
                <Button onClick={() => setOpen('add')}>
                  Add Configuration
                </Button>
              </div>
            </div>
          )}
        </div>
      </Main>

      <ActionDialog
        key='oauth2-add'
        open={open === 'add'}
        onOpenChange={() => setOpen('add')}
      />

      {currentRow && (
        <>
          <ActionDialog
            key={`oauth2-edit-${currentRow.id}`}
            open={open === 'edit'}
            onOpenChange={() => {
              setOpen('edit')
              setTimeout(() => {
                setCurrentRow(null)
              }, 500)
            }}
            currentRow={currentRow}
          />

          <TokenDeleteDialog
            key={`oauth2-delete-${currentRow.id}`}
            open={open === 'delete'}
            onOpenChange={() => {
              setOpen('delete')
              setTimeout(() => {
                setCurrentRow(null)
              }, 500)
            }}
            currentRow={currentRow}
          />

          <AuthorizeDialog
            key={`oauth2-authorize-${currentRow.id}`}
            open={open === 'authorize'}
            onOpenChange={() => {
              setOpen('authorize')
              setTimeout(() => {
                setCurrentRow(null)
              }, 500)
            }}
            currentRow={currentRow}
          />
        </>
      )}
    </OAuth2Provider>
  )
}
