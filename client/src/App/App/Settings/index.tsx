import { ServersState } from "@/App/store/db/servers";
import useStateData from "@/App/store/state";

import { Category } from "@/components/category";
import { Separator } from "@/components/ui/separator";
import { PlusIcon, ScrollText } from "lucide-react";

import { AppleLogoIcon, WindowsLogoIcon, AndroidLogoIcon, LinuxLogoIcon, /*ShoppingBagOpenIcon, ApplePodcastsLogoIcon*/ } from "@phosphor-icons/react";

import { ServerStackIcon } from "@heroicons/react/24/outline";

import { useState } from "react";
import { ResponsiveDialog } from "@/components/responsive/dialog";

import AddServer from "./AddServer";

import license from "../../../licenses.txt?raw";
import ServerBlob from "./ServerBlob";
import { isApple, os } from "../os";

export default function Settings() {
  const servers = useStateData(ServersState);

  const [open, setOpen] = useState(false);
  const [large, setLarge] = useState(false);

  return <>
    <ResponsiveDialog
      open={open}
      setOpen={setOpen}
      buttonVariant={"ghost"}
      forceLarge={large}
      title="Add Server"
      description="Enter the server information with required credentials"
      content={
        <AddServer setOpen={setOpen} setLarge={setLarge} />
      }
    />

    <h1 className="text-lg mb-1">General Settings</h1>
    <div className="flex flex-col gap-2 w-full">
      {isApple && <Category
        title="Development"
        description="Support for Apple macOS and iOS will be considered later. Active development is in progress."
        Icon={AppleLogoIcon}
        openable={false}
      />}

      {os == "windows" && <Category
        title="Windows Platform"
        description="This device is entitled to priority updates and rapid development by the leader of AHQ AI Project."
        Icon={WindowsLogoIcon}
        openable={false}
      />}

      {os == "android" && <Category
        title="Android"
        description="This phone is entitled to priority updates under AHQ Store distribution."
        Icon={AndroidLogoIcon}
        openable={false}
      />}

      {os == "linux" && <Category
        title="Linux"
        description="Support for linux is considered potentially in future with broken ui until other priority platforms are ready."
        Icon={LinuxLogoIcon}
        openable={false}
      />}

      <Category
        title="Servers"
        description="Configure servers"
        Icon={ServerStackIcon}
      >
        <h1 className="text-lg! mb-2">Server List</h1>

        <div className="w-full my-2 gap-2">
          {servers.map((server, i) => (
            <ServerBlob server={server} index={i} key={`server-${server.url}-${server.name}-${i}`} />
          ))}

          {servers.length == 0 && <span className="text-muted-foreground">No Servers Found...</span>}
        </div>

        <Separator />

        <div className="w-full flex text-center mt-6">
          <span className="text-lg my-auto">Add a new server</span>

          <button
            className="ml-auto flex cursor-pointer items-center gap-1 px-2 py-1 bg-accent dark:bg-neutral-content/20 dark:hover:bg-neutral-content/30 rounded-md"
            onClick={() => setOpen(true)}
          >
            <PlusIcon size={"1.2rem"} />
            <span>Add</span>
          </button>
        </div>
      </Category >
    </div>

    <h1 className="text-lg mt-3 mb-1">About & Attributions</h1>

    <div className="flex flex-col w-full gap-2">
      {/* <Category
        title="Evergreen Build"
        description="This build of AHQ AI is evergreen"
        Icon={ApplePodcastsLogoIcon}
        openable={false}
      />


      <Category
        title="AHQ Store Build"
        description="AHQ AI is shipped and updated by the AHQ Store"
        Icon={ShoppingBagOpenIcon}
        openable={false}
      /> */}

      <Category
        title="Licenses"
        description="Open Sourced Licenses"
        Icon={ScrollText}

      >
        <div className="w-full rounded-lg overflow-x-hidden">
          <pre
            className="bg-base-100/60"
            style={{
              whiteSpace: 'pre-wrap', // Essential for wrapping
              wordBreak: 'break-word', // Essential for long URLs
              fontFamily: 'monospace', // Keeps the text looking like code/documentation
              padding: '1em',
            }}
          >
            {license}
          </pre>
        </div>
      </Category>
    </div>


    <div className="flex flex-col justify-center items-center text-center mt-6 mb-6 text-muted-foreground gap-5">
      <span>AHQ AI<br></br>Licensed under GPLv3</span>
      <img width="200em" src="/gpl.svg" />

      <div>
        <a href="https://commons.wikimedia.org/wiki/File:GPLv3_Logo.svg" target="_blank" className="text-blue-600 underline">&copy; Free Software Foundation</a>, Public domain, via Wikimedia Commons
      </div>
    </div>
  </>;
}
