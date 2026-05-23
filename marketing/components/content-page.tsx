import { type ReactNode } from "react";

interface ContentPageProps {
  title: string;
  description?: string;
  lastUpdated?: string;
  children: ReactNode;
}

export function ContentPage({ title, description, lastUpdated, children }: ContentPageProps) {
  return (
    <article className="mx-auto max-w-3xl px-4 py-10">
      <header className="mb-8">
        <h1 className="text-3xl font-bold tracking-tight">{title}</h1>
        {description && (
          <p className="mt-2 text-lg text-muted-foreground">{description}</p>
        )}
        {lastUpdated && (
          <p className="mt-2 text-sm text-muted-foreground">
            Last updated: {lastUpdated}
          </p>
        )}
      </header>
      <div className="prose-content">{children}</div>
    </article>
  );
}