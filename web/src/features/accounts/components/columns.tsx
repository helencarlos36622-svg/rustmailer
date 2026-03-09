/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { ColumnDef } from '@tanstack/react-table'
import { cn } from '@/lib/utils'
import { Checkbox } from '@/components/ui/checkbox'
import LongText from '@/components/long-text'

import { AccountEntity } from '../data/schema'
import { DataTableColumnHeader } from './data-table-column-header'
import { DataTableRowActions } from './data-table-row-actions'
import { format } from 'date-fns'
import { OAuth2Action } from './oauth2-action'
import { RunningStateCellAction } from './running-state-action'
import { EnableAction } from './enable-action'

export const columns: ColumnDef<AccountEntity>[] = [
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
      <LongText className='max-w-30'>{row.original.id}</LongText>
    ),
    enableHiding: false,
    meta: { className: 'w-30' },
    enableSorting: false
  },
  {
    accessorKey: "enabled",
    header: ({ column }) => (
      <DataTableColumnHeader className="ml-4" column={column} title='Enabled' />
    ),
    cell: EnableAction,
    meta: { className: 'w-8 text-center' },
  },
  {
    accessorKey: "name",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Name' />
    ),
    cell: ({ row }) => {
      return <LongText>{row.original.name ?? "n/a"}</LongText>
    },
  },
  {
    accessorKey: "email",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Email' />
    ),
    cell: ({ row }) => {
      return <LongText>{row.original.email}</LongText>
    },
  },
  {
    id: 'auth_type',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Auth Type' />
    ),
    cell: OAuth2Action,
    meta: { className: 'w-8' },
    enableHiding: false,
    enableSorting: false
  },
  {
    id: 'mailer_type',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Type' />
    ),
    cell: ({ row }) => row.original.mailer_type,
    meta: { className: 'w-12' },
    enableHiding: false,
    enableSorting: false
  },
  {
    accessorKey: "full_sync_interval_min",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Full Sync' />
    ),
    cell: ({ row }) => {
      return <LongText className='max-w-12'>{row.original.full_sync_interval_min} min</LongText>
    },
    meta: { className: 'w-12 text-center' },
  },
  {
    accessorKey: "incremental_sync_interval_sec",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Inc Sync' />
    ),
    cell: ({ row }) => {
      return <LongText className='max-w-12'>{row.original.incremental_sync_interval_sec} sec</LongText>
    },
    meta: { className: 'w-12 text-center' },
  },
  {
    id: 'running_state',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Running State' />
    ),
    cell: RunningStateCellAction,
    meta: { className: 'w-36' },
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
    id: 'actions',
    cell: DataTableRowActions,
  },
]
