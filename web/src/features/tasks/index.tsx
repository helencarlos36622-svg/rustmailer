/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { useState } from 'react'
import useDialogState from '@/hooks/use-dialog-state'
import { Main } from '@/components/layout/main'
import { columns as emailColumns } from './components/email-columns'
import { columns as eventColumns } from './components/event-columns'
import { EmailTaskTable } from './components/email-table'
import TasksProvider, {
  type TasksDialogType,
} from './context'
import { EmailTask, EventHookTask } from './data/schema'
import { EmailTaskRemoveDialog } from './components/delete-email-task-dialog'
import { TaskDetailDialog } from './components/task-detail'
import { FixedHeader } from '@/components/layout/fixed-header'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { EventTaskTable } from './components/event-table'
import { EventTaskRemoveDialog } from './components/delete-event-task-dialog'

export default function Tasks() {
  // Dialog states
  const [currentEmailRow, setCurrentEmailRow] = useState<EmailTask | null>(null)
  const [currentEventRow, setCurrentEventRow] = useState<EventHookTask | null>(null)

  const [open, setOpen] = useDialogState<TasksDialogType>(null)
  const [activeTab, setActiveTab] = useState<'email' | 'event'>('email')

  return (
    <TasksProvider value={{ open, setOpen, currentEmailRow, setCurrentEmailRow, currentEventRow, setCurrentEventRow }}>
      {/* ===== Top Heading ===== */}
      <FixedHeader />

      <Main>
        <div className='mb-4 flex items-center justify-between space-y-2 flex-wrap gap-x-4'>
          <div>
            <h2 className='text-2xl font-bold tracking-tight'>Task Queue</h2>
            <p className='text-muted-foreground'>
              The Task Queue stores email sending tasks and event hook tasks submitted via REST or gRPC, awaiting scheduling for delivery.
            </p>
          </div>
        </div>

        <Tabs value={activeTab} onValueChange={(value) => setActiveTab(value as 'email' | 'event')} className='w-full'>
          <TabsList className='grid w-[200px] grid-cols-2'>
            <TabsTrigger value='email'>Emails</TabsTrigger>
            <TabsTrigger value='event'>Events</TabsTrigger>
          </TabsList>
          <TabsContent value='email'>
            <div className='-mx-4 flex-1 overflow-auto px-4 py-1 flex-row lg:space-x-8 space-y-0'>
              <EmailTaskTable columns={emailColumns} />
            </div>
          </TabsContent>
          <TabsContent value='event'>
            <div className='-mx-4 flex-1 overflow-auto px-4 py-1 flex-row lg:space-x-8 space-y-0'>
              <EventTaskTable columns={eventColumns} />
            </div>
          </TabsContent>
        </Tabs>
      </Main>


      {currentEmailRow && (
        <>
          <EmailTaskRemoveDialog
            key={`task-remove-${currentEmailRow.id}`}
            open={open === 'delete'}
            onOpenChange={() => {
              setOpen('delete')
              setTimeout(() => {
                setCurrentEmailRow(null)
              }, 500)
            }}
            currentRow={currentEmailRow}

          />
          <TaskDetailDialog
            key='task-detail'
            open={open === 'detail'}
            currentRow={currentEmailRow}
            onOpenChange={() => {
              setOpen('detail')
              setTimeout(() => {
                setCurrentEmailRow(null)
              }, 500)
            }}
          />
        </>
      )}


      {currentEventRow && (
        <>
          <EventTaskRemoveDialog
            key={`task-remove-${currentEventRow.id}`}
            open={open === 'delete'}
            onOpenChange={() => {
              setOpen('delete')
              setTimeout(() => {
                setCurrentEventRow(null)
              }, 500)
            }}
            currentRow={currentEventRow}

          />
          <TaskDetailDialog
            key='task-detail'
            open={open === 'detail'}
            currentRow={currentEventRow}
            onOpenChange={() => {
              setOpen('detail')
              setTimeout(() => {
                setCurrentEventRow(null)
              }, 500)
            }}
          />
        </>
      )}
    </TasksProvider>
  )
}
