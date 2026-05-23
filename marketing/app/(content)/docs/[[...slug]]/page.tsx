import { RenderMarkdown } from "@/components/render-markdown";
import { getDocsContent, getDocsSlugs } from "@/lib/content";
import { notFound } from "next/navigation";
import { DocsPage as FumadocsDocsPage, DocsBody, DocsTitle, DocsDescription } from "fumadocs-ui/layouts/docs/page";

export default async function DocsPage({
  params,
}: {
  params: Promise<{ slug?: string[] }>;
}) {
  const { slug } = await params;
  const path = slug?.join("/") || "index";

  const entry = getDocsContent(path);

  if (!entry) {
    notFound();
  }

  return (
    <FumadocsDocsPage toc={entry.toc}>
      <DocsTitle>{entry.title}</DocsTitle>
      {entry.description && <DocsDescription>{entry.description}</DocsDescription>}
      <DocsBody>
        <RenderMarkdown content={entry.content} />
      </DocsBody>
    </FumadocsDocsPage>
  );
}

export async function generateStaticParams() {
  const slugs = getDocsSlugs();
  // Add the root docs path (index)
  return [{ slug: [] }, ...slugs.map((s) => ({ slug: [s] }))];
}

export async function generateMetadata({
  params,
}: {
  params: Promise<{ slug?: string[] }>;
}) {
  const { slug } = await params;
  const path = slug?.join("/") || "index";

  const entry = getDocsContent(path);

  if (!entry) {
    return { title: "Not Found" };
  }

  return {
    title: entry.title,
    description: entry.description,
  };
}