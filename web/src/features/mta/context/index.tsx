/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import React from 'react'
import { MTARecord } from '../data/schema'

export type MTADialogType = 'add' | 'edit' | 'delete' | 'send-test'

interface MTAContextType {
  open: MTADialogType | null
  setOpen: (str: MTADialogType | null) => void
  currentRow: MTARecord | null
  setCurrentRow: React.Dispatch<React.SetStateAction<MTARecord | null>>
}

const MTAContext = React.createContext<MTAContextType | null>(null)

interface Props {
  children: React.ReactNode
  value: MTAContextType
}

export default function MTAProvider({ children, value }: Props) {
  return <MTAContext.Provider value={value}>{children}</MTAContext.Provider>
}

export const useMTAContext = () => {
  const mtaContext = React.useContext(MTAContext)

  if (!mtaContext) {
    throw new Error(
      'useMTAContext has to be used within <MTAContext.Provider>'
    )
  }

  return mtaContext
}
