// import ctrlOrCmd from "@/App/server/kbd";

import { ChatInstance } from "@/App/store/db/chats";
import { ServersState } from "@/App/store/db/servers";
import useStateData from "@/App/store/state";
import { Button } from "@/components/ui/button";

import { DropdownMenu, DropdownMenuContent, DropdownMenuLabel, DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuSeparator, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { InputGroup, InputGroupAddon, InputGroupButton, InputGroupTextarea } from "@/components/ui/input-group";
import { Separator } from "@/components/ui/separator";
import { useMediaQuery } from "@/hooks/use-media-query";
import { UnlistenFn } from "@tauri-apps/api/event";

import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow"

import { ArrowUp, ChevronLeft, ChevronRight, PodcastIcon, NetworkIcon, Package } from "lucide-react";
import React, { Suspense, use, useCallback, useEffect, useMemo, useRef, useState } from "react";

import { toast } from "sonner";
import { Messages } from "./Messages";
import { AIWSChat } from "@/App/server/ws";

export interface ChatProps {
  newChat: boolean;
  temporary: boolean;
  chatId?: number;
  updateChatPage: ((data: number | undefined) => void) | undefined;
}

interface InternalProps {
  chat: Promise<ChatInstance>;
  updateChatPage: ((data: number | undefined) => void) | undefined;
}

export default function Chat(props: ChatProps) {
  const { temporary, newChat, chatId } = props; // Destructure here

  const chatctx = useMemo(() => {
    if (temporary) {
      return (async () => {
        const c = new ChatInstance();
        await c.init("temporary");
        return c;
      })();
    }

    if (newChat && !chatId) {
      return (async () => {
        const c = new ChatInstance();
        await c.init(undefined);
        return c;
      })();
    }

    return (async () => {
      const c = new ChatInstance();
      await c.init(chatId);
      return c;
    })();
  }, [temporary, newChat, chatId]);

  return <Suspense fallback={<Loading />}>
    <ChatLayout chat={chatctx} updateChatPage={props.updateChatPage} />
  </Suspense>
}

function Loading() {
  return <div className="w-full h-full flex flex-col justify-center text-center align-center">
    <span className="dui-loading dui-loading-dots dui-loading-xl block mx-auto" />
  </div>;
}

function ChatLayout(props: InternalProps) {
  const chatinterface = use(props.chat);

  const scrollable = useRef<HTMLDivElement | null>(null);
  const [scrolled, setScroll] = useState(0);
  const [width, setWidth] = useState(0);

  const serverList = useStateData(ServersState);

  const [connection, setConnection] = useState<object | "connecting" | undefined>(undefined);
  const [selection, setSelection] = useState<string | undefined>();

  const [ws, setWs] = useState<AIWSChat | undefined>();

  const size = useMediaQuery("(min-width: 768px)");

  // Clean up WS
  useEffect(() => {
    return () => {
      ws?.disconnect();
      chatinterface.cleanup();
    };
  }, [ws, chatinterface]);

  // TODO: Side Effects
  useEffect(() => {
    let ev: UnlistenFn;
    try {
      const window = getCurrentWebviewWindow();

      (async () => {
        ev = await window.onDragDropEvent((ev) => {
          const ty = ev.payload.type;

          // User hovering
          if (ty == "over") {

          } else
            // User dropped
            if (ty == "drop") {
              alert("Dropped");
            } else
            // Cancelled
            {

            }
        });
      })()
    } catch (e) {
      console.log(e);
    }

    return () => {
      try { ev(); } catch (e) { }
      toast.dismiss();
    }
  }, []);

  // Scroll
  useEffect(() => {
    setWidth(scrollable.current!!.scrollWidth - scrollable.current!!.clientWidth);
    scrollable.current!!.addEventListener("scroll", () => {
      setScroll(Math.round(scrollable.current!!.scrollLeft));
    });
  }, [scrollable]);

  const onSelectConnect = useCallback(() => {
    if (selection) {
      setConnection("connecting");

      toast.promise(
        async () => {
          const server = serverList[Number(selection!.split("-")[0])].instance;
          const model = server.models[Number(selection!.split("-")[1])].id;

          const ws = server.getWSCLass(model, chatinterface);
          await ws.connect().catch(console.error);

          try {
            await ws.restore();
            await ws.init();

            setWs(ws);
            setConnection({});
          } catch (e) {
            console.log(e);
            await ws.disconnect();
            throw new Error(String(e));
          }

        },
        {
          position: size ? "top-right" : "top-center",
          loading: "Connecting...",
          success: `Successfully connected`,
          error: (data) => `Error: ${data}`,
          duration: 2000
        }
      );
    }
  }, [chatinterface, selection, size]);


  const inputRef = useRef<HTMLTextAreaElement | null>(null);
  const submit = useCallback(() => {
    const prompt = inputRef.current!.value;

    chatinterface.insertMessage({
      content: prompt,
      responder: "user",
      metadata: ""
    }).then(() => {
      if (typeof (chatinterface.chat_id) == "number") {
        props.updateChatPage?.(chatinterface.chat_id);
      }
    });

    inputRef.current!.value = "";
  }, [chatinterface, inputRef]);

  return <div className="w-full h-full flex flex-col gap-1 md:pb-5">
    <div className="h-full w-full overflow-y-scroll">
      <Messages key={`chat-${chatinterface.chat_id}`} chat={chatinterface} server={serverList[Number(selection?.split("-")?.[0])]?.instance} ws={ws} />
    </div>

    <div className="w-full items-center text-center justify-center flex">
      <InputGroup className="w-full rounded-none sm:rounded-md max-h-64 md:min-w-120 sm:max-w-[75%]">
        <InputGroupTextarea
          disabled={typeof (connection) != "object"}
          onPaste={(data) => {
            console.log(data.clipboardData.files);
            alert("Paste!!");
          }}
          ref={inputRef}
          placeholder={typeof (connection) != "object" ? "Connect to chat with AI" : "Ask, Converse or Chat about a topic..."}
          onKeyDown={(e) => {
            if (!e.shiftKey && e.key == 'Enter') {
              e.preventDefault();
              submit();
            }
          }}
        />

        <InputGroupAddon
          ref={scrollable}
          align="block-start"
          className="cursor-default overflow-y-hidden overflow-x-scroll transition-all min-h-25! hidden"
        >
          <div className="min-h-20 max-h-20 max-w-20 min-w-20 bg-black rounded-lg" />
        </InputGroupAddon>
        <InputGroupAddon
          align="block-start"
          className="cursor-default my-auto flex px-5 absolute hidden"
        >
          <InputGroupButton
            variant="default"
            className="rounded-full cursor-pointer absolute top-15"
            size="icon-xs"
            disabled={scrolled == 0}
            onClick={() => {
              scrollable.current!!.scroll({
                left: scrollable.current!!.scrollLeft - 160,
                behavior: "smooth"
              });
            }}
          >
            <ChevronLeft />
          </InputGroupButton>

          <InputGroupButton
            variant="default"
            className="rounded-full cursor-pointer absolute right-5 top-15"
            size="icon-xs"
            disabled={scrolled == width}
            onClick={() => {
              scrollable.current!!.scroll({
                left: scrollable.current!!.scrollLeft + 160,
                behavior: "smooth"
              });
            }}
          >
            <ChevronRight />
          </InputGroupButton>
        </InputGroupAddon>

        <InputGroupAddon align="block-end" className="cursor-default">
          {connection === undefined &&
            <>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <InputGroupButton
                    variant="outline"
                    className="rounded-lg"
                  >
                    {selection ?
                      <>
                        <Package />
                        {serverList[Number(selection.split("-")[0])].instance.models[Number(selection.split("-")[1])].name}
                      </>
                      :
                      "Select model"
                    }
                  </InputGroupButton>
                </DropdownMenuTrigger>

                {/* Model Select */}
                <DropdownMenuContent
                  side="top"
                  align="start"
                  className="rounded-md"
                >
                  <DropdownMenuRadioGroup value={selection} onValueChange={setSelection}>
                    {
                      serverList.map((server, index) => (
                        <React.Fragment key={`${server.url}-${index}`}>
                          <DropdownMenuLabel>{server.name} {!server.instance.usable && "(Relogin)"}</DropdownMenuLabel>

                          <DropdownMenuSeparator />

                          {
                            server.instance.models.map((model, idx) => (
                              <DropdownMenuRadioItem disabled={!server.instance.usable} key={`${idx}-${model.id}`} value={`${index}-${idx}`}>{model.name}</DropdownMenuRadioItem>
                            ))
                          }
                        </React.Fragment>
                      ))
                    }
                  </DropdownMenuRadioGroup>
                </DropdownMenuContent>
              </DropdownMenu>

              <Button
                variant="outline"
                className="rounded-full ml-auto"
                size="xs"
                onClick={() => onSelectConnect()}
              >
                <PodcastIcon />
                Connect
              </Button>
            </>}

          {connection === "connecting" &&
            <>
              <Button
                variant="outline"
                className="rounded-full"
                size="xs"
                disabled
              >
                {selection ?
                  <>
                    <Package />
                    {serverList[Number(selection.split("-")[0])].instance.models[Number(selection.split("-")[1])].name}
                  </>
                  :
                  "Select model"
                }
              </Button>

              <Button
                variant="outline"
                className="rounded-full ml-auto"
                size="xs"
                disabled
              >
                <PodcastIcon />
                Connecting...
              </Button>
            </>}

          {typeof (connection) == "object" && <>


            {/* <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <InputGroupButton
                  variant="outline"
                  className="rounded-full"
                  size="icon-xs"
                >
                  <Plus />
                </InputGroupButton>
              </DropdownMenuTrigger>

              <DropdownMenuContent
                side="top"
                align="start"
                className="p-2 [--radius:0.95rem]"
              >
                <DropdownMenuItem>
                  <Image />
                  Upload Image

                  <DropdownMenuShortcut className="ml-10">{ctrlOrCmd("V")}</DropdownMenuShortcut>
                </DropdownMenuItem>
              </DropdownMenuContent>

            </DropdownMenu> */}

            <div className="ml-auto">
              <Button
                variant="outline"
                className="rounded-full"
                size="xs"
              >
                <NetworkIcon />
                {serverList[Number(selection!!.split("-")[0])].instance.models[Number(selection!!.split("-")[1])].name}
              </Button>
            </div>

            <Separator orientation="vertical" className="h-4!" />

            <InputGroupButton
              variant="default"
              className="rounded-full cursor-pointer"
              size="icon-xs"
              onClick={() => submit()}
            >
              <ArrowUp />
              <span className="sr-only">Send</span>
            </InputGroupButton>
          </>}
        </InputGroupAddon>
      </InputGroup>
    </div>
  </div>
}