/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { Row } from '@tanstack/react-table'
import { Button } from '@/components/ui/button'
import { useAccessTokensContext } from '../context'
import { AccessToken } from '../data/schema'

interface DataTableRowActionsProps {
  row: Row<AccessToken>
}

export function AccountCellAction({ row }: DataTableRowActionsProps) {
  const { setOpen, setCurrentRow } = useAccessTokensContext()

  const accounts = row.original.accounts;
  return (
    <Button variant='ghost' onClick={() => {
      setCurrentRow(row.original)
      setOpen('account-detail')
    }}>
      <span>{accounts.length}</span>
    </Button>
  )
}
