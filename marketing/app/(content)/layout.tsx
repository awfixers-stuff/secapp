import { buildPageTree } from "@/lib/page-tree";
import { DocsLayout } from "fumadocs-ui/layouts/docs";
import { Root } from "fumadocs-core/page-tree";

const tree: Root = buildPageTree();

export default function ContentLayout({ children }: { children: React.ReactNode }) {
  return (
    <DocsLayout
      tree={tree}
      nav={{
        title: (
          <span className="flex items-center gap-2 font-semibold">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              className="shrink-0"
            >
              <path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z" />
            </svg>
            secapp
          </span>
        ),
        url: "/",
      }}
      links={[
        {
          text: "GitHub",
          url: "https://github.com/awfixer/secapp",
          type: "main",
        },
      ]}
    >
      {children}
    </DocsLayout>
  );
}