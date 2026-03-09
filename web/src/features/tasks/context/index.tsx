/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import React from 'react'
import { EmailTask, EventHookTask } from '../data/schema'

export type TasksDialogType = 'delete' | 'detail'

interface TasksContextType {
  open: TasksDialogType | null
  setOpen: (str: TasksDialogType | null) => void
  currentEmailRow: EmailTask | null
  currentEventRow: EventHookTask | null
  setCurrentEmailRow: React.Dispatch<React.SetStateAction<EmailTask | null>>
  setCurrentEventRow: React.Dispatch<React.SetStateAction<EventHookTask | null>>
}

const TasksContext = React.createContext<TasksContextType | null>(null)

interface Props {
  children: React.ReactNode
  value: TasksContextType
}

export default function TasksProvider({ children, value }: Props) {
  return <TasksContext.Provider value={value}>{children}</TasksContext.Provider>
}

export const useTasksContext = () => {
  const tasksContext = React.useContext(TasksContext)

  if (!tasksContext) {
    throw new Error(
      'useTasksContext has to be used within <TasksContext.Provider>'
    )
  }

  return tasksContext
}
