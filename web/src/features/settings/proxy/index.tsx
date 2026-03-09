/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { useState } from 'react'
import useDialogState from '@/hooks/use-dialog-state'
import { Button } from '@/components/ui/button'
import { ProxyActionDialog } from './components/action-dialog'
import { columns } from './components/columns'
import { ProxyDeleteDialog } from './components/delete-dialog'
import { ProxyTable } from './components/table'
import ProxyProvider, {
  type ProxyDialogType,
} from './context'
import { Plus } from 'lucide-react'
import { Proxy } from './data/schema'
import { TableSkeleton } from '@/components/table-skeleton'
import Logo from '@/assets/logo.svg'
import useProxyList from '@/hooks/use-proxy'

export default function ProxyManagerPage() {
  // Dialog states
  const [currentRow, setCurrentRow] = useState<Proxy | null>(null)
  const [open, setOpen] = useDialogState<ProxyDialogType>(null)

  const { proxyList, isLoading } = useProxyList();

  return (
    <ProxyProvider value={{ open, setOpen, currentRow, setCurrentRow }}>
      <div>
        <div className='mb-2 flex items-center justify-between space-y-2 flex-wrap gap-x-4'>
          <div>
            <h2 className='text-2xl font-bold tracking-tight'>Proxy Management</h2>
          </div>
          <div className='flex gap-2'>
            <Button className='space-x-1' onClick={() => setOpen('add')}>
              <span>Add</span> <Plus size={18} />
            </Button>
          </div>
        </div>
        <div className='-mx-4 flex-1 md:w-[960px] overflow-auto px-4 py-1 flex-row lg:space-x-12 space-y-0'>
          {isLoading ? <TableSkeleton columns={columns.length} rows={10} /> : proxyList?.length ? (
            <ProxyTable data={proxyList} columns={columns} />
          ) : (
            <div className="flex h-[450px] shrink-0 items-center justify-center rounded-md border border-dashed">
              <div className="mx-auto flex max-w-[420px] flex-col items-center justify-center text-center">
                <img
                  src={Logo}
                  className="max-h-[100px] w-auto opacity-20 saturate-0 transition-all duration-300 hover:opacity-100 hover:saturate-100 object-contain"
                  alt="RustMailer Logo"
                />
                <h3 className="mt-4 text-lg font-semibold">No Proxy</h3>
                <p className="mb-4 mt-2 text-sm text-muted-foreground">
                  You haven't added any Proxy yet.
                </p>
                <Button onClick={() => setOpen('add')}>
                  Add Proxy
                </Button>
              </div>
            </div>
          )}
        </div>
      </div>

      <ProxyActionDialog
        key='Proxy-add'
        open={open === 'add'}
        onOpenChange={() => setOpen('add')}
      />

      {currentRow && (
        <>
          <ProxyActionDialog
            key={`Proxy-edit-${currentRow.id}`}
            open={open === 'edit'}
            onOpenChange={() => {
              setOpen('edit')
              setTimeout(() => {
                setCurrentRow(null)
              }, 500)
            }}
            currentRow={currentRow}
          />

          <ProxyDeleteDialog
            key={`Proxy-delete-${currentRow.id}`}
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
    </ProxyProvider>
  )
}
