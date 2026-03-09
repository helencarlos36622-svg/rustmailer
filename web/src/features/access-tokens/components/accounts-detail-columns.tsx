/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { ColumnDef } from '@tanstack/react-table'
import LongText from '@/components/long-text'
import { AccountInfo } from '../data/schema'
import { DataTableColumnHeader } from './data-table-column-header'

export const columns: ColumnDef<AccountInfo>[] = [
  {
    accessorKey: 'id',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Account_Id' />
    ),
    cell: ({ row }) => (
      <LongText className='max-w-80'>{row.original.id}</LongText>
    ),
    meta: { className: 'w-80' },
    enableHiding: false,
    enableSorting: false
  },
  {
    accessorKey: 'email',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Email' />
    ),
    cell: ({ row }) => (
      <LongText className='max-w-80'>{row.getValue('email')}</LongText>
    ),
    meta: { className: 'w-80' },
    enableHiding: true,
    enableSorting: false
  },
]
