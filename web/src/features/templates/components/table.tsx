/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { useState } from 'react'
import {
  ColumnDef,
  ColumnFiltersState,
  RowData,
  SortingState,
  VisibilityState,
  flexRender,
  getCoreRowModel,
  getFacetedRowModel,
  getFacetedUniqueValues,
  getSortedRowModel,
  getFilteredRowModel,
  useReactTable,
} from '@tanstack/react-table'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { EmailTemplate } from '../data/schema'
import { DataTablePagination } from './data-table-pagination'
import { DataTableToolbar } from './data-table-toolbar'
import { useQuery } from '@tanstack/react-query'
import { list_email_templates } from '@/api/templates/api'
import { TableSkeleton } from '@/components/table-skeleton'
import { Button } from '@/components/ui/button'
import Logo from '@/assets/logo.svg'

declare module '@tanstack/react-table' {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  interface ColumnMeta<TData extends RowData, TValue> {
    className: string
  }
}

interface DataTableProps {
  columns: ColumnDef<EmailTemplate>[],
  onCallback?: () => void;
}



export function EmailTemplatesTable({ columns, onCallback }: DataTableProps) {
  const [rowSelection, setRowSelection] = useState({})
  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({})
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([])
  const [sorting, setSorting] = useState<SortingState>([])
  const [pagination, setPagination] = useState({
    pageIndex: 0, //initial page index
    pageSize: 10, //default page size
  });

  const { data: queryResult, isLoading } = useQuery({
    queryKey: ['email-templates-list', pagination.pageIndex, pagination.pageSize],
    queryFn: () => list_email_templates(pagination.pageIndex, pagination.pageSize),
  });

  const total = queryResult?.total_items || 0;

  const table = useReactTable({
    data: queryResult?.items || [],
    columns,
    state: {
      sorting,
      columnVisibility,
      rowSelection,
      columnFilters,
      pagination
    },
    enableRowSelection: true,
    onRowSelectionChange: setRowSelection,
    onSortingChange: setSorting,
    onColumnFiltersChange: setColumnFilters,
    onColumnVisibilityChange: setColumnVisibility,
    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    onPaginationChange: setPagination,
    manualPagination: true,
    rowCount: total,
    pageCount: queryResult?.total_pages || 0,
    //getPaginationRowModel: getPaginationRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFacetedRowModel: getFacetedRowModel(),
    getFacetedUniqueValues: getFacetedUniqueValues(),
    globalFilterFn: (row, _, filterValue) => {
      const searchValue = filterValue.toLowerCase();

      const checkValue = (value: any): boolean => {
        if (typeof value === 'string') {
          return value.toLowerCase().includes(searchValue);
        }
        if (typeof value === 'object' && value !== null) {
          return Object.values(value).some((nestedValue) =>
            checkValue(nestedValue)
          );
        }
        return false;
      };

      return checkValue(row.original);
    }
  })



  return isLoading ? (
    <TableSkeleton columns={columns.length} rows={10} />
  ) : total > 0 ? (
    <div className="space-y-4">
      <DataTableToolbar table={table} />
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id} className="group/row">
                {headerGroup.headers.map((header) => {
                  return (
                    <TableHead
                      key={header.id}
                      colSpan={header.colSpan}
                      className={header.column.columnDef.meta?.className ?? ''}
                    >
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                          header.column.columnDef.header,
                          header.getContext()
                        )}
                    </TableHead>
                  );
                })}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {table.getRowModel().rows?.length ? (
              table.getRowModel().rows.map((row) => (
                <TableRow
                  key={row.id}
                  data-state={row.getIsSelected() && 'selected'}
                  className="group/row"
                >
                  {row.getVisibleCells().map((cell) => (
                    <TableCell
                      key={cell.id}
                      className={cell.column.columnDef.meta?.className ?? ''}
                    >
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext()
                      )}
                    </TableCell>
                  ))}
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell
                  colSpan={columns.length}
                  className="h-24 text-center"
                >
                  No results.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
      <DataTablePagination table={table} />
    </div>
  ) : (
    <div className="flex h-[450px] shrink-0 items-center justify-center rounded-md border border-dashed">
      <div className="mx-auto flex max-w-[420px] flex-col items-center justify-center text-center">
        <img
          src={Logo}
          className="max-h-[100px] w-auto opacity-20 saturate-0 transition-all duration-300 hover:opacity-100 hover:saturate-100 object-contain"
          alt="RustMailer Logo"
        />
        <h3 className="mt-4 text-lg font-semibold">No Email Templates</h3>
        <p className="mb-4 mt-2 text-sm text-muted-foreground">
          You haven't created any email templates yet. Create one to streamline your email communications.
        </p>
        <Button onClick={onCallback}>
          Create Template
        </Button>
      </div>
    </div>
  );
}
