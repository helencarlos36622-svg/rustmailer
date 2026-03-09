/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { AccountEntity } from '../data/schema'
import { Button } from '@/components/ui/button'
import { get_oauth2_tokens } from '@/api/oauth2/api'
import { useQuery } from '@tanstack/react-query'
import { Card, CardContent } from '@/components/ui/card'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { TableSkeleton } from '@/components/table-skeleton'
import { FileIcon } from 'lucide-react'
import { format, formatDistanceToNow } from 'date-fns'
import LongText from '@/components/long-text'
import { useCallback } from 'react'
import { IconCopy } from '@tabler/icons-react'
import { toast } from '@/hooks/use-toast'
import { ToastAction } from '@/components/ui/toast'
import { useNavigate } from '@tanstack/react-router'

interface Props {
  currentRow: AccountEntity
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function OAuth2TokensDialog({ currentRow, open, onOpenChange }: Props) {
  const navigate = useNavigate()
  const { data: oauth2Tokens, isLoading } = useQuery({
    queryKey: ['oauth2-tokens', currentRow.id],
    queryFn: () => get_oauth2_tokens(currentRow.id),
    enabled: open && !!currentRow.id,
    retry: 0,
    refetchOnWindowFocus: false,
    refetchOnMount: false,
  })


  const onCopy = useCallback(async (access: boolean, token: string) => {
    try {
      await navigator.clipboard.writeText(token);
      if (access) {
        toast({
          title: "Success",
          description: "Access token copied to clipboard",
        });
      } else {
        toast({
          title: "Success",
          description: "Refresh token copied to clipboard",
        });
      }
    } catch (err) {
      toast({
        variant: "destructive",
        title: "Failed to copy text",
        description: (err as Error).message,
        action: <ToastAction altText="Try again">Try again</ToastAction>,
      });
    }
  }, []);

  return (
    <Dialog
      open={open}
      onOpenChange={(state) => {
        onOpenChange(state)
      }}
    >
      <DialogContent className='sm:max-w-3xl'>
        <DialogHeader className='text-left'>
          <DialogTitle>OAuth2 Tokens</DialogTitle>
          <DialogDescription>
            Details of the OAuth2 tokens for the account.
          </DialogDescription>
        </DialogHeader>
        <Card>
          <CardContent>
            {isLoading ? (
              <TableSkeleton columns={2} rows={10} />
            ) : oauth2Tokens ? (
              <Table className='w-full'>
                <TableHeader>
                  <TableRow>
                    <TableHead>Field</TableHead>
                    <TableHead>Value</TableHead>
                    <TableHead>Action</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow>
                    <TableCell className='max-w-80'>OAuth2 Name</TableCell>
                    <TableCell>
                      <LongText className='max-w-[240px] sm:max-w-[430px]'>{oauth2Tokens.oauth2_name}</LongText>
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell className='max-w-80'>Access Token</TableCell>
                    <TableCell>
                      <LongText className='max-w-[240px] sm:max-w-[430px]'>{oauth2Tokens.access_token}</LongText>
                    </TableCell>
                    <TableCell>
                      <Button className='text-xs px-1.5 py-0.5' onClick={() => onCopy(true, oauth2Tokens.access_token)}>
                        <IconCopy className="h-5 w-5" aria-hidden="true" />
                      </Button>
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell className='max-w-80'>Refresh Token</TableCell>
                    <TableCell>
                      <LongText className='max-w-[240px] sm:max-w-[430px]'>{oauth2Tokens.refresh_token}</LongText>
                    </TableCell>
                    <TableCell>
                      <Button className='text-xs px-1.5 py-0.5' onClick={() => onCopy(false, oauth2Tokens.refresh_token)}>
                        <IconCopy className="h-5 w-5" aria-hidden="true" />
                      </Button>
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell className='max-w-80'>Created At</TableCell>
                    <TableCell>
                      {format(new Date(oauth2Tokens.created_at), 'yyyy-MM-dd HH:mm:ss')}
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell className='max-w-80'>Updated At</TableCell>
                    <TableCell>
                      {formatDistanceToNow(new Date(oauth2Tokens.updated_at), { addSuffix: true })}
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            ) : (
              <div className="flex h-[250px] mt-4 shrink-0 items-center justify-center rounded-md border border-dashed">
                <div className="mx-auto flex max-w-[420px] flex-col items-center justify-center text-center">
                  <FileIcon className="h-10 w-10 text-muted-foreground" />
                  <h3 className="mt-4 text-lg font-semibold">No OAuth2 Tokens</h3>
                  <p className="mb-4 mt-2 text-sm text-muted-foreground">
                    The account has not completed the authorization process. Please
                    <a onClick={() => navigate({ to: '/oauth2' })} className="ml-1 text-blue-500 underline cursor-pointer">click here</a> to authorize the account.
                  </p>
                </div>
              </div>
            )}
          </CardContent>
        </Card>
        <DialogFooter>
          <DialogClose asChild>
            <Button variant='outline' className="px-2 py-1 text-sm h-auto">Close</Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
