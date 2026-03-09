/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { useState } from 'react'
import useDialogState from '@/hooks/use-dialog-state'
import { Button } from '@/components/ui/button'
import { Main } from '@/components/layout/main'
import { TokensActionDialog } from './components/action-dialog'
import { columns } from './components/columns'
import { TokenDeleteDialog } from './components/delete-dialog'
import { AccessTokensTable } from './components/access-token-table'
import AccessTokensProvider, {
  type AccessTokensDialogType,
} from './context'
import Logo from '@/assets/logo.svg'
import { Plus } from 'lucide-react'
import { AccessToken } from './data/schema'
import { AccountDetailDialog } from './components/accounts-detail-dialog'
import { AclDetailDialog } from './components/acl-detail-dialog'
import { useQuery } from '@tanstack/react-query'
import { list_access_tokens } from '@/api/access-tokens/api'
import { TableSkeleton } from '@/components/table-skeleton'
import { FixedHeader } from '@/components/layout/fixed-header'

export default function AccessTokens() {
  // Dialog states
  const [currentRow, setCurrentRow] = useState<AccessToken | null>(null)
  const [open, setOpen] = useDialogState<AccessTokensDialogType>(null)

  const { data: accessTokens, isLoading } = useQuery({
    queryKey: ['access-tokens'],
    queryFn: list_access_tokens,
  })

  return (
    <AccessTokensProvider value={{ open, setOpen, currentRow, setCurrentRow }}>
      {/* ===== Top Heading ===== */}
      <FixedHeader />

      <Main>
        <div className='mb-2 flex items-center justify-between space-y-2 flex-wrap gap-x-4'>
          <div>
            <h2 className='text-2xl font-bold tracking-tight'>Access Token</h2>
            <p className='text-muted-foreground'>
              Manage your access tokens, their access controls, IP whitelists, and rate-limiting settings here.
            </p>
          </div>
          <div className='flex gap-2'>
            <Button className='space-x-1' onClick={() => setOpen('add')}>
              <span>Add</span> <Plus size={18} />
            </Button>
          </div>
        </div>
        <div className='-mx-4 flex-1 overflow-auto px-4 py-1 flex-row lg:space-x-12 space-y-0'>
          {isLoading ? (
            <TableSkeleton columns={columns.length} rows={10} />
          ) : accessTokens?.length ? (
            <AccessTokensTable data={accessTokens} columns={columns} />
          ) : (
            <div className="flex h-[450px] shrink-0 items-center justify-center rounded-md border border-dashed">
              <div className="mx-auto flex max-w-[420px] flex-col items-center justify-center text-center">
                <img
                  src={Logo}
                  className="max-h-[100px] w-auto opacity-20 saturate-0 transition-all duration-300 hover:opacity-100 hover:saturate-100 object-contain"
                  alt="RustMailer Logo"
                />
                <h3 className="mt-4 text-lg font-semibold">No Access Tokens</h3>
                <p className="mb-4 mt-2 text-sm text-muted-foreground">
                  You haven't created any access tokens yet. Create one to start accessing the API securely.
                </p>
                <Button onClick={() => setOpen('add')}>
                  Create Token
                </Button>
              </div>
            </div>
          )}
        </div>
      </Main>

      <TokensActionDialog
        key='token-add'
        open={open === 'add'}
        onOpenChange={() => setOpen('add')}
      />

      {currentRow && (
        <>
          <TokensActionDialog
            key={`token-edit-${currentRow.token}`}
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
            key={`token-delete-${currentRow.token}`}
            open={open === 'delete'}
            onOpenChange={() => {
              setOpen('delete')
              setTimeout(() => {
                setCurrentRow(null)
              }, 500)
            }}
            currentRow={currentRow}
          />

          <AccountDetailDialog
            key={`accounts-detail-${currentRow.token}`}
            currentRow={currentRow}
            open={open === 'account-detail'}
            onOpenChange={() => {
              setOpen('account-detail')
              setTimeout(() => {
                setCurrentRow(null)
              }, 500)
            }} />

          <AclDetailDialog
            key={`acl-detail-${currentRow.token}`}
            currentRow={currentRow}
            open={open === 'acl-detail'}
            onOpenChange={() => {
              setOpen('acl-detail')
              setTimeout(() => {
                setCurrentRow(null)
              }, 500)
            }} />
        </>
      )}
    </AccessTokensProvider>
  )
}
