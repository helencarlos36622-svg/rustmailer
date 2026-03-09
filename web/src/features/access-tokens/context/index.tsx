/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import React from 'react'
import { AccessToken } from '../data/schema'

export type AccessTokensDialogType = 'add' | 'edit' | 'delete' | 'account-detail' | 'acl-detail'

interface AccessTokensContextType {
  open: AccessTokensDialogType | null
  setOpen: (str: AccessTokensDialogType | null) => void
  currentRow: AccessToken | null
  setCurrentRow: React.Dispatch<React.SetStateAction<AccessToken | null>>
}

const AccessTokensContext = React.createContext<AccessTokensContextType | null>(null)

interface Props {
  children: React.ReactNode
  value: AccessTokensContextType
}

export default function AccessTokensProvider({ children, value }: Props) {
  return <AccessTokensContext.Provider value={value}>{children}</AccessTokensContext.Provider>
}

export const useAccessTokensContext = () => {
  const accessTokensContext = React.useContext(AccessTokensContext)

  if (!accessTokensContext) {
    throw new Error(
      'useAccessTokensContext has to be used within <AccessTokensContext.Provider>'
    )
  }

  return accessTokensContext
}
