"use client";

import { remarkGfm } from "fumadocs-core/mdx-plugins/remark-gfm";
import type { TOCItemType } from "fumadocs-core/toc";
import {
  AnchorProvider,
  useActiveAnchor,
} from "fumadocs-core/toc";
import { Markdown } from "fumadocs-core/content/md";
import type { Components } from "hast-util-to-jsx-runtime";
import type { Compatible } from "vfile";
import type { ReactNode } from "react";
import rehypeSlug from "rehype-slug";

/**
 * Minimal MDX component overrides.
 *
 * fumadocs-ui's DocsBody already handles typography (headings, paragraphs,
 * lists, code, etc.). We only override what needs custom treatment:
 * - tables need border styling
 * - checkboxes need proper styling for GFM task lists
 * - del needs strikethrough
 */
const mdxComponents: Components = {
  table: ({ children, ...props }) => (
    <div className="overflow-x-auto mb-4">
      <table className="w-full border-collapse border border-border" {...props}>
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
  del: ({ children, ...props }) => (
    <del className="line-through" {...props}>
      {children}
    </del>
  ),
  code: ({ children, ...props }) => (
    <code
      className="rounded bg-muted px-1.5 py-0.5 text-sm font-mono"
      {...props}
    >
      {children}
    </code>
  ),
  pre: ({ children, ...props }) => (
    <pre
      className="overflow-x-auto rounded-lg border bg-muted/50 p-4 text-sm leading-6"
      {...props}
    >
      {children}
    </pre>
  ),
  input: (props) => {
    if (props.type === "checkbox") {
      return (
        <input
          className="mr-1.5 h-4 w-4 rounded border-border align-middle"
          {...props}
          disabled
        />
      );
    }
    return <input {...props} />;
  },
};

/**
 * Render markdown content with GFM support, heading IDs for TOC linking.
 *
 * Typography is handled by fumadocs-ui's DocsBody — we only override
 * tables, checkboxes, and strikethrough here.
 */
export function RenderMarkdown({ content }: { content: string }) {
  return (
    <Markdown
      remarkPlugins={[remarkGfm]}
      rehypePlugins={[rehypeSlug]}
      components={mdxComponents}
    >
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