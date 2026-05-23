import { RenderMarkdown } from "@/components/render-markdown";
import { getInfoContent } from "@/lib/content";
import { ContentPage } from "@/components/content-page";

const slug = "license";

export default async function LicensePage() {
  const entry = getInfoContent(slug);

  if (!entry) {
    return (
      <ContentPage title="License">
        <p>Content not found.</p>
      </ContentPage>
    );
  }

  return (
    <ContentPage
      title="AWFixer Source Available License v0.4"
      description="The license governing your use of secapp"
      lastUpdated={entry.data.lastUpdated as string | undefined}
    >
      <RenderMarkdown content={entry.content} />
    </ContentPage>
  );
}