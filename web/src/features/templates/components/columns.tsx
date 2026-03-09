/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { ColumnDef } from '@tanstack/react-table'
import { cn } from '@/lib/utils'
import { Checkbox } from '@/components/ui/checkbox'
import LongText from '@/components/long-text'

import { EmailTemplate } from '../data/schema'
import { DataTableColumnHeader } from './data-table-column-header'
import { DataTableRowActions } from './data-table-row-actions'
import { format, formatDistanceToNow } from 'date-fns'

export const columns: ColumnDef<EmailTemplate>[] = [
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
    meta: {
      className: cn(
        'sticky md:table-cell left-0 z-10 rounded-tl',
        'bg-background transition-colors duration-200 group-hover/row:bg-muted group-data-[state=selected]/row:bg-muted'
      ),
    },
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
    cell: ({ row }) => (
      <LongText className='max-w-72'>{`${row.original.id}`}</LongText>
    ),
    meta: { className: 'w-72' },
    enableHiding: false,
    enableSorting: false
  },
  {
    accessorKey: 'account',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Account' />
    ),
    cell: ({ row }) => {
      const account = row.original.account;
      if (account) {
        return <LongText>{account.email}</LongText>
      } else {
        return <LongText className='text-xs'>n/a</LongText>
      }
    },
    filterFn: (row, columnId, filterValue) => {
      const accounts = row.getValue(columnId) as { account_id: string; email: string }[];
      if (!filterValue) return true;
      return accounts.some(
        (account) =>
          account.account_id.includes(filterValue) ||
          account.email.includes(filterValue)
      );
    },
  },
  {
    id: "isPublic",
    header: ({ column }) => (
      <DataTableColumnHeader className="ml-4" column={column} title='Public' />
    ),
    cell: ({ row }) => {
      const account = row.original.account;
      if (account) {
        return <Checkbox className='max-w-8' disabled />
      } else {
        return <Checkbox className='max-w-8' checked disabled />
      }
    },
    meta: { className: 'w-8 text-center' },
  },
  {
    accessorKey: 'description',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Description' />
    ),
    cell: ({ row }) => (
      <LongText className='max-w-96'>{row.original.description}</LongText>
    ),
    meta: { className: 'w-96' },
    enableHiding: true,
    enableSorting: false
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
    meta: { className: 'w-36' },
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
    meta: { className: 'w-36' },
    enableHiding: false,
  },
  {
    accessorKey: 'last_access_at',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Last Access' />
    ),
    cell: ({ row }) => {
      const last_access_at = row.original.last_access_at;
      if (last_access_at === 0) {
        return <LongText className='max-w-40'>Not used yet</LongText>;
      }
      const result = formatDistanceToNow(new Date(last_access_at), { addSuffix: true });
      return <LongText className='max-w-40'>{result}</LongText>;
    },
    meta: { className: 'w-40' },
    enableHiding: false,
  },
  {
    id: 'actions',
    cell: DataTableRowActions,
  },
]
