/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { ColumnDef } from '@tanstack/react-table'
import { cn } from '@/lib/utils'
import { Checkbox } from '@/components/ui/checkbox'
import LongText from '@/components/long-text'

import { MTARecord } from '../data/schema'
import { DataTableColumnHeader } from './data-table-column-header'
import { DataTableRowActions } from './data-table-row-actions'
import { format, formatDistanceToNow } from 'date-fns'
import { Badge } from '@/components/ui/badge'

export const columns: ColumnDef<MTARecord>[] = [
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
      <LongText className='max-w-48'>{`${row.original.id}`}</LongText>
    ),
    enableHiding: false,
    meta: { className: 'w-48' },
    enableSorting: false
  },
  {
    id: 'host',
    accessorKey: "server.host",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Host' />
    ),
    cell: ({ row }) => {
      const server = row.original.server;
      return <LongText>{server.host}</LongText>
    },
  },
  {
    id: 'port',
    accessorKey: "server.port",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Port' />
    ),
    cell: ({ row }) => {
      const server = row.original.server;
      return <LongText className='max-w-52'>{server.port}</LongText>
    },
    meta: { className: 'w-18' },
    enableHiding: true,
    enableSorting: false
  },
  {
    id: 'use_proxy',
    accessorKey: "use_proxy",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Use Proxy' />
    ),
    cell: ({ row }) => {
      const use_proxy = row.original.use_proxy;
      if (use_proxy) {
        return <Checkbox className='max-w-24' checked disabled />
      } else {
        return <Checkbox className='max-w-24' disabled />
      }
    },
    meta: { className: 'w-24' },
    enableHiding: true,
    enableSorting: false
  },
  {
    id: 'encryption',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Secure' />
    ),
    cell: ({ row }) => {
      const server = row.original.server;
      const encryption = server.encryption;
      let badgeColor = '';
      switch (encryption) {
        case 'StartTls':
          badgeColor = 'bg-blue-100/30 text-blue-900 dark:text-blue-200 border-blue-200';
          break;
        case 'Ssl':
          badgeColor = 'bg-green-100/30 text-green-900 dark:text-green-200 border-green-200';
          break;
        case 'None':
          badgeColor = 'bg-gray-100/30 text-gray-900 dark:text-gray-200 border-gray-200';
          break;
        default:
          badgeColor = 'bg-gray-100/30 text-gray-900 dark:text-gray-200 border-gray-200';
      }

      return (
        <Badge
          variant="outline"
          className={cn('capitalize', badgeColor)}
        >
          {encryption}
        </Badge>
      );
    },
    meta: { className: 'w-18' },
  },
  {
    accessorKey: 'dsn_capable',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='DSN' />
    ),
    cell: ({ row }) => {
      const dsn_capable = row.original.dsn_capable;
      if (dsn_capable) {
        return <Checkbox className='max-w-8' checked disabled />
      } else {
        return <Checkbox className='max-w-8' disabled />
      }
    },
    meta: { className: 'w-18' },
    enableHiding: true,
    enableSorting: false
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
      const last_access_at = row.getValue('last_access_at') as number;
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
