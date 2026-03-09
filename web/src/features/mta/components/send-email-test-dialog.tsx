/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { z } from 'zod';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { toast } from '@/hooks/use-toast';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';
import { MTARecord } from '../data/schema';
import { Textarea } from '@/components/ui/textarea';
import { useMutation } from '@tanstack/react-query';
import { send_test_email } from '@/api/mta/api';

const sendTestFormSchema = z.object({
  from: z
    .string({ required_error: "From email address is required." })
    .min(1, { message: "From email address cannot be empty." })
    .email({ message: "Please enter a single valid email address (e.g., no-reply@yourdomain.com)." }),
  to: z
    .string({ required_error: "To email address is required." })
    .min(1, { message: "To email address cannot be empty." })
    .email({ message: "Please enter a single valid email address (e.g., example@domain.com)." }),
  subject: z
    .string({ required_error: "Email subject is required." })
    .min(1, { message: "Email subject cannot be empty." }),
  message: z
    .string({ required_error: "Email message is required." })
    .min(1, { message: "Email message cannot be empty." }),
});

export type SendTestEmailForm = z.infer<typeof sendTestFormSchema>;

interface Props {
  currentRow: MTARecord;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function SentTestEmailDialog({ currentRow, open, onOpenChange }: Props) {
  const form = useForm<SendTestEmailForm>({
    resolver: zodResolver(sendTestFormSchema),
    defaultValues: {
      from: "",
      to: "",
      subject: `Test Email for MTA: ${currentRow.id}`,
      message: `This is a test email sent via the MTA configuration below:
**MTA Id:**    ${currentRow.id}
**Host:**        ${currentRow.server.host}
**Port:**        ${currentRow.server.port}
**Encryption:**  ${currentRow.server.encryption}
Please verify if the email is received successfully.`,
    },
  });

  const mutation = useMutation({
    mutationFn: (values: SendTestEmailForm) => send_test_email(currentRow.id, values),
    onSuccess: () => {
      toast({
        title: "Test Email Sent",
        description: "The test email was successfully sent. Please check the recipient's inbox.",
      });
      form.reset({
        from: "",
        to: "",
        subject: `Test Email for MTA: ${currentRow.id}`,
        message: `This is a test email sent via the MTA configuration below:
**MTA Id:**    ${currentRow.id}
**Host:**        ${currentRow.server.host}
**Port:**        ${currentRow.server.port}
**Encryption:**  ${currentRow.server.encryption}
Please verify if the email is received successfully.`,
      });
      onOpenChange(false);
    },
    onError: (error: any) => {
      toast({
        title: "Failed to Send Test Email",
        description: error.response?.data?.message || "An error occurred while sending the test email.",
        variant: "destructive",
      });
    },
  });

  const onSubmit = (values: SendTestEmailForm) => {
    mutation.mutate(values);
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(state) => {
        form.reset({
          from: "",
          to: "",
          subject: `Test Email for MTA: ${currentRow.id}`,
          message: `        This is a test email sent via the MTA configuration below:
                      **MTA Name:**    ${currentRow.id}
                      **Host:**        ${currentRow.server.host}
                      **Port:**        ${currentRow.server.port}
                      **Encryption:**  ${currentRow.server.encryption}

                      Please verify if the email is received successfully.`
        });
        onOpenChange(state);
      }}
    >
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader className="text-left">
          <DialogTitle>Send Test Email</DialogTitle>
          <DialogDescription>
            Send a test email to verify the MTA configuration is correct.
          </DialogDescription>
        </DialogHeader>
        <ScrollArea className="h-[26.5rem] w-full pr-4 -mr-4 py-1">
          <Form {...form}>
            <form
              id="send-email-test-form"
              onSubmit={form.handleSubmit(onSubmit)}
              className="space-y-4 p-0.5"
            >
              <div className="space-y-6">
                <FormField
                  control={form.control}
                  name="from"
                  render={({ field }) => (
                    <FormItem className="flex flex-col gap-y-1 space-y-0">
                      <FormLabel className="mb-1">From Email Address:</FormLabel>
                      <FormControl>
                        <Input
                          placeholder="Enter the sender email address (e.g., no-reply@yourdomain.com)"
                          {...field}
                        />
                      </FormControl>
                      <FormDescription>
                        The email address that will appear as the sender of the test email.
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name="to"
                  render={({ field }) => (
                    <FormItem className="flex flex-col gap-y-1 space-y-0">
                      <FormLabel className="mb-1">Recipient Email Address:</FormLabel>
                      <FormControl>
                        <Input
                          placeholder="Enter the recipient email address (e.g., example@domain.com)"
                          {...field}
                        />
                      </FormControl>
                      <FormDescription>
                        The email address where the test email will be sent.
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name="subject"
                  render={({ field }) => (
                    <FormItem className="flex flex-col gap-y-1 space-y-0">
                      <FormLabel className="mb-1">Email Subject:</FormLabel>
                      <FormControl>
                        <Input
                          placeholder="Enter the email subject (e.g., Test Email)"
                          {...field}
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name="message"
                  render={({ field }) => (
                    <FormItem className="flex flex-col gap-y-1 space-y-0">
                      <FormLabel className="mb-1">Email Message:</FormLabel>
                      <FormControl>
                        <Textarea
                          className="max-h-[240px] min-h-[180px]"
                          placeholder="Enter the email message content (e.g., This is a test email.)"
                          {...field}
                        />
                      </FormControl>
                      <FormDescription>
                        The plain text content that will be sent as the body of the test email.
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />
              </div>
            </form>
          </Form>
        </ScrollArea>
        <DialogFooter>
          <DialogClose asChild>
            <Button variant="outline" className="px-2 py-1 text-sm h-auto">
              Close
            </Button>
          </DialogClose>
          <Button
            type="submit"
            form="send-email-test-form"
            disabled={mutation.isPending}
          >
            {mutation.isPending ? "Sending..." : "Send"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}