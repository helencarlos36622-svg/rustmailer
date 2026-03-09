/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { useState } from 'react'
import useDialogState from '@/hooks/use-dialog-state'
import { Button } from '@/components/ui/button'
import { Main } from '@/components/layout/main'
import { TemplateActionDialog } from './components/action-dialog'
import { columns } from './components/columns'
import { TemplateDeleteDialog } from './components/delete-dialog'
import { EmailTemplatesTable } from './components/table'
import EmailTemplatesProvider, {
  type EmailTemplatesDialogType,
} from './context'
import { Plus } from 'lucide-react'
import { EmailTemplate } from './data/schema'
import { SentTestEmailDialog } from './components/send-email-test-dialog'
import { FixedHeader } from '@/components/layout/fixed-header'

export default function EmailTemplates() {
  // Dialog states
  const [currentRow, setCurrentRow] = useState<EmailTemplate | null>(null)
  const [open, setOpen] = useDialogState<EmailTemplatesDialogType>(null)


  return (
    <EmailTemplatesProvider value={{ open, setOpen, currentRow, setCurrentRow }}>
      {/* ===== Top Heading ===== */}
      <FixedHeader />

      <Main>
        <div className='mb-2 flex items-center justify-between space-y-2 flex-wrap gap-x-4'>
          <div>
            <h2 className='text-2xl font-bold tracking-tight'>Email Templates</h2>
            <p className='text-muted-foreground'>
              Email templates with Handlebars support dynamic content for personalized messaging.
            </p>
          </div>
          <div className='flex gap-2'>
            <Button className='space-x-1' onClick={() => setOpen('add')}>
              <span>Add</span> <Plus size={18} />
            </Button>
          </div>
        </div>
        <div className='-mx-4 flex-1 overflow-auto px-4 py-1 flex-row lg:space-x-8 space-y-0'>
          <EmailTemplatesTable columns={columns} onCallback={() => setOpen('add')} />
        </div>
      </Main>
      <TemplateActionDialog
        key='template-add'
        open={open === 'add'}
        onOpenChange={() => setOpen('add')}
      />

      {currentRow && (
        <>
          <TemplateActionDialog
            key={`template-edit-${currentRow.id}`}
            open={open === 'edit'}
            onOpenChange={() => {
              setOpen('edit')
              setTimeout(() => {
                setCurrentRow(null)
              }, 500)
            }}
            currentRow={currentRow}
          />

          <TemplateDeleteDialog
            key={`template-delete-${currentRow.id}`}
            open={open === 'delete'}
            onOpenChange={() => {
              setOpen('delete')
              setTimeout(() => {
                setCurrentRow(null)
              }, 500)
            }}
            currentRow={currentRow}
          />

          <SentTestEmailDialog
            key={`template-send-test-${currentRow.id}`}
            open={open === 'send-test'}
            onOpenChange={() => {
              setOpen('send-test')
              setTimeout(() => {
                setCurrentRow(null)
              }, 500)
            }}
            currentRow={currentRow}
          />
        </>
      )}
    </EmailTemplatesProvider>
  )
}
