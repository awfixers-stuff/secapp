"use client";

import { remarkGfm } from "fumadocs-core/mdx-plugins/remark-gfm";
import type { TOCItemType } from "fumadocs-core/toc";
import {
  AnchorProvider,
  TOCItem,
  ScrollProvider,
  useActiveAnchor,
} from "fumadocs-core/toc";
import { Markdown } from "fumadocs-core/content/md";
import type { Components } from "hast-util-to-jsx-runtime";
import type { Compatible } from "vfile";
import { useRef, type ReactNode } from "react";

const mdxComponents: Components = {
  h2: ({ children, ...props }) => (
    <h2
      className="text-2xl font-semibold tracking-tight mt-8 mb-3 scroll-mt-16"
      {...props}
    >
      {children}
    </h2>
  ),
  h3: ({ children, ...props }) => (
    <h3
      className="text-xl font-semibold tracking-tight mt-6 mb-2 scroll-mt-16"
      {...props}
    >
      {children}
    </h3>
  ),
  h4: ({ children, ...props }) => (
    <h4
      className="text-lg font-semibold tracking-tight mt-4 mb-2 scroll-mt-16"
      {...props}
    >
      {children}
    </h4>
  ),
  p: ({ children, ...props }) => (
    <p className="leading-7 mb-4" {...props}>
      {children}
    </p>
  ),
  ul: ({ children, ...props }) => (
    <ul className="list-disc pl-6 mb-4 leading-7" {...props}>
      {children}
    </ul>
  ),
  ol: ({ children, ...props }) => (
    <ol className="list-decimal pl-6 mb-4 leading-7" {...props}>
      {children}
    </ol>
  ),
  li: ({ children, ...props }) => (
    <li className="mb-1" {...props}>
      {children}
    </li>
  ),
  a: ({ children, href, ...props }) => (
    <a
      href={href}
      className="text-primary underline underline-offset-4 hover:text-primary/80"
      {...props}
    >
      {children}
    </a>
  ),
  code: ({ children, ...props }) => (
    <code
      className="bg-muted rounded px-1.5 py-0.5 font-mono text-sm"
      {...props}
    >
      {children}
    </code>
  ),
  pre: ({ children, ...props }) => (
    <pre
      className="bg-muted rounded-lg p-4 overflow-x-auto mb-4 text-sm leading-6"
      {...props}
    >
      {children}
    </pre>
  ),
  blockquote: ({ children, ...props }) => (
    <blockquote
      className="border-l-4 border-muted-foreground/30 pl-4 italic text-muted-foreground mb-4"
      {...props}
    >
      {children}
    </blockquote>
  ),
  hr: (props) => <hr className="my-8 border-border" {...props} />,
  table: ({ children, ...props }) => (
    <div className="overflow-x-auto mb-4">
      <table className="w-full border-collapse" {...props}>
        {children}
      </table>
    </div>
  ),
  thead: ({ children, ...props }) => (
    <thead className="bg-muted" {...props}>
      {children}
    </thead>
  ),
  th: ({ children, ...props }) => (
    <th className="border border-border px-4 py-2 text-left font-semibold" {...props}>
      {children}
    </th>
  ),
  td: ({ children, ...props }) => (
    <td className="border border-border px-4 py-2" {...props}>
      {children}
    </td>
  ),
  tr: ({ children, ...props }) => (
    <tr className="border-b" {...props}>
      {children}
    </tr>
  ),
  strong: ({ children, ...props }) => (
    <strong className="font-semibold" {...props}>
      {children}
    </strong>
  ),
  em: ({ children, ...props }) => (
    <em className="italic" {...props}>
      {children}
    </em>
  ),
  del: ({ children, ...props }) => (
    <del className="line-through" {...props}>
      {children}
    </del>
  ),
  input: (props) => {
    if (props.type === "checkbox") {
      return (
        <input
          className="mr-1.5 h-4 w-4 rounded border-border"
          disabled
          {...props}
        />
      );
    }
    return <input {...props} />;
  },
};

/**
 * Render markdown content with GFM support, heading IDs for TOC linking,
 * and consistent typography styling.
 */
export function RenderMarkdown({ content }: { content: string }) {
  return (
    <Markdown remarkPlugins={[remarkGfm]} components={mdxComponents}>
      {content as Compatible}
    </Markdown>
  );
}

/**
 * Right-side TOC sidebar that highlights the active heading.
 * Uses fumadocs-core's AnchorProvider for scroll tracking.
 */
export function TocSidebar({ toc }: { toc: TOCItemType[] }) {
  if (!toc || toc.length === 0) return null;

  return (
    <AnchorProvider toc={toc}>
      <TocContent toc={toc} />
    </AnchorProvider>
  );
}

function TocContent({ toc }: { toc: TOCItemType[] }) {
  const activeId = useActiveAnchor();

  return (
    <nav className="text-sm">
      <h4 className="font-semibold mb-3">On This Page</h4>
      <ul className="space-y-1 border-l">
        {toc.map((item) => {
          const isActive = `#${activeId}` === item.url;
          return (
            <li
              key={item.url}
              style={{ paddingLeft: `${(item.depth - 2) * 12}px` }}
            >
              <a
                href={item.url}
                className={`block py-0.5 pl-3 transition-colors border-l-2 -ml-px ${
                  isActive
                    ? "border-primary text-primary font-medium"
                    : "border-transparent text-muted-foreground hover:text-foreground"
                }`}
              >
                {(item.title as ReactNode)}
              </a>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}