/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { Row } from '@tanstack/react-table'
import { Button } from '@/components/ui/button'
import { useAccessTokensContext } from '../context'
import { AccessToken } from '../data/schema'
import { MoreHorizontal } from 'lucide-react'

interface DataTableRowActionsProps {
  row: Row<AccessToken>
}

export function AclCellAction({ row }: DataTableRowActionsProps) {
  const { setOpen, setCurrentRow } = useAccessTokensContext()
  return (
    <Button variant='ghost' onClick={() => {
      if (row.original.acl) {
        setCurrentRow(row.original)
        setOpen('acl-detail')
      }
    }}>
      {row.original.acl ? <MoreHorizontal className='h-4 w-4' /> : <span className='text-xs'>n/a</span>}
    </Button>
  )
}
