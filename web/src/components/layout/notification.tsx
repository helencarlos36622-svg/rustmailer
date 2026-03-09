/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { BellIcon, Loader2, ExternalLinkIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useQuery } from "@tanstack/react-query";
import { get_notifications } from "@/api/system/api";
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { useMemo } from "react";

interface Release {
  tag_name: string;
  published_at: string;
  body: string;
  html_url: string;
}

interface BaseNotification {
  type: string;
}

interface ReleaseNotification extends BaseNotification {
  type: 'new-release';
  data: Release;
}

interface LicenseExpiredNotification extends BaseNotification {
  type: 'license-expired';
  days: number;
}

interface LicenseWarningNotification extends BaseNotification {
  type: 'license-warning';
  days: number;
}

type ActiveNotification =
  | ReleaseNotification
  | LicenseExpiredNotification
  | LicenseWarningNotification;

export function NotificationPopover() {
  const { data, isLoading } = useQuery({
    queryKey: ['system-notifications'],
    queryFn: get_notifications,
    staleTime: 1000 * 60 * 30, // 30 minutes
  });

  const activeNotifications = useMemo((): ActiveNotification[] => {
    if (!data) return [];

    const notifications: ActiveNotification[] = [];
    if (data.license.expired) {
      notifications.push({
        type: 'license-expired',
        days: data.license.days
      });
    } else if (data.license.days <= 30) {
      notifications.push({
        type: 'license-warning',
        days: data.license.days
      });
    }

    if (data.release.is_newer && data.release.latest) {
      notifications.push({
        type: 'new-release',
        data: {
          tag_name: data.release.latest.tag_name,
          published_at: data.release.latest.published_at,
          body: data.release.latest.body,
          html_url: data.release.latest.html_url
        }
      });
    }
    return notifications;
  }, [data]);

  const showNotificationBadge = activeNotifications.length > 0;

  const hasCriticalNotification = activeNotifications.some(
    n => n.type === 'license-expired'
  );

  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          className="relative"
          disabled={isLoading}
        >
          {isLoading ? (
            <Loader2 className="h-5 w-5 animate-spin" />
          ) : (
            <>
              <BellIcon className="h-5 w-5" />
              {showNotificationBadge && (
                <Badge
                  variant={hasCriticalNotification ? "destructive" : "default"}
                  className="absolute -right-1 -top-1 h-5 w-5 rounded-full p-0 flex items-center justify-center"
                >
                  {activeNotifications.length}
                </Badge>
              )}
            </>
          )}
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[32rem] p-0" align="end">
        <div className="p-4 border-b">
          <h4 className="font-medium">
            System Notifications
            {showNotificationBadge && ` (${activeNotifications.length})`}
          </h4>
        </div>
        <ScrollArea className="h-72">
          {isLoading ? (
            <div className="flex items-center justify-center p-8">
              <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : activeNotifications.length === 0 ? (
            <div className="p-8 text-center space-y-2">
              <BellIcon className="mx-auto h-6 w-6 text-muted-foreground" />
              <p className="text-sm text-muted-foreground">
                No new notifications
              </p>
            </div>
          ) : (
            <div className="divide-y">
              {activeNotifications.map((notification, index) => (
                <div key={index} className="p-4">
                  {notification.type === 'license-expired' ? (
                    <LicenseExpiredNotificationView days={notification.days} />
                  ) : notification.type === 'license-warning' ? (
                    <LicenseWarningNotificationView days={notification.days} />
                  ) : (
                    <ReleaseNotificationView data={notification.data} />
                  )}
                </div>
              ))}
            </div>
          )}
        </ScrollArea>
      </PopoverContent>
    </Popover>
  );
}

function LicenseExpiredNotificationView({ days }: { days: number }) {
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-destructive">
          License Expired
        </h3>
        <span className="text-xs bg-red-100 text-red-800 px-2 py-1 rounded-full">
          Critical
        </span>
      </div>
      <p className="text-sm">
        Your license expired {days} days ago. Please renew immediately.
      </p>
    </div>
  );
}

function LicenseWarningNotificationView({ days }: { days: number }) {
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-yellow-600">
          License Expiring Soon
        </h3>
        <span className="text-xs bg-yellow-100 text-yellow-800 px-2 py-1 rounded-full">
          Warning
        </span>
      </div>
      <p className="text-xs">
        Your license will expire in {days} days.
      </p>
    </div>
  );
}

function ReleaseNotificationView({ data }: { data: Release }) {
  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold">
            {data.tag_name}
          </h3>
          <span className="text-xs bg-green-100 text-green-800 px-2 py-1 rounded-full">
            New Release
          </span>
        </div>
        <p className="text-xs text-muted-foreground">
          Released {data.published_at}
        </p>
      </div>

      <div className="prose prose-xs dark:prose-invert max-w-none">
        <ReactMarkdown remarkPlugins={[remarkGfm]}>
          {data.body}
        </ReactMarkdown>
      </div>

      {data.html_url && (
        <div className="pt-2">
          <a
            href={data.html_url}
            target="_blank"
            rel="noopener noreferrer"
            className="text-sm text-primary hover:underline inline-flex items-center"
          >
            View full release notes <ExternalLinkIcon className="ml-1 h-3 w-3" />
          </a>
        </div>
      )}
    </div>
  );
}