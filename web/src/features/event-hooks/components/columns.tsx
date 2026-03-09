/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { ColumnDef } from '@tanstack/react-table'
import { Badge } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import { EventHook } from '../data/schema'
import { DataTableColumnHeader } from './data-table-column-header'
import { DataTableRowActions } from './data-table-row-actions'
import LongText from '@/components/long-text'
import { format } from 'date-fns'
import { EnableAction } from './enable-action'

export const columns: ColumnDef<EventHook>[] = [
  {
    id: 'select',
    header: ({ table }) => (
      <Checkbox
        checked={
          table.getIsAllPageRowsSelected() ||
          (table.getIsSomePageRowsSelected() && 'indeterminate')
        }
        onCheckedChange={(value) => table.toggleAllPageRowsSelected(!!value)}
        aria-label='Select all'
        className='translate-y-[2px]'
      />
    ),
    cell: ({ row }) => (
      <Checkbox
        checked={row.getIsSelected()}
        onCheckedChange={(value) => row.toggleSelected(!!value)}
        aria-label='Select row'
        className='translate-y-[2px]'
      />
    ),
    enableSorting: false,
    enableHiding: false,
  },
  {
    accessorKey: 'id',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Id' />
    ),
    cell: ({ row }) => <LongText>{`${row.original.id}`}</LongText>,
    enableSorting: false,
    enableHiding: false,
  },
  {
    accessorKey: 'email',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Account' />
    ),
    cell: ({ row }) => <LongText>{row.original.email ?? "n/a"}</LongText>,
    enableSorting: false,
    enableHiding: false,
  },
  {
    id: 'type',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Type' />
    ),
    cell: ({ row }) => {
      const hook_type = row.original.hook_type;

      return (
        <div className='flex space-x-2 max-w-80'>
          {hook_type === "Http" ? (
            <Badge variant='outline' className='bg-blue-100 text-blue-800'>
              HTTP
            </Badge>
          ) : hook_type === "Nats" ? (
            <Badge variant='outline' className='bg-green-100 text-green-800'>
              NATS
            </Badge>
          ) : (
            <Badge variant='outline' className='bg-gray-100 text-gray-800'>
              Unknown
            </Badge>
          )}
        </div>
      )
    },
  },
  {
    accessorKey: 'enabled',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Enabled' />
    ),
    cell: EnableAction,
  },
  {
    accessorKey: 'global',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Global' />
    ),
    cell: ({ row }) => {
      const global = row.original.global;
      if (global === 1) {
        return <Checkbox className='max-w-8' checked disabled />
      } else {
        return <Checkbox className='max-w-8' disabled />
      }
    },
  },
  {
    accessorKey: 'call_count',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Call Count' />
    ),
    cell: ({ row }) => (
      <div className="text-center">
        <LongText className='max-w-8'>{row.original.call_count}</LongText>
      </div>
    ),
    enableSorting: true,
    enableHiding: false,
  },
  {
    accessorKey: 'success_count',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Success Count' />
    ),
    cell: ({ row }) => (
      <div className="text-center">
        <LongText className='max-w-8'>{row.original.success_count}</LongText>
      </div>
    ),
    enableSorting: true,
    enableHiding: false,
  },
  {
    accessorKey: 'failure_count',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Failure Count' />
    ),
    cell: ({ row }) => (
      <div className="text-center">
        <LongText className='max-w-8'>{row.original.failure_count}</LongText>
      </div>
    ),
    enableSorting: true,
    enableHiding: false,
  },
  {
    accessorKey: 'last_error',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Last Error' />
    ),
    cell: ({ row }) => <LongText className='max-w-48'>{row.original.last_error ?? 'n/a'}</LongText>,
    enableSorting: false,
    enableHiding: false,
  },
  {
    accessorKey: 'created_at',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Created At' />
    ),
    cell: ({ row }) => {
      const created_at = row.original.created_at;
      const date = format(new Date(created_at), 'yyyy-MM-dd HH:mm:ss');
      return <LongText className='max-w-36'>{date}</LongText>;
    },
    meta: { className: 'w-32' },
    enableHiding: false,
  },
  {
    accessorKey: 'updated_at',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Updated At' />
    ),
    cell: ({ row }) => {
      const updated_at = row.original.updated_at;
      const date = format(new Date(updated_at), 'yyyy-MM-dd HH:mm:ss');
      return <LongText className='max-w-36'>{date}</LongText>;
    },
    meta: { className: 'w-32' },
    enableHiding: false,
  },
  {
    id: 'actions',
    cell: ({ row }) => <DataTableRowActions row={row} />,
  },
]
