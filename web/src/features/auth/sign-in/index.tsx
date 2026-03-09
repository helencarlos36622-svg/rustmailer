/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import Logo from '@/assets/logo.svg'
import { UserAuthForm } from './components/user-auth-form'

export default function SignIn() {
  return (
    <div className='container relative flex h-svh flex-col items-center justify-center'>
      <div className='p-8 flex flex-col items-center'>
        <img
          src={Logo}
          className='mb-6'
          width={350}
          height={350}
          alt='RustMailer Logo'
        />
        <h2 className='mb-4 text-lg font-medium text-muted-foreground'>
          Welcome to RustMailer
        </h2>
        <div className='mx-auto flex w-full flex-col justify-center space-y-2 sm:w-[350px]'>
          <UserAuthForm />
        </div>
      </div>
    </div>
  )
}
