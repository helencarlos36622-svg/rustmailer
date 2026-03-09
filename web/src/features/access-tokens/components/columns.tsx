/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { ColumnDef } from '@tanstack/react-table'
import { cn } from '@/lib/utils'
import { Badge } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import LongText from '@/components/long-text'

import { AccessToken } from '../data/schema'
import { DataTableColumnHeader } from './data-table-column-header'
import { DataTableRowActions } from './data-table-row-actions'
import { format, formatDistanceToNow } from 'date-fns'
import { AccountCellAction } from './account-action'
import { AclCellAction } from './acl-action'

export const columns: ColumnDef<AccessToken>[] = [
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
    accessorKey: 'token',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Token' />
    ),
    cell: ({ row }) => {
      return <LongText className='w-60'>{row.original.token}</LongText>
    },
    meta: { className: 'w-60' },
    enableHiding: false,
    enableSorting: false,
  },
  {
    accessorKey: 'accounts',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Accounts' />
    ),
    cell: AccountCellAction,
    meta: { className: 'w-10 text-center' },
    filterFn: (row, columnId, filterValue) => {
      const accounts = row.getValue(columnId) as { account_id: number; email: string }[];
      if (!filterValue) return true;
      return accounts.some(
        (account) =>
          `${account.account_id}`.includes(filterValue) ||
          account.email.includes(filterValue)
      );
    },
  },
  {
    id: 'scopes',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Scopes' />
    ),
    cell: ({ row }) => {
      const access_scopes = row.original.access_scopes;
      return (
        <div className="flex flex-wrap w-64 gap-1">
          {access_scopes.map((scope) => (
            <Badge
              key={scope}
              variant="outline"
              className={cn(
                'capitalize',
                'bg-blue-100/30 text-blue-900 dark:text-blue-200 border-blue-200'
              )}
            >
              {scope}
            </Badge>
          ))}
        </div>
      );
    },
    meta: { className: 'w-52' },
  },
  {
    id: 'acl',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Acl' />
    ),
    cell: AclCellAction,
    meta: { className: 'w-8 text-center' },
    enableSorting: false
  },
  {
    accessorKey: 'description',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Description' />
    ),
    cell: ({ row }) => (
      <LongText className='max-w-80'>{row.original.description}</LongText>
    ),
    meta: { className: 'w-80' },
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
