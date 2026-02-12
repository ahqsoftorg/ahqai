// import ctrlOrCmd from "@/App/server/kbd";

import { ServersState } from "@/App/store/db/servers";
import useStateData from "@/App/store/state";
import { Button } from "@/components/ui/button";

import { DropdownMenu, DropdownMenuContent, DropdownMenuLabel, DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuSeparator, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { InputGroup, InputGroupAddon, InputGroupButton, InputGroupTextarea } from "@/components/ui/input-group";
import { Separator } from "@/components/ui/separator";
import { UnlistenFn } from "@tauri-apps/api/event";

import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow"

import { ArrowUp, ChevronLeft, ChevronRight, PodcastIcon, NetworkIcon, Package } from "lucide-react";
import React, { Suspense, useCallback, useEffect, useRef, useState } from "react";

import { toast } from "sonner";
import { Messages } from "./Messages";
import { ButtonGroup } from "@/components/ui/button-group";
import { useChatManager } from "@/App/server/chat";

export interface ChatProps {
  newChat: boolean;
  temporary: boolean;
  chatId?: number;
  refresh: number | undefined;
  updateChatPage: ((data: number | undefined) => void) | undefined;
}

export default function Chat(props: ChatProps) {
  return <Suspense fallback={<Loading />}>
    <ChatLayout key={`chatinstance-${props.refresh || props.chatId || "temporary"}`} {...props} />
  </Suspense>
}

function Loading() {
  return <div className="w-full h-full flex flex-col justify-center text-center align-center">
    <span className="dui-loading dui-loading-dots dui-loading-xl block mx-auto" />
  </div>;
}

function ChatLayout(props: ChatProps) {
  console.log(props.refresh || props.chatId || 5512);
  const { chat, ws, messages, chatHook, status, connectWS } = useChatManager(
    props.chatId,
    props.temporary,
    props.newChat,
    props.refresh || props.chatId || 5512
  );

  const scrollable = useRef<HTMLDivElement | null>(null);
  const [scrolled, setScroll] = useState(0);
  const [width, setWidth] = useState(0);

  const serverList = useStateData(ServersState);

  const [selection, setSelection] = useState<string | undefined>();

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

  const [responding, setResponding] = useState(false);

  const onSelectConnect = useCallback(async () => {
    if (!selection || !chat) return;

    const [serverIdx, modelIdx] = selection.split("-").map(Number);
    const server = serverList[serverIdx].instance;
    const model = server.models[modelIdx].id;

    try {
      await connectWS(server, model);
      toast.success("Successfully connected");
    } catch (e) {
      toast.error(`Connection failed: ${e}`);
    }
  }, [selection, serverList, chat, connectWS]);


  const inputRef = useRef<HTMLTextAreaElement | null>(null);

  const submit = useCallback(async () => {
    if (!chat || !ws || responding || !inputRef.current?.value) return;

    const prompt = inputRef.current.value;
    setResponding(true);
    inputRef.current.value = "";

    try {
      // The hook's 'chat.cb' will automatically pick up this insertion 
      // and update the 'messages' array for us.
      const msgid = await chat.insertMessage({
        content: prompt,
        responder: "user",
        metadata: ""
      });

      // Update parent page if it's a new real chat
      if (typeof chat.chat_id === "number") {
        props.updateChatPage?.(chat.chat_id);
      }

      await chatHook(msgid!!, prompt);
    } catch (e) {
      toast.error("Failed to send message");
    } finally {
      setResponding(false);
    }
  }, [chat, ws, responding, props.updateChatPage]);

  return <div className="w-full h-full flex flex-col gap-1 md:pb-5">
    <div className="h-full w-full overflow-y-scroll">
      <Messages
        messages={messages}
        generating={responding}
        chat={chat!}
      />
    </div>

    <div className="w-full items-center text-center justify-center flex">
      <InputGroup className="w-full rounded-none sm:rounded-md max-h-64 md:min-w-120 sm:max-w-[75%]">
        <InputGroupTextarea
          disabled={status !== "connected"}
          onPaste={(data) => {
            console.log(data.clipboardData.files);
            alert("Paste!!");
          }}
          ref={inputRef}
          placeholder={status !== "connected" ? "Connect to chat with AI" : "Ask, Converse or Chat about a topic..."}
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
          className="cursor-default my-auto px-5 absolute hidden!"
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
          {status === "ready" &&
            <ButtonGroup className="ml-auto">
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <InputGroupButton
                    variant="outline"
                    className="rounded-lg"
                  >
                    {selection ?
                      <>
                        <Package />
                        {(() => {
                          const [sIdx, mIdx] = selection.split("-").map(Number);
                          return serverList[sIdx]?.instance?.models[mIdx]?.name || "Connected";
                        })()}
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
                className="rounded-full"
                size="xs"
                onClick={() => onSelectConnect()}
              >
                <PodcastIcon />
                Connect
              </Button>
            </ButtonGroup>}

          {status === "connecting" &&
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

          {status === "connected" && <>


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
                {(() => {
                  const [sIdx, mIdx] = (selection || "-")?.split("-").map(Number)!;
                  return serverList[sIdx]?.instance?.models[mIdx]?.name || "Connected";
                })()}
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