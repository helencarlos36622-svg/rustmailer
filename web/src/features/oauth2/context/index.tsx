/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import React from 'react'
import { OAuth2Entity } from '../data/schema'

export type OAuth2DialogType = 'add' | 'edit' | 'delete' | 'authorize'

interface OAuth2ContextType {
  open: OAuth2DialogType | null
  setOpen: (str: OAuth2DialogType | null) => void
  currentRow: OAuth2Entity | null
  setCurrentRow: React.Dispatch<React.SetStateAction<OAuth2Entity | null>>
}

const OAuth2Context = React.createContext<OAuth2ContextType | null>(null)

interface Props {
  children: React.ReactNode
  value: OAuth2ContextType
}

export default function OAuth2Provider({ children, value }: Props) {
  return <OAuth2Context.Provider value={value}>{children}</OAuth2Context.Provider>
}

export const useOAuth2Context = () => {
  const oauth2Context = React.useContext(OAuth2Context)

  if (!oauth2Context) {
    throw new Error(
      'useOAuth2Context has to be used within <OAuth2Context.Provider>'
    )
  }

  return oauth2Context
}
