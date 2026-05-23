import { Markdown } from "fumadocs-core/content/md";
import { getInfoContent } from "@/lib/content";
import { ContentPage } from "@/components/content-page";
import { mdxComponents } from "@/components/mdx-components";

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
      <Markdown components={mdxComponents}>{entry.content}</Markdown>
    </ContentPage>
  );
}