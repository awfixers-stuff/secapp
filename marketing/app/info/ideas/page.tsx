import { Markdown } from "fumadocs-core/content/md";
import { getInfoContent } from "@/lib/content";
import { ContentPage } from "@/components/content-page";
import { mdxComponents } from "@/components/mdx-components";

const slug = "ideas";

export default async function IdeasPage() {
  const entry = getInfoContent(slug);

  if (!entry) {
    return (
      <ContentPage title="Ideas">
        <p>Content not found.</p>
      </ContentPage>
    );
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