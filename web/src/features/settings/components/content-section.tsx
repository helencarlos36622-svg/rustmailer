/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'

interface ContentSectionProps {
  title: string
  desc: string
  children: React.JSX.Element,
  showHeader?: boolean
}

export default function ContentSection({
  title,
  desc,
  children,
  showHeader = true
}: ContentSectionProps) {
  return (
    <div className='flex flex-1 flex-col'>
      {showHeader && <div className='flex-none'>
        <h3 className='text-lg font-medium'>{title}</h3>
        <p className='text-sm text-muted-foreground'>{desc}</p>
      </div>}

      {showHeader && <Separator className='my-4 flex-none' />}

      <ScrollArea className='faded-bottom -mx-4 flex-1 scroll-smooth px-4 md:pb-16'>
        <div className='lg:max-w-2xl -mx-1 px-1.5'>{children}</div>
      </ScrollArea>
    </div>
  )
}
