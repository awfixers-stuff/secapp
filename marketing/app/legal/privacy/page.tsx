import { RenderMarkdown } from "@/components/render-markdown";
import { getLegalContent } from "@/lib/content";
import { ContentPage } from "@/components/content-page";

const slug = "privacy";

export default async function PrivacyPage() {
  const entry = getLegalContent(slug);

  if (!entry) {
    return (
      <ContentPage title="Privacy Policy">
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