import { RenderMarkdown } from "@/components/render-markdown";
import { getLegalContent } from "@/lib/content";
import { ContentPage } from "@/components/content-page";

const slug = "subprocessors";

export default async function SubprocessorsPage() {
  const entry = getLegalContent(slug);

  if (!entry) {
    return (
      <ContentPage title="Subprocessors">
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
      <RenderMarkdown content={entry.content} />
    </ContentPage>
  );
}