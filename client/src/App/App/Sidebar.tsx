import { Separator } from "@/components/ui/separator";
import { MessageCircleDashed, MessageCircle, Settings, /*ShieldUser,*/ LucideProps } from "lucide-react";
import { ForwardRefExoticComponent, RefAttributes } from "react";
import { AppPage } from ".";

interface Chat {
  content: string;
  id: string;
}

interface SidebarProps {
  chats: Chat[];
  page: AppPage;
  pageSet: (prop: AppPage) => void;
  chatPage: string;
  chatPageSet: (prop: string) => void;
}

export default function Sidebar({ chats: chats, page, pageSet, chatPage, chatPageSet }: SidebarProps) {
  chats = [
    {
      content: "Hello World",
      id: "asf6f"
    },
    {
      content: "HOw to become a millionare in 28seconds and 30mins and 1200000days",
      id: "asf6f"
    }
  ];

  return <div className="w-full h-full px-3 py-2 gap-1 flex flex-col overflow-y-scroll overflow-x-clip">
    <SidebarItem
      text="New Chat"
      Icon={MessageCircle}
      isActive={page == AppPage.Chat}
      activated={() => {
        pageSet(AppPage.Chat);
      }}
    />

    <SidebarItem
      text="Incognito Chat"
      Icon={MessageCircleDashed}
      isActive={page == AppPage.Diposable}
      activated={() => {
        pageSet(AppPage.Diposable);
      }}
    />

    <Separator />

    <div className="h-full flex flex-col w-full gap-1">
      <div className="text-muted-foreground select-none ml-2">
        Chats
      </div>

      {chats.map((data) => (
        <SidebarItem
          text={data.content}
          isActive={page == AppPage.ChatPage && chatPage == data.id}
          Icon={MessageCircle}
          activated={() => {
            chatPageSet(data.id);
            pageSet(AppPage.ChatPage);
          }}
          key={data.id}
        />
      ))}

      {chats.length == 0 && <span className="mx-auto mt-2 select-none text-secondary-foreground">No chats found</span>}
    </div>

    <Separator />

    {/* <SidebarItem
      text="Admin Portal"
      Icon={ShieldUser}
      isActive={page == AppPage.Admin}
      activated={() => {
        pageSet(AppPage.Admin);
      }}
    /> */}

    <SidebarItem
      text="Settings"
      Icon={Settings}
      isActive={page == AppPage.Settings}
      activated={() => {
        pageSet(AppPage.Settings);
      }}
    />
  </div>
}

function SidebarItem({ text, Icon, isActive, activated }: { text: string, Icon: ForwardRefExoticComponent<Omit<LucideProps, "ref"> & RefAttributes<SVGSVGElement>>, activated?: () => void, isActive: boolean }) {
  return <div onClick={() => activated && activated()} className={`w-full h-10 flex overflow-x-hidden rounded-lg overflow-y-hidden px-3 gap-2 py-2 select-none cursor-pointer transition-all border border-transparent ${isActive ? "shadow-lg! border-border! bg-neutral/30!" : "hover:shadow-lg hover:border-border hover:bg-neutral/30"} items-center group`}>
    <Icon className={`text-muted-foreground ${isActive ? "text-base-content!" : "group-hover:text-base-content"} min-h-5 max-h-5 min-w-5 max-w-5`} />
    <span className={`text-sm line-clamp-1 text-muted-foreground ${isActive ? "text-base-content!" : "group-hover:text-base-content"}`}>{text}</span>
  </div>
}