import { RenderMarkdown } from "@/components/render-markdown";
import { getInfoContent } from "@/lib/content";
import { ContentPage } from "@/components/content-page";

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
      <RenderMarkdown content={entry.content} />
    </ContentPage>
  );
}