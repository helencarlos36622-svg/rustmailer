/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { Table } from '@tanstack/react-table'
import { Input } from '@/components/ui/input'
import { DataTableViewOptions } from './data-table-view-options'

interface DataTableToolbarProps<TData> {
  table: Table<TData>
}

export function DataTableToolbar<TData>({
  table,
}: DataTableToolbarProps<TData>) {
  return (
    <div className='flex items-center justify-between'>
      <div className='flex flex-1 flex-col-reverse items-start gap-y-2 sm:flex-row sm:items-center sm:space-x-2'>
        <Input
          placeholder='Filter MTA...'
          value={(table.getState().globalFilter as string) ?? ''}
          onChange={(event) => {
            table.setGlobalFilter(event.target.value);
          }}
          className='h-8 w-80'
        />
      </div>
      <DataTableViewOptions table={table} />
    </div>
  )
}
