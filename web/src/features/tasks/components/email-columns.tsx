/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { ColumnDef } from '@tanstack/react-table'
import { cn } from '@/lib/utils'
import { Checkbox } from '@/components/ui/checkbox'
import LongText from '@/components/long-text'

import { EmailTask, TaskStatus } from '../data/schema'
import { DataTableColumnHeader } from './data-table-column-header'
import { EmailTableRowActions } from './email-table-row-actions'
import { format } from 'date-fns'
import { Badge } from '@/components/ui/badge'

export const columns: ColumnDef<EmailTask>[] = [
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
      <DataTableColumnHeader column={column} title='Task ID' />
    ),
    cell: ({ row }) => (
      <LongText className='max-w-24'>{`${row.original.id}`}</LongText>
    ),
    enableHiding: false,
    enableSorting: false
  },
  {
    accessorKey: 'account_email',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Account Email' />
    ),
    cell: ({ row }) => {
      return <LongText>{row.original.account_email}</LongText>
    },
  },
  {
    accessorKey: 'subject',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Subject' />
    ),
    cell: ({ row }) => {
      return <LongText className='max-w-24'>{row.original.subject || "n/a"}</LongText>;
    },
    enableHiding: true,
    enableSorting: false
  },
  {
    accessorKey: 'attachment_count',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Attachment' />
    ),
    cell: ({ row }) => {
      return <LongText className='max-w-8 text-center'>{row.original.attachment_count}</LongText>
    },
  },
  {
    accessorKey: 'retry_count',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Retry Count' />
    ),
    cell: ({ row }) => {
      return <LongText className='max-w-8 text-center'>{row.original.retry_count}</LongText>
    },
  },
  {
    accessorKey: 'status',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Status' />
    ),
    cell: ({ row }) => {
      let status = row.original.status;
      let className;
      switch (status) {
        case TaskStatus.Scheduled:
          className = 'bg-blue-100 text-blue-500';
          break;
        case TaskStatus.Running:
          className = 'bg-yellow-100 text-yellow-500';
          break;
        case TaskStatus.Success:
          className = 'bg-green-100 text-green-500';
          break;
        case TaskStatus.Failed:
          className = 'bg-red-100 text-red-500';
          break;
        case TaskStatus.Removed:
          className = 'bg-gray-100 text-gray-500';
          break;
        case TaskStatus.Stopped:
          className = 'bg-purple-100 text-purple-500';
          break;
        default:
          className = 'bg-gray-100 text-gray-500';
          break;
      }
      return <LongText className='text-center'><Badge variant='outline' className={className}>
        {status}
      </Badge></LongText>
    },
  },
  {
    accessorKey: 'reply',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Type' />
    ),
    cell: ({ row }) => {
      const reply = row.original.reply;
      let type;
      if (reply === undefined || reply === null) {
        type = <Badge variant='outline' className='bg-blue-100 text-blue-500'>
          New Email
        </Badge>;

      } else if (reply === true) {
        type = <Badge variant='outline' className='bg-blue-100 text-green-500'>
          Reply
        </Badge>;
      } else if (reply === false) {
        type = <Badge variant='outline' className='bg-blue-100 text-orange-500'>
          Forward
        </Badge>;
      }
      return <LongText>{type}</LongText>;
    },
  },
  {
    accessorKey: 'error',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Last Error' />
    ),
    cell: ({ row }) => {
      return <LongText className='max-w-20 text-center'>{row.original.error ?? 'n/a'}</LongText>
    },
  },
  {
    accessorKey: 'stopped_reason',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Stopped Reason' />
    ),
    cell: ({ row }) => {
      return <LongText className='text-center'>{row.original.stopped_reason ?? 'n/a'}</LongText>
    },
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
    accessorKey: 'scheduled_at',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title='Scheduled At' />
    ),
    cell: ({ row }) => {
      const scheduled_at = row.original.scheduled_at;
      const date = format(new Date(scheduled_at), 'yyyy-MM-dd HH:mm:ss');
      return <LongText className='max-w-36'>{date}</LongText>;
    },
    meta: { className: 'w-36' },
    enableHiding: false,
  },
  {
    id: 'actions',
    cell: EmailTableRowActions,
  },
]
