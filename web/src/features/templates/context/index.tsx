/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import React from 'react'
import { EmailTemplate } from '../data/schema'

export type EmailTemplatesDialogType = 'add' | 'edit' | 'delete' | 'send-test'

interface EmailTemplatesContextType {
  open: EmailTemplatesDialogType | null
  setOpen: (str: EmailTemplatesDialogType | null) => void
  currentRow: EmailTemplate | null
  setCurrentRow: React.Dispatch<React.SetStateAction<EmailTemplate | null>>
}

const EmailTemplatesContext = React.createContext<EmailTemplatesContextType | null>(null)

interface Props {
  children: React.ReactNode
  value: EmailTemplatesContextType
}

export default function EmailTemplatesProvider({ children, value }: Props) {
  return <EmailTemplatesContext.Provider value={value}>{children}</EmailTemplatesContext.Provider>
}

export const useEmailTemplatesContext = () => {
  const emailTemplatesContext = React.useContext(EmailTemplatesContext)

  if (!emailTemplatesContext) {
    throw new Error(
      'useEmailTemplatesContext has to be used within <EmailTemplatesContext.Provider>'
    )
  }

  return emailTemplatesContext
}
