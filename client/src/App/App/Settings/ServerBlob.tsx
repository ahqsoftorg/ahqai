import { StatusFlags } from "@/App/server";
import { ServersState, Server as ServerType } from "@/App/store/db/servers";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "@/components/ui/hover-card";

import { CloudXIcon, CloudIcon, CloudWarningIcon, CloudSlashIcon, SmileyXEyesIcon, ClockCountdownIcon } from "@phosphor-icons/react"
import { AlertCircleIcon, AlertTriangleIcon, LogIn, MoreHorizontalIcon, Trash2Icon } from "lucide-react";
import { ReactNode, useMemo } from "react";
import { AuthType } from "../../server";

export default function ServerBlob({ server, index }: { server: ServerType, index: number }) {
  const errors = useMemo(() => {
    const flags = server.instance.flags;

    const err: ReactNode[] = [];

    if ((flags & (StatusFlags.Unavailable)) > 0) {
      err.push(
        <Alert variant="destructive">
          <CloudSlashIcon />
          <AlertTitle>Server Offline</AlertTitle>
          <AlertDescription>Please turn on the server</AlertDescription>
        </Alert>
      );
    }

    if ((flags & (StatusFlags.SessionExpired)) > 0) {
      err.push(
        <Alert variant="destructive">
          <AlertTriangleIcon />
          <AlertTitle>Session Expired</AlertTitle>
          <AlertDescription>Please re authenticate.</AlertDescription>
        </Alert>
      );
    }
    if ((flags & (StatusFlags.ExpiresSoon)) > 0) {
      err.push(
        <Alert>
          <ClockCountdownIcon className="text-warning!" />
          <AlertTitle>Expires Soon</AlertTitle>
          <AlertDescription>
            <span>This version of AHQ AI Server will expire on <strong className="text-error">{server.instance.expiry?.toLocaleDateString()}</strong> at <strong className="text-error">{server.instance.expiry?.toLocaleTimeString()}</strong></span>
          </AlertDescription>
        </Alert>
      );
    }
    if ((flags & (StatusFlags.Unauthorized)) > 0) {
      err.push(
        <Alert variant="destructive">
          <AlertCircleIcon />
          <AlertTitle>Unauthorized</AlertTitle>
          <AlertDescription>Kindly relogin</AlertDescription>
        </Alert>
      );
    }
    if ((flags & (StatusFlags.ChallengeFailed)) > 0) {
      err.push(
        <Alert variant="destructive">
          <AlertTriangleIcon />
          <AlertTitle>Integrity Check Failed</AlertTitle>
          <AlertDescription>Unable to verify server integrity.</AlertDescription>
        </Alert>
      );
    }
    if ((flags & StatusFlags.UnsupportedServerVersion) > 0) {
      err.push(
        <Alert variant="destructive">
          <SmileyXEyesIcon />
          <AlertTitle>Outdated</AlertTitle>
          <AlertDescription>Running on outdated server. Kindly upgrade the server to use it.</AlertDescription>
        </Alert>
      );
    }

    return err;
  }, [server]);

  const icon = useMemo(() => {
    const flags = server.instance.flags;

    if ((flags & (StatusFlags.SessionExpired)) > 0) {
      return <CloudSlashIcon className="size-8 text-error" />;
    } else if ((flags & (StatusFlags.Unavailable)) > 0) {
      return <CloudSlashIcon className="size-8 text-warning" />;
    } else if ((flags & (StatusFlags.Unauthorized)) > 0) {
      return <CloudXIcon className="size-8 text-error" />;
    } else if ((flags & (StatusFlags.ExpiresSoon)) > 0) {
      return <ClockCountdownIcon className="size-8 text-warning" />;
    } else if ((flags && (StatusFlags.ChallengeFailed)) > 0) {
      return <CloudWarningIcon className="size-8 text-warning" />;
    } else if ((flags && StatusFlags.UnsupportedServerVersion) > 0) {
      return <SmileyXEyesIcon className="size-8 text-warning" />;
    } else {
      return <CloudIcon className="size-8 text-success" />;
    }
  }, [server]);

  return <div className="w-full h-18 flex rounded-lg bg-neutral-content/10 px-2">
    <div className="ml-1 mr-3 my-3 min-w-12 min-h-12 max-w-12 max-h-12 max-size-16 flex justify-center items-center text-center rounded-lg bg-base-100/50">
      {icon}
    </div>

    <div className="h-full max-w-1/4 sm:max-w-3/5 overflow-clip mr-7 flex flex-col py-2">
      <div className="text-xl flex text-center items-center overflow-clip">
        <h1 className="truncate lines-1">{server.name}</h1>
      </div>
      <span className="text-muted-foreground">{server.url}</span>
    </div>

    {errors.length > 0 &&
      <div className="h-full py-2">
        <HoverCard>
          <HoverCardTrigger asChild>
            <Badge variant={"destructive"}>{errors.length} <span className="hidden sm:block">issues</span></Badge>
          </HoverCardTrigger>

          <HoverCardContent className="flex flex-col w-[30rem] max-h-72 overflow-y-scroll overflow-x-hidden bg-base-100/70 gap-2">
            {
              errors.map((x, i) => <div key={`err-${i}`} className="w-full flex flex-col">{x}</div>)
            }
          </HoverCardContent>
        </HoverCard>
      </div>
    }

    <div className="h-full w-[3rem] flex justify-center text-center items-center ml-auto">
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
          >
            <MoreHorizontalIcon />
          </Button>
        </DropdownMenuTrigger>

        <DropdownMenuContent className="w-[10rem] mr-5">
          <DropdownMenuLabel>Account</DropdownMenuLabel>

          <DropdownMenuGroup>
            <DropdownMenuItem disabled={server.instance.auth == AuthType.OpenToAll}>
              <LogIn />
              Reauthenticate
            </DropdownMenuItem>
          </DropdownMenuGroup>

          <DropdownMenuLabel>Actions</DropdownMenuLabel>

          <DropdownMenuGroup>
            <DropdownMenuItem
              variant="destructive"
              onClick={() => {
                ServersState.updateValueViaCallback((c) => {
                  return c.filter((_unk, i) => i != index)
                });
              }}
            >
              <Trash2Icon />
              Delete
            </DropdownMenuItem>
          </DropdownMenuGroup>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  </div >
}