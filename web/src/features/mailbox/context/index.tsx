/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import React from 'react'
import { EmailEnvelope } from '../data/schema'
import { MailboxData } from '@/api/mailbox/api'

export type MailboxDialogType = 'mailbox' | 'display' | 'move-to-trash' | 'filters'

interface MailboxContextType {
  open: MailboxDialogType | null
  setOpen: (str: MailboxDialogType | null) => void
  currentMailbox: MailboxData | undefined
  currentEnvelope: EmailEnvelope | undefined
  setCurrentMailbox: React.Dispatch<React.SetStateAction<MailboxData | undefined>>
  setCurrentEnvelope: React.Dispatch<React.SetStateAction<EmailEnvelope | undefined>>
  deleteIds: string[]
  setDeleteIds: React.Dispatch<React.SetStateAction<string[]>>
}

const MailboxContext = React.createContext<MailboxContextType | null>(null)

interface Props {
  children: React.ReactNode
  value: MailboxContextType
}

export default function MailboxProvider({ children, value }: Props) {
  return <MailboxContext.Provider value={value}>{children}</MailboxContext.Provider>
}

export const useMailboxContext = () => {
  const mailboxContext = React.useContext(MailboxContext)

  if (!mailboxContext) {
    throw new Error(
      'useMailboxContext has to be used within <MailboxContext.Provider>'
    )
  }

  return mailboxContext
}
