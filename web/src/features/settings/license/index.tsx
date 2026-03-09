/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import ContentSection from '@/features/settings/components/content-section'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { BellRing, Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Table, TableBody, TableCell, TableRow } from '@/components/ui/table'
import { useCallback, useMemo, useState } from 'react'
import { useIsMobile } from '@/hooks/use-mobile'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
import { Drawer, DrawerClose, DrawerContent, DrawerDescription, DrawerFooter, DrawerHeader, DrawerTitle, DrawerTrigger } from '@/components/ui/drawer'
import { cn } from '@/lib/utils'
import { z } from 'zod'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { Form, FormControl, FormDescription, FormField, FormItem, FormMessage } from '@/components/ui/form'
import { Textarea } from '@/components/ui/textarea'
import { toast } from '@/hooks/use-toast'
import { LicenseImportDialog } from './import-license'
import { IconDownload } from '@tabler/icons-react'
import { get_license, set_license } from '@/api/license/api'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Skeleton } from '@/components/ui/skeleton'
import { AxiosError } from 'axios'
import { ToastAction } from '@/components/ui/toast'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import LongText from '@/components/long-text'

function formatDate(createdAt?: number): string {
  return createdAt
    ? new Date(createdAt).toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' })
    : 'n/a';
}

interface TimeData {
  remaining: number;
}

function getTimeData(endDate: number): TimeData {
  const currentTime = Date.now();
  const remainingTime = endDate - currentTime;
  const remaining = Math.floor(remainingTime / (1000 * 60 * 60 * 24));
  return { remaining: remaining > 0 ? remaining : 0 };
}

export default function License() {
  const [open, setOpen] = useState(false);
  const isMobile = useIsMobile();

  const { data, isLoading } = useQuery({
    queryKey: ['license'],
    queryFn: get_license,
  });

  const timeData = useMemo<TimeData | null>(() => {
    if (data?.expires_at) {
      return getTimeData(data.expires_at);
    }
    return null;
  }, [data?.expires_at]);

  return (
    <ContentSection
      title="License"
      desc="Manage your license details."
      showHeader={false}
    >
      <Card className="overflow-hidden">
        <CardHeader className="items-center pb-0">
          <CardTitle className="text-2xl font-semibold">
            {isLoading ? <Skeleton className="h-8 w-48" /> : `${data?.license_type} License`}
          </CardTitle>
          <CardDescription>
            {isLoading ? <Skeleton className="h-4 w-64" /> : data?.license_type === "Trial"
              ? "14-days evaluation period"
              : "Annual subscription"}
          </CardDescription>
        </CardHeader>
        <CardContent className="p-6 space-y-6">
          <div className="text-center">
            {isLoading ? (
              <Skeleton className="h-10 w-40 mx-auto" />
            ) : (
              <div>
                <p className="text-sm text-muted-foreground">Days Remaining</p>
                <p className="text-4xl font-bold">{timeData?.remaining.toLocaleString()}</p>
              </div>
            )}
          </div>

          <div className="text-center">
            {isLoading ? (
              <Skeleton className="h-10 w-40 mx-auto" />
            ) : (
              !isMobile ? (
                <Dialog open={open} onOpenChange={setOpen}>
                  <DialogTrigger asChild>
                    <Button>Update License</Button>
                  </DialogTrigger>
                  <DialogContent className="sm:max-w-2xl">
                    <DialogHeader>
                      <DialogTitle>Update License</DialogTitle>
                      <DialogDescription>Enter your license key or upload a file.</DialogDescription>
                    </DialogHeader>
                    <UploadLicenseForm close={() => setOpen(false)} />
                  </DialogContent>
                </Dialog>
              ) : (
                <Drawer open={open} onOpenChange={setOpen}>
                  <DrawerTrigger asChild>
                    <Button>Update License</Button>
                  </DrawerTrigger>
                  <DrawerContent>
                    <DrawerHeader className="text-left">
                      <DrawerTitle>Update License</DrawerTitle>
                      <DrawerDescription>Enter your license key or upload a file.</DrawerDescription>
                    </DrawerHeader>
                    <UploadLicenseForm className="px-4" close={() => setOpen(false)} />
                    <DrawerFooter>
                      <DrawerClose asChild>
                        <Button variant="outline">Cancel</Button>
                      </DrawerClose>
                    </DrawerFooter>
                  </DrawerContent>
                </Drawer>
              )
            )}
          </div>

          {isLoading ? (
            <Skeleton className="h-12 w-full" />
          ) : (
            timeData?.remaining === 0 ? (
              <Alert>
                <BellRing className="h-4 w-4" />
                <AlertTitle>License Expired</AlertTitle>
                <AlertDescription>
                  Your license expired on {formatDate(data?.expires_at)}. Please{" "}
                  <a href="/subscribe" className="underline">renew your subscription</a>.
                </AlertDescription>
              </Alert>
            ) : timeData?.remaining && timeData?.remaining <= 30 ? (
              <Alert>
                <BellRing className="h-4 w-4" />
                <AlertTitle>{data?.license_type === 'Trial' ? "Trial Expiring Soon" : "License Expiring Soon"}</AlertTitle>
                <AlertDescription>
                  {data?.license_type === 'Trial' ? (
                    <>
                      Your <strong>trial</strong> ends in <strong>{timeData.remaining} days</strong>.
                      To keep using RustMailer, please&nbsp;
                      <a
                        href="https://rustmailer.com/pricing"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="underline text-primary font-medium hover:text-primary/80 transition-colors"
                      >
                        choose a plan
                      </a>.
                    </>
                  ) : (
                    <>
                      Your license will expire in <strong>{timeData.remaining} days</strong>.
                      To avoid interruption, please&nbsp;
                      <a
                        href="https://rustmailer.com/pricing"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="underline text-primary font-medium hover:text-primary/80 transition-colors"
                      >
                        renew your plan
                      </a>.
                    </>
                  )}
                </AlertDescription>
              </Alert>
            ) : null
          )}

          <Table className="text-sm">
            <TableBody>
              {isLoading ? (
                <>
                  <TableRow><TableCell><Skeleton className="h-4 w-24" /></TableCell><TableCell><Skeleton className="h-4 w-48" /></TableCell></TableRow>
                  <TableRow><TableCell><Skeleton className="h-4 w-24" /></TableCell><TableCell><Skeleton className="h-4 w-48" /></TableCell></TableRow>
                  <TableRow><TableCell><Skeleton className="h-4 w-24" /></TableCell><TableCell><Skeleton className="h-4 w-48" /></TableCell></TableRow>
                </>
              ) : (
                <>
                  {data?.license_type != 'Trial' && (
                    <>
                      <TableRow>
                        <TableCell className="font-medium text-muted-foreground">Application</TableCell>
                        <TableCell><LongText className="max-w-64">{data?.application_name ?? "n/a"}</LongText></TableCell>
                      </TableRow>
                      <TableRow>
                        <TableCell className="font-medium text-muted-foreground">Licensed To</TableCell>
                        <TableCell><LongText className="max-w-64">{data?.customer_name ?? "n/a"}</LongText></TableCell>
                      </TableRow>
                      <TableRow>
                        <TableCell className="font-medium text-muted-foreground">Max Accounts</TableCell>
                        <TableCell>
                          <LongText className="max-w-64">
                            {data?.max_accounts ? data.max_accounts : "Unlimited"}
                          </LongText>
                        </TableCell>
                      </TableRow>
                    </>
                  )}
                  <TableRow>
                    <TableCell className="font-medium text-muted-foreground">Start Date</TableCell>
                    <TableCell>{formatDate(data?.created_at)}</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell className="font-medium text-muted-foreground">Expiration</TableCell>
                    <TableCell>{formatDate(data?.expires_at)}</TableCell>
                  </TableRow>
                </>
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </ContentSection>
  );
}

const FormSchema = z.object({
  license: z
    .string({ required_error: 'License key is required.' })
    .min(300, { message: "License key is too short. Please check your license." })
    .max(800, { message: "License key is too long. Please check your license." }),
});


function UploadLicenseForm({ className, close }: React.ComponentProps<"form"> & { close: () => void }) {
  const [open, setOpen] = useState(false);
  const mutation = useMutation({
    mutationFn: (licenseKey: string) => set_license(licenseKey),
    retry: 0,
  });
  const queryClient = useQueryClient();

  const form = useForm<z.infer<typeof FormSchema>>({
    resolver: zodResolver(FormSchema),
  });

  const fillForm = useCallback((licenseContent: string) => {
    form.setValue('license', licenseContent);
    form.trigger("license");
  }, [form]);

  function onSubmit(data: z.infer<typeof FormSchema>) {
    mutation.mutate(data.license, {
      onSuccess: (data) => {
        close();
        queryClient.setQueryData(['license'], data);
        toast({
          title: "License Updated",
          description: "Your license has been successfully updated.",
          action: <ToastAction altText="Close">Close</ToastAction>,
        });
      },
      onError: (error) => {
        if (error instanceof AxiosError && error.response?.status === 400) {
          toast({
            variant: "destructive",
            title: "Invalid License",
            description: "The provided license key is invalid. Please verify and try again.",
            action: <ToastAction altText="Try again">Try again</ToastAction>,
          });
        }
      },
    });
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className={cn("space-y-4", className)}>
        <FormField
          control={form.control}
          name="license"
          render={({ field }) => (
            <FormItem>
              <FormControl>
                <Textarea
                  className="min-h-[200px] resize-none"
                  placeholder="Paste your license key here..."
                  {...field}
                />
              </FormControl>
              <FormDescription className="flex items-center justify-between text-xs text-muted-foreground">
                <span>Or upload a license file:</span>
                <Button
                  variant="outline"
                  size="sm"
                  className="space-x-1"
                  onClick={() => setOpen(true)}
                >
                  <span>Import</span>
                  <IconDownload size={16} />
                </Button>
                <LicenseImportDialog key="license-import" open={open} onOpenChange={setOpen} onRead={fillForm} />
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        <Button
          type="submit"
          className="w-full relative"
          disabled={mutation.isPending}
        >
          {mutation.isPending ? (
            <span className="flex items-center justify-center">
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              <span className="opacity-0">Submit License</span>
              <span className="absolute">Updating...</span>
            </span>
          ) : (
            "Submit License"
          )}
        </Button>
      </form>
    </Form>
  );
}