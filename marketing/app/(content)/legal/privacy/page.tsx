import { RenderMarkdown } from "@/components/render-markdown";
import { getLegalContent } from "@/lib/content";
import { notFound } from "next/navigation";
import { DocsPage, DocsBody, DocsTitle, DocsDescription } from "fumadocs-ui/layouts/docs/page";

const slug = "privacy";

export default async function PrivacyPage() {
  const entry = getLegalContent(slug);
  if (!entry) notFound();

  return (
    <DocsPage toc={entry.toc}>
      <DocsTitle>{entry.title}</DocsTitle>
      {entry.description && <DocsDescription>{entry.description}</DocsDescription>}
      <DocsBody>
        <RenderMarkdown content={entry.content} />
      </DocsBody>
    </DocsPage>
  );
}