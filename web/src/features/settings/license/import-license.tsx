/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { z } from 'zod'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { Button } from '@/components/ui/button'
import {
    Dialog,
    DialogClose,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import {
    Form,
    FormControl,
    FormField,
    FormItem,
    FormMessage,
} from '@/components/ui/form'
import { Input } from '@/components/ui/input'

const formSchema = z.object({
    file: z
        .instanceof(FileList)
        .refine((files) => files.length === 1, {
            message: 'Please upload exactly one text file',
        })
        .refine(
            (files) => {
                const file_size = files?.[0]?.size;
                return file_size > 100 && file_size < 544400
            },
            {
                message: 'The license file is approximately 400 characters in size. The file you uploaded may not be a valid license file',
            }
        ),
});


interface Props {
    open: boolean
    onOpenChange: (open: boolean) => void
    onRead: (content: string) => void
}

export function LicenseImportDialog({ open, onOpenChange, onRead }: Props) {
    const form = useForm<z.infer<typeof formSchema>>({
        resolver: zodResolver(formSchema),
        defaultValues: { file: undefined },
    })

    const fileRef = form.register('file')

    const onSubmit = () => {
        const file = form.getValues('file')

        if (file && file[0]) {
            const reader = new FileReader();
            reader.onload = (e) => {
                const fileContent = e.target?.result;
                if (typeof fileContent === 'string') {
                    onRead(fileContent);
                } else {
                    console.error(`File content is not a string. File name: ${file[0].name}, File type: ${file[0].type}`);

                }
            };
            reader.readAsText(file[0]);
            onOpenChange(false)
        }
    }

    return (
        <Dialog
            open={open}
            onOpenChange={(val) => {
                onOpenChange(val)
                form.reset()
            }}
        >
            <DialogContent className='sm:max-w-sm gap-2'>
                <DialogHeader>
                    <DialogTitle>Import License</DialogTitle>
                    <DialogDescription>
                        Import license quickly from a file.
                    </DialogDescription>
                </DialogHeader>
                <Form {...form}>
                    <form id='license-import-form' onSubmit={form.handleSubmit(onSubmit)}>
                        <FormField
                            control={form.control}
                            name='file'
                            render={() => (
                                <FormItem className='space-y-1 mb-2'>
                                    <FormControl>
                                        <Input type='file' {...fileRef} className='h-8' />
                                    </FormControl>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />
                    </form>
                </Form>
                <DialogFooter>
                    <DialogClose asChild>
                        <Button variant='outline' className="px-2 py-1 text-sm h-auto">Close</Button>
                    </DialogClose>
                    <Button type='submit' form='license-import-form' className="px-2 py-1 text-sm h-auto">
                        Import
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
