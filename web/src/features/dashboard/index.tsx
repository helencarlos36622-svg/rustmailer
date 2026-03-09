/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Main } from '@/components/layout/main';
import { FixedHeader } from '@/components/layout/fixed-header';
import { useQuery } from '@tanstack/react-query';
import { Skeleton } from '@/components/ui/skeleton';
import { get_overview, Overview, TimeSeriesPoint } from '@/api/system/api';
import { AreaChart, Area, XAxis, CartesianGrid, YAxis } from 'recharts';
import { format } from 'date-fns';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { ChevronsUpDown } from 'lucide-react';
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  ChartConfig,
} from '@/components/ui/chart';

export default function Dashboard() {
  const { data, isLoading, error, refetch } = useQuery<Overview>({
    queryKey: ['dashboard-overview'],
    queryFn: get_overview,
    refetchInterval: 300000,
  });

  if (error) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center space-y-4">
          <h2 className="text-xl font-semibold">Failed to load dashboard data</h2>
          <p className="text-muted-foreground">{(error as Error).message}</p>
          <Button onClick={() => refetch()}>Retry</Button>
        </div>
      </div>
    );
  }

  return (
    <>
      <FixedHeader />
      <Main>
        <div className="mb-2 flex items-center justify-between space-y-2">
          <h1 className="text-2xl font-bold tracking-tight">Dashboard</h1>
          <div className="flex items-center space-x-2">
            <Button onClick={() => refetch()} disabled={isLoading}>
              {isLoading ? 'Refreshing...' : 'Refresh'}
            </Button>
          </div>
        </div>

        {isLoading ? (
          <DashboardSkeleton />
        ) : (
          <div className="mt-8 space-y-4">
            {/* Overview cards */}
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              <MetricCard
                title="Pending Email Tasks"
                value={data?.pending_email_task_num || 0}
                icon="mail"
                description="Number of queued email tasks"
              />
              <MetricCard
                title="Pending Hook Tasks"
                value={data?.pending_hook_task_num || 0}
                icon="hook"
                description="Number of pending event hooks"
              />
              <MetricCard
                title="Total Accounts"
                value={data?.account_num || 0}
                icon="users"
                description="Number of email accounts managed"
              />
              <MetricCard
                title="Uptime"
                value={data?.uptime || 0}
                icon="clock"
                description={`Version: ${data?.rustmailer_version || 'unknown'}`}
                isUptime={true}
              />
            </div>

            {/* Collapsible Chart Sections */}
            <Collapsible defaultOpen={true}>
              <CollapsibleTrigger className="flex items-center justify-between w-full p-2 bg-muted rounded-md">
                <h2 className="text-lg font-semibold">IMAP Traffic</h2>
                <ChevronsUpDown className="h-4 w-4" />
              </CollapsibleTrigger>
              <CollapsibleContent className="grid grid-cols-1 gap-4 lg:grid-cols-2 pt-4">
                <ChartCard
                  title="IMAP Traffic Sent"
                  data={data?.time_series.imap_traffic_sent}
                  dataKey="imap_traffic_sent"
                  color="var(--chart-1)"
                  formatUnit="bytes"
                />
                <ChartCard
                  title="IMAP Traffic Received"
                  data={data?.time_series.imap_traffic_received}
                  dataKey="imap_traffic_received"
                  color="var(--chart-1)"
                  formatUnit="bytes"
                />
              </CollapsibleContent>
            </Collapsible>
            <Collapsible defaultOpen={true}>
              <CollapsibleTrigger className="flex items-center justify-between w-full p-2 bg-muted rounded-md">
                <h2 className="text-lg font-semibold">Email Activity</h2>
                <ChevronsUpDown className="h-4 w-4" />
              </CollapsibleTrigger>
              <CollapsibleContent className="grid grid-cols-1 gap-4 lg:grid-cols-2 pt-4">
                <ChartCard
                  title="New Email Arrival"
                  data={data?.time_series.new_email_arrival}
                  dataKey="new_email_arrival"
                  color="var(--chart-1)"
                />
                <ChartCard
                  title="Mail Flag Change"
                  data={data?.time_series.mail_flag_change}
                  dataKey="mail_flag_change"
                  color="var(--chart-1)"
                />
              </CollapsibleContent>
            </Collapsible>
            <Collapsible defaultOpen={false}>
              <CollapsibleTrigger className="flex items-center justify-between w-full p-2 bg-muted rounded-md">
                <h2 className="text-lg font-semibold">Email Stats</h2>
                <ChevronsUpDown className="h-4 w-4" />
              </CollapsibleTrigger>
              <CollapsibleContent className="grid grid-cols-1 gap-4 lg:grid-cols-2 pt-4">
                <ChartCard
                  title="Email Sent Success"
                  data={data?.time_series.email_sent_success}
                  dataKey="email_sent_success"
                  color="var(--chart-1)"
                />
                <ChartCard
                  title="Email Sent Failure"
                  data={data?.time_series.email_sent_failure}
                  dataKey="email_sent_failure"
                  color="var(--chart-1)"
                />
                <ChartCard
                  title="Email Sent Bytes"
                  data={data?.time_series.email_sent_bytes}
                  dataKey="email_sent_bytes"
                  color="var(--chart-1)"
                  formatUnit="bytes"
                />
              </CollapsibleContent>
            </Collapsible>
            <Collapsible defaultOpen={false}>
              <CollapsibleTrigger className="flex items-center justify-between w-full p-2 bg-muted rounded-md">
                <h2 className="text-lg font-semibold">Email Engagement</h2>
                <ChevronsUpDown className="h-4 w-4" />
              </CollapsibleTrigger>
              <CollapsibleContent className="grid grid-cols-1 gap-4 lg:grid-cols-2 pt-4">
                <ChartCard
                  title="Email Opens"
                  data={data?.time_series.email_opens}
                  dataKey="email_opens"
                  color="var(--chart-1)"
                />
                <ChartCard
                  title="Email Clicks"
                  data={data?.time_series.email_clicks}
                  dataKey="email_clicks"
                  color="var(--chart-1)"
                />
              </CollapsibleContent>
            </Collapsible>

            <Collapsible defaultOpen={false}>
              <CollapsibleTrigger className="flex items-center justify-between w-full p-2 bg-muted rounded-md">
                <h2 className="text-lg font-semibold">Task Queue</h2>
                <ChevronsUpDown className="h-4 w-4" />
              </CollapsibleTrigger>
              <CollapsibleContent className="grid grid-cols-1 gap-4 lg:grid-cols-2 pt-4">
                <ChartCard
                  title="Email Task Queue"
                  data={data?.time_series.email_task_queue_length}
                  dataKey="email_task_queue_length"
                  color="var(--chart-1)"
                />
                <ChartCard
                  title="Hook Task Queue"
                  data={data?.time_series.hook_task_queue_length}
                  dataKey="hook_task_queue_length"
                  color="var(--chart-1)"
                />
              </CollapsibleContent>
            </Collapsible>
            <Collapsible defaultOpen={false}>
              <CollapsibleTrigger className="flex items-center justify-between w-full p-2 bg-muted rounded-md">
                <h2 className="text-lg font-semibold">Event Dispatch</h2>
                <ChevronsUpDown className="h-4 w-4" />
              </CollapsibleTrigger>
              <CollapsibleContent className="grid grid-cols-1 gap-4 lg:grid-cols-2 pt-4">
                <ChartCard
                  title="HTTP Success"
                  data={data?.time_series.event_dispatch_success_http}
                  dataKey="event_dispatch_success_http"
                  color="var(--chart-1)"
                />
                <ChartCard
                  title="HTTP Failure"
                  data={data?.time_series.event_dispatch_failure_http}
                  dataKey="event_dispatch_failure_http"
                  color="var(--chart-1)"
                />
                <ChartCard
                  title="NATS Success"
                  data={data?.time_series.event_dispatch_success_nats}
                  dataKey="event_dispatch_success_nats"
                  color="var(--chart-1)"
                />
                <ChartCard
                  title="NATS Failure"
                  data={data?.time_series.event_dispatch_failure_nats}
                  dataKey="event_dispatch_failure_nats"
                  color="var(--chart-1)"
                />
              </CollapsibleContent>
            </Collapsible>
          </div>
        )}
      </Main>
    </>
  );
}

const formatChartData = (data?: TimeSeriesPoint[], expectedIntervalMs: number = 60000) => {
  if (!data || data.length === 0) return [];

  // First format the existing data
  const formattedData = data.map(point => ({
    timestamp: point.timestamp,
    time: format(new Date(point.timestamp), 'HH:mm'),
    value: point.value
  }));

  // Sort by timestamp just in case
  formattedData.sort((a, b) => a.timestamp - b.timestamp);

  // Fill gaps with null values
  const filledData = [];
  for (let i = 0; i < formattedData.length; i++) {
    // Add the current point
    filledData.push(formattedData[i]);

    // Check gap to next point (if there is one)
    if (i < formattedData.length - 1) {
      const currentTime = formattedData[i].timestamp;
      const nextTime = formattedData[i + 1].timestamp;
      const gap = nextTime - currentTime;

      // If gap is significantly larger than expected interval
      if (gap > expectedIntervalMs * 5) {
        // Calculate how many missing points there should be
        const missingPoints = Math.round(gap / expectedIntervalMs) - 1;

        // Add null points for each missing interval
        for (let j = 1; j <= missingPoints; j++) {
          const missingTimestamp = currentTime + j * expectedIntervalMs;
          filledData.push({
            timestamp: missingTimestamp,
            time: format(new Date(missingTimestamp), 'HH:mm'),
            value: null
          });
        }
      }
    }
  }

  return filledData;
};

// ChartCard component using Shadcn chart components
function ChartCard({
  title,
  data,
  dataKey,
  color,
  height = 300,
  formatUnit,
}: {
  title: string;
  data?: TimeSeriesPoint[];
  dataKey: string;
  color: string;
  height?: number;
  formatUnit?: 'bytes';
}) {
  const chartData = formatChartData(data);

  const formatBytes = (bytes: number): string => {
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let value = bytes;
    let unitIndex = 0;

    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex++;
    }

    return `${value.toFixed(1)} ${units[unitIndex]}`;
  };

  const chartConfig = {
    [dataKey]: {
      label: title,
      color: color,
    },
  } satisfies ChartConfig;

  return (
    <Card>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
      </CardHeader>
      <CardContent>
        <ChartContainer config={chartConfig} className="w-full" style={{ height: `${height}px` }}>
          <AreaChart
            accessibilityLayer
            data={chartData}
            margin={{
              left: 12,
              right: 12,
              top: 10,
              bottom: 10,
            }}
          >
            <CartesianGrid vertical={false} strokeDasharray="3 3" />
            <XAxis
              dataKey="time"
              tickLine={true}
              axisLine={true}
              tickMargin={8}
            />
            <YAxis
              tickLine={true}
              axisLine={true}
              tickMargin={8}
              tickFormatter={formatUnit === 'bytes' ? formatBytes : undefined}
            />
            <ChartTooltip
              cursor={false}
              content={<ChartTooltipContent indicator="dot" hideLabel />}
            />
            <Area
              dataKey="value"
              connectNulls={false}
              type="monotone"
              fill={`var(--color-${dataKey})`}
              fillOpacity={0.4}
              stroke={`var(--color-${dataKey})`}
            />
          </AreaChart>
        </ChartContainer>
      </CardContent>
    </Card>
  );
}

// MetricCard component (unchanged)
function MetricCard({
  title,
  value,
  icon,
  description,
  isUptime = false
}: {
  title: string;
  value: string | number;
  icon: string;
  description: string;
  isUptime?: boolean;
}) {
  const formatUptime = (uptime: string) => {
    if (!uptime) return 'n/a';
    const ms = parseInt(uptime);
    const seconds = Math.floor(ms / 1000);
    const days = Math.floor(seconds / (3600 * 24));
    const hours = Math.floor((seconds % (3600 * 24)) / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    const parts = [];
    if (days > 0) parts.push(`${days}d`);
    if (hours > 0) parts.push(`${hours}h`);
    if (mins > 0) parts.push(`${mins}m`);
    if (secs > 0 || parts.length === 0) parts.push(`${secs}s`);
    return parts.join(' ');
  };

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        <IconComponent name={icon} />
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-bold">
          {isUptime ? formatUptime(value as string) : value}
        </div>
        <p className="text-xs text-muted-foreground">{description}</p>
      </CardContent>
    </Card>
  );
}

function IconComponent({ name }: { name: string }) {
  const icons: Record<string, JSX.Element> = {
    mail: (
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="h-4 w-4 text-muted-foreground">
        <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z" />
        <polyline points="22,6 12,13 2,6" />
      </svg>
    ),
    hook: (
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="h-4 w-4 text-muted-foreground">
        <path d="M18 6a4 4 0 0 0-4 4 7 7 0 0 0-7 7c0-5 4-5 4-10.5a4.5 4.5 0 1 0-9 0 2.5 2.5 0 0 0 5 0C7 10 3 11 3 17c0 2.8 2.2 5 5 5h10" />
      </svg>
    ),
    users: (
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="h-4 w-4 text-muted-foreground">
        <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
        <circle cx="9" cy="7" r="4" />
        <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
        <path d="M16 3.13a4 4 0 0 1 0 7.75" />
      </svg>
    ),
    clock: (
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="h-4 w-4 text-muted-foreground">
        <circle cx="12" cy="12" r="10" />
        <polyline points="12 6 12 12 16 14" />
      </svg>
    )
  };

  return icons[name] || <div className="h-4 w-4" />;
}

function DashboardSkeleton() {
  return (
    <div className="mt-8 space-y-4">
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {[...Array(4)].map((_, i) => (
          <Card key={i}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <Skeleton className="h-4 w-[100px]" />
              <Skeleton className="h-4 w-4" />
            </CardHeader>
            <CardContent>
              <Skeleton className="h-8 w-full" />
              <Skeleton className="h-4 w-full mt-2" />
            </CardContent>
          </Card>
        ))}
      </div>

      {[...Array(5)].map((_, i) => (
        <Card key={i} className="h-[350px]">
          <CardHeader>
            <Skeleton className="h-6 w-[200px]" />
          </CardHeader>
          <CardContent>
            <Skeleton className="h-full w-full" />
          </CardContent>
        </Card>
      ))}
    </div>
  );
}