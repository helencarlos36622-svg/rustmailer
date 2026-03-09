/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { Skeleton } from "@/components/ui/skeleton"
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table"

interface TableSkeletonProps {
    columns?: number
    rows?: number
    showPagination?: boolean
}

export function TableSkeleton({ columns = 5, rows = 5, showPagination = true }: TableSkeletonProps) {
    const getColumnWidth = (index: number): string => {
        if (index === columns - 1) return "100px"
        const widths = ["100px", "200px", "150px", "120px", "180px"]
        return widths[index % widths.length]
    }

    return (
        <div className="space-y-4">
            <Table>
                <TableHeader>
                    <TableRow>
                        {Array.from({ length: columns }).map((_, index) => (
                            <TableHead key={index}>
                                {index === columns - 1 ? (
                                    <div className="w-[100px]" />
                                ) : (
                                    <Skeleton className={`h-4 w-[${getColumnWidth(index)}]`} />
                                )}
                            </TableHead>
                        ))}
                    </TableRow>
                </TableHeader>
                <TableBody>
                    {Array.from({ length: rows }).map((_, rowIndex) => (
                        <TableRow key={rowIndex}>
                            {Array.from({ length: columns }).map((_, colIndex) => (
                                <TableCell key={colIndex}>
                                    {colIndex === columns - 1 ? (
                                        <div className="flex space-x-2">
                                            <Skeleton className="h-8 w-8" />
                                            <Skeleton className="h-8 w-8" />
                                        </div>
                                    ) : (
                                        <Skeleton className={`h-4 w-[${getColumnWidth(colIndex)}]`} />
                                    )}
                                </TableCell>
                            ))}
                        </TableRow>
                    ))}
                </TableBody>
            </Table>

            {showPagination && (
                <div className="flex items-center justify-end px-2">
                    <div className="flex items-center space-x-6">
                        <Skeleton className="h-8 w-[100px]" />
                        <Skeleton className="h-4 w-[100px]" />
                        <div className="flex items-center space-x-2">
                            <Skeleton className="h-8 w-8" />
                            <Skeleton className="h-8 w-8" />
                            <Skeleton className="h-8 w-8" />
                        </div>
                    </div>
                </div>
            )}
        </div>
    )
}