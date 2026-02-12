import { Message as MsgType } from "@/App/store/db/chats";

import Markdown from 'react-markdown';

import remarkGfm from 'remark-gfm';
import remarkMath from 'remark-math';

import rehypeKatex from 'rehype-katex';
import rehypeHighlight from "rehype-highlight";

import 'katex/dist/katex.min.css';
import "highlight.js/styles/dark.min.css";


export function Message({ msg }: { msg: MsgType }) {
  console.log(msg);
  return <div className={`max-w-[75%] min-w-8 bg-secondary h-auto flex flex-col px-4 py-3 rounded-xl ${msg.responder == "user" ? "ml-auto" : "mr-auto"} overflow-clip`}>
    <div className="w-full min-w-0 wrap-break-word overflow-x-scroll">
      <Markdown
        remarkPlugins={[remarkGfm, remarkMath]}
        rehypePlugins={[rehypeKatex, rehypeHighlight]}
        components={{
          p: ({ node, ...props }) => <p className="whitespace-pre-wrap wrap-break-word" {...props} />,

          // Force the 'pre' block to wrap instead of scrolling
          pre: ({ node, className, ...props }) => (
            <pre className={`${className} whitespace-pre-wrap wrap-break-word`} {...props} />
          ),

          // Force the 'code' tag to wrap
          code: ({ node, className, ...props }) => (
            <code
              className={`${className} wrap-break-word whitespace-pre-wrap px-1 rounded`}
              {...props}
            />
          )
        }}
      >
        {msg.content}
      </Markdown>
    </div>

    <span className="text-muted-foreground text-xs ml-auto mt-2">{new Date(msg.created_at + "Z").toLocaleString()}</span>
    {msg.created_at != msg.updated_at && <span className="text-muted-foreground text-xs ml-auto mt-2">(Edited {new Date(msg.updated_at + "Z").toLocaleString()})</span>}
  </div>
}