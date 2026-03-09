/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import {
  IconExternalLink,
  IconHelp,
  IconLayoutDashboard,
  IconLockAccess,
  IconSettings
} from '@tabler/icons-react'
import { DoorOpen, IdCard, Mailbox, MailCheck, NotepadTextDashed, Webhook } from 'lucide-react'
import { type SidebarData } from '../types'

export const sidebarData: SidebarData = {
  navGroups: [
    {
      title: 'General',
      items: [
        {
          title: 'Dashboard',
          url: '/',
          icon: IconLayoutDashboard,
        }
      ],
    },
    {
      title: 'Accounts',
      items: [
        {
          title: 'Email Accounts',
          url: '/accounts',
          icon: MailCheck,
        },
        {
          title: 'Email Viewer',
          url: '/mailboxes',
          icon: Mailbox,
        }
      ],
    },

    {
      title: 'Tasks',
      items: [
        {
          title: 'Tasks Queue',
          url: '/tasks',
          icon: IconExternalLink,
        }
      ],
    },
    {
      title: 'Hooks',
      items: [
        {
          title: 'Event Hooks',
          url: '/event-hooks',
          icon: Webhook,
        }
      ],
    },
    {
      title: 'SMTP',
      items: [
        {
          title: 'Email Templates',
          url: '/templates',
          icon: NotepadTextDashed,
        },
        {
          title: 'Mail Transfer Agents',
          url: '/mta',
          icon: DoorOpen,
        },
      ],
    },
    {
      title: 'Auth',
      items: [
        {
          title: 'OAuth2 Provider',
          url: '/oauth2',
          icon: IdCard,
        },
        {
          title: 'Access Tokens',
          url: '/access-tokens',
          icon: IconLockAccess,
        }
      ]
    },
    {
      title: 'Other',
      items: [
        {
          title: 'Settings',
          url: '/settings',
          icon: IconSettings,
        },
        {
          title: 'API Documentation',
          url: '/api-docs',
          icon: IconHelp,
        },
      ],
    },
  ],
}
