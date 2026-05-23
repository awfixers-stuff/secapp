import fs from "node:fs";
import path from "node:path";
import type { Root, Folder, Item, Separator } from "fumadocs-core/page-tree";

/**
 * Build the complete page tree for the docs sidebar.
 *
 * Scans docs/, info/, and content/legal/ at build time,
 * returning a fumadocs-compatible page tree with three
 * top-level sections: Documentation, Info, Legal.
 */
export function buildPageTree(): Root {
  const cwd = process.cwd();
  const docsDir = path.join(cwd, "docs");
  const infoDir = path.join(cwd, "info");
  const legalDir = path.join(cwd, "content", "legal");

  const children: (Folder | Item | Separator)[] = [];

  // --- Documentation section ---
  const docsChildren: (Item | Folder)[] = [];

  if (fs.existsSync(docsDir)) {
    const docsFiles = scanMdxDir(docsDir);
    for (const file of docsFiles) {
      // Skip index from the sidebar — it's the landing page
      if (file.slug === "index") continue;

      const frontmatter = readFrontmatter(file.filePath);
      docsChildren.push({
        type: "page",
        name: (frontmatter.title as string) || titleCase(file.slug),
        url: `/docs/${file.slug}`,
      });
    }
  }

  if (docsChildren.length > 0) {
    children.push({
      type: "folder",
      name: "Documentation",
      defaultOpen: true,
      index: {
        type: "page",
        name: "Documentation",
        url: "/docs",
      },
      children: docsChildren,
    });
  }

  children.push({ type: "separator", name: "Info" });

  // --- Info section (symlinked to root files) ---
  const infoMap: Record<string, { file: string; title: string; url: string }> = {
    roadmap: { file: "TODO.md", title: "Roadmap", url: "/info/roadmap" },
    ideas: { file: "IDEA.md", title: "Ideas", url: "/info/ideas" },
    license: { file: "LICENSE.md", title: "License", url: "/legal/license" },
  };

  const infoItems: Item[] = [];
  for (const [, entry] of Object.entries(infoMap)) {
    const filePath = path.join(infoDir, entry.file);
    if (fs.existsSync(filePath)) {
      infoItems.push({
        type: "page",
        name: entry.title,
        url: entry.url,
      });
    }
  }

  for (const item of infoItems) {
    children.push(item);
  }

  children.push({ type: "separator", name: "Legal" });

  // --- Legal section ---
  if (fs.existsSync(legalDir)) {
    const legalFiles = scanMdxDir(legalDir);
    for (const file of legalFiles) {
      const frontmatter = readFrontmatter(file.filePath);
      children.push({
        type: "page",
        name: (frontmatter.title as string) || titleCase(file.slug),
        url: `/legal/${file.slug}`,
      });
    }
  }

  return {
    name: "secapp",
    children,
  };
}

interface ScanResult {
  slug: string;
  filePath: string;
}

function scanMdxDir(dir: string): ScanResult[] {
  const results: ScanResult[] = [];
  if (!fs.existsSync(dir)) return results;

  const files = fs.readdirSync(dir).sort();
  for (const file of files) {
    if (!file.endsWith(".md") && !file.endsWith(".mdx")) continue;
    const filePath = path.join(dir, file);
    const stat = fs.statSync(filePath);
    if (stat.isDirectory()) continue;

    const slug = file.replace(/\.mdx?$/, "");
    results.push({ slug, filePath });
  }

  return results;
}

function readFrontmatter(filePath: string): Record<string, unknown> {
  try {
    const raw = fs.readFileSync(filePath, "utf-8");
    const match = raw.match(/^---\n([\s\S]*?)\n---/);
    if (!match) return {};

    // Simple YAML frontmatter parser — handles title, description, etc.
    const data: Record<string, unknown> = {};
    const lines = match[1].split("\n");
    for (const line of lines) {
      const colonIdx = line.indexOf(":");
      if (colonIdx === -1) continue;
      const key = line.slice(0, colonIdx).trim();
      let value: string = line.slice(colonIdx + 1).trim();
      // Remove quotes
      if (
        (value.startsWith('"') && value.endsWith('"')) ||
        (value.startsWith("'") && value.endsWith("'"))
      ) {
        value = value.slice(1, -1);
      }
      data[key] = value;
    }
    return data;
  } catch {
    return {};
  }
}

function titleCase(slug: string): string {
  return slug
    .split("-")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}