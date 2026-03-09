/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import React from 'react'
import { cn } from '@/lib/utils'

interface MainProps extends React.HTMLAttributes<HTMLElement> {
  fixed?: boolean;
  higher?: boolean;
  ref?: React.Ref<HTMLElement>;
}

export const Main = ({ fixed, higher, ...props }: MainProps) => {
  return (
    <main
      className={cn(
        'peer-[.header-fixed]/header',
        higher ? 'mt-12' : 'mt-16', // Conditional class for 'higher'
        'px-4 py-6',
        fixed && 'fixed-main flex flex-col flex-grow overflow-hidden'
      )}
      {...props}
    />
  );
};

Main.displayName = 'Main';

