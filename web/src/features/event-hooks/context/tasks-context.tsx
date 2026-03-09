/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import React from 'react'
import { EventHook } from '../data/schema'

export type EventHooksDialogType = 'create' | 'update' | 'delete'

interface EventHooksContextType {
  open: EventHooksDialogType | null
  setOpen: (str: EventHooksDialogType | null) => void
  currentRow: EventHook | null
  setCurrentRow: React.Dispatch<React.SetStateAction<EventHook | null>>
}

const EventHooksContext = React.createContext<EventHooksContextType | null>(null)

interface Props {
  children: React.ReactNode
  value: EventHooksContextType
}

export default function EventHooksContextProvider({ children, value }: Props) {
  return <EventHooksContext.Provider value={value}>{children}</EventHooksContext.Provider>
}

// eslint-disable-next-line react-refresh/only-export-components
export const useEventHooksContext = () => {
  const eventHooksContext = React.useContext(EventHooksContext)

  if (!eventHooksContext) {
    throw new Error(
      'useEventHooksContext has to be used within <EventHooksContext.Provider>'
    )
  }

  return eventHooksContext
}
