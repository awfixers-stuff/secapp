import { Markdown } from "fumadocs-core/content/md";
import { getDocsContent, getDocsSlugs } from "@/lib/content";
import { notFound } from "next/navigation";
import { ContentPage } from "@/components/content-page";
import { mdxComponents } from "@/components/mdx-components";

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
    <ContentPage
      title={entry.title}
      description={entry.description}
      lastUpdated={entry.data.lastUpdated as string | undefined}
    >
      <Markdown components={mdxComponents}>{entry.content}</Markdown>
    </ContentPage>
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