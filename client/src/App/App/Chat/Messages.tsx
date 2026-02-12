import { ChatInstance, Message } from "@/App/store/db/chats";
import { Button } from "@/components/ui/button";
import { ArrowDown } from "lucide-react";
import { useRef } from "react";
import { Message as MessageComp } from "./Message";

export interface MsgsProps {
  messages: Message[],
  chat: ChatInstance,
  generating: boolean
}

export function Messages({ messages, generating }: MsgsProps) {
  const downRef = useRef<HTMLButtonElement>(null);
  const scrollDivRef = useRef<HTMLDivElement>(null);

  return <div className="relative h-full w-full">
    <div
      className="flex flex-col-reverse w-full h-full overflow-y-scroll px-5 py-1 md:px-2 gap-2 scroll-smooth"
      ref={scrollDivRef}
      onScroll={(e) => {
        if (e.currentTarget.scrollTop != 0) {
          downRef.current!.hidden = false;
        } else {
          downRef.current!.hidden = true;
        }
      }}
    >
      {generating && <span className="mx-auto dui-loading dui-loading-dots dui-loading-xl" />}

      {messages && [...messages].reverse().map((x) => <MessageComp key={x.id} msg={x} />)}
    </div>

    <Button
      ref={downRef}
      hidden
      onClick={() => scrollDivRef.current!.scrollTo({ top: 0, behavior: "smooth" })}
      className="absolute bottom-5 right-5 rounded-full" variant={"outline"} size={"icon-sm"}
    >
      <span className="absolute -top-1/15 -right-1/15 flex h-2 w-2">
        <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-info/75"></span>
        <span className="relative inline-flex rounded-full h-2 w-2 bg-info"></span>
      </span>

      <ArrowDown />
    </Button>
  </div>;
}