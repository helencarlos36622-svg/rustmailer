/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { Row } from '@tanstack/react-table'
import { Button } from '@/components/ui/button'
import { EmailTemplate } from '../data/schema'

interface DataTableRowActionsProps {
  row: Row<EmailTemplate>
}

export function AccountCellAction({ row }: DataTableRowActionsProps) {
  const accounts = row.getValue('accounts') as { account_id: string; email: string } | undefined;

  if (accounts) {
    return <Button variant='ghost'>
      <span>{accounts.email}</span>
    </Button>
  } else {
    <Button variant='ghost'>
      <span>n/a</span>
    </Button>
  }
}
