/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import {
  ChevronLeftIcon,
  ChevronRightIcon,
} from '@radix-ui/react-icons'
import { Button } from '@/components/ui/button'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'

interface PaginationProps {
  totalItems: number
  pageIndex: number,
  pageSize: number,
  hasNextPage: () => boolean,
  setPageIndex: (pageIndex: number) => void,
  setPageSize: (pageSize: number) => void,
}

export function EnvelopeListPagination({
  totalItems,
  pageIndex,
  pageSize,
  hasNextPage,
  setPageIndex,
  setPageSize,
}: PaginationProps) {
  const pageCount = Math.ceil(totalItems / pageSize)

  const handlePageSizeChange = (value: string) => {
    const newPageSize = Number(value)
    setPageSize(newPageSize)
    setPageIndex(0) // Reset to the first page when page size changes
  }

  // const goToFirstPage = () => {
  //   setPageIndex(0)
  // }

  const goToPreviousPage = () => {
    const newPageIndex = Math.max(pageIndex - 1, 0)
    setPageIndex(newPageIndex)
  }

  const goToNextPage = () => {
    const newPageIndex = Math.min(pageIndex + 1, pageCount - 1)
    setPageIndex(newPageIndex)
  }

  // const goToLastPage = () => {
  //   const newPageIndex = pageCount - 1
  //   setPageIndex(newPageIndex)
  // }

  return (
    <div className='flex items-center justify-between space-x-2 overflow-auto px-2'>
      <div className='hidden flex-1 text-sm text-muted-foreground sm:block'>
        {totalItems} total items
      </div>
      <div className='flex items-center sm:space-x-6 lg:space-x-4'>
        <div className='flex items-center space-x-2'>
          <p className='hidden text-sm font-medium sm:block'>Rows per page</p>
          <Select
            value={`${pageSize}`}
            onValueChange={handlePageSizeChange}
          >
            <SelectTrigger className='h-8 w-[70px]'>
              <SelectValue placeholder={pageSize} />
            </SelectTrigger>
            <SelectContent side='top'>
              {[10, 20, 30, 40, 50].map((size) => (
                <SelectItem key={size} value={`${size}`}>
                  {size}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className='flex items-center justify-center text-sm font-medium'>
          Page {pageIndex + 1} of {pageCount}
        </div>
        <div className='flex items-center space-x-2'>
          {/* <Button
            variant='outline'
            className='hidden h-8 w-8 p-0 lg:flex'
            onClick={goToFirstPage}
            disabled={pageIndex === 0}
          >
            <span className='sr-only'>Go to first page</span>
            <DoubleArrowLeftIcon className='h-4 w-4' />
          </Button> */}
          <Button
            variant='outline'
            className='h-8 w-8 p-0'
            onClick={goToPreviousPage}
            disabled={pageIndex === 0}
          >
            <span className='sr-only'>Go to previous page</span>
            <ChevronLeftIcon className='h-4 w-4' />
          </Button>
          <Button
            variant='outline'
            className='h-8 w-8 p-0'
            onClick={goToNextPage}
            disabled={!hasNextPage()}
          >
            <span className='sr-only'>Go to next page</span>
            <ChevronRightIcon className='h-4 w-4' />
          </Button>
          {/* <Button
            variant='outline'
            className='hidden h-8 w-8 p-0 lg:flex'
            onClick={goToLastPage}
            disabled={pageIndex === pageCount - 1}
          >
            <span className='sr-only'>Go to last page</span>
            <DoubleArrowRightIcon className='h-4 w-4' />
          </Button> */}
        </div>
      </div>
    </div>
  )
}