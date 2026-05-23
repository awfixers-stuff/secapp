import { Markdown } from "fumadocs-core/content/md";
import { getLegalContent } from "@/lib/content";
import { ContentPage } from "@/components/content-page";
import { mdxComponents } from "@/components/mdx-components";

const slug = "terms";

export default async function TermsPage() {
  const entry = getLegalContent(slug);

  if (!entry) {
    return (
      <ContentPage title="Terms of Service">
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