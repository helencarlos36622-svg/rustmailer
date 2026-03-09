/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import React from 'react'
import { Proxy } from '../data/schema'

export type ProxyDialogType = 'add' | 'edit' | 'delete'

interface ProxyContextType {
  open: ProxyDialogType | null
  setOpen: (str: ProxyDialogType | null) => void
  currentRow: Proxy | null
  setCurrentRow: React.Dispatch<React.SetStateAction<Proxy | null>>
}

const ProxyContext = React.createContext<ProxyContextType | null>(null)

interface Props {
  children: React.ReactNode
  value: ProxyContextType
}

export default function ProxyProvider({ children, value }: Props) {
  return <ProxyContext.Provider value={value}>{children}</ProxyContext.Provider>
}

export const useProxyContext = () => {
  const proxyContext = React.useContext(ProxyContext)

  if (!proxyContext) {
    throw new Error(
      'useProxyContext has to be used within <ProxyContext.Provider>'
    )
  }

  return proxyContext
}
