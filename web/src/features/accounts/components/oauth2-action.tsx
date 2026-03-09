/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { Row } from '@tanstack/react-table'
import { Button } from '@/components/ui/button'
import { useAccountContext } from '../context'
import { AccountEntity, MailerType } from '../data/schema'

interface DataTableRowActionsProps {
  row: Row<AccountEntity>
}
export function OAuth2Action({ row }: DataTableRowActionsProps) {
  const { setOpen, setCurrentRow } = useAccountContext()
  const mailer = row.original

  const isOAuth2 =
    (mailer.mailer_type === MailerType.ImapSmtp &&
      mailer.imap?.auth.auth_type === "OAuth2") ||
    mailer.mailer_type === MailerType.GmailApi ||
    mailer.mailer_type === MailerType.GraphApi

  if (isOAuth2) {
    return (
      <Button
        variant="ghost"
        size="sm"
        className="text-xs text-blue-500 hover:text-blue-700 underline"
        onClick={() => {
          setCurrentRow(mailer)
          setOpen("oauth2")
        }}
      >
        OAuth2
      </Button>
    )
  }

  return <span className="text-xs text-muted-foreground">Password</span>
}
