import fs from "node:fs";
import path from "node:path";
import matter from "gray-matter";
import type { TOCItemType } from "fumadocs-core/toc";

export interface ContentEntry {
  slug: string;
  title: string;
  description?: string;
  content: string;
  data: Record<string, unknown>;
  toc: TOCItemType[];
}

const DOCS_DIR = path.join(process.cwd(), "docs");
const CONTENT_DIR = path.join(process.cwd(), "content");
const INFO_DIR = path.join(process.cwd(), "info");

/**
 * Strip leading H1 from markdown content so it doesn't duplicate
 * the page title rendered by the ContentPage component.
 */
function stripLeadingH1(content: string): string {
  return content.replace(/^#\s+.+\n*/m, "");
}

/**
 * Extract a table of contents from markdown content.
 * Parses ATX-style headings (## and ###) and generates
 * URL-friendly slugs for anchor linking.
 */
function extractToc(content: string): TOCItemType[] {
  const headings: TOCItemType[] = [];
  const seenSlugs = new Map<string, number>();

  for (const line of content.split("\n")) {
    const match = line.match(/^(#{2,4})\s+(.+)/);
    if (!match) continue;

    const level = match[1].length;
    const title = match[2].replace(/[*_`~]/g, "").trim();
    const slug = slugifyHeading(title, seenSlugs);

    headings.push({
      title,
      url: `#${slug}`,
      depth: level,
    });
  }

  return headings;
}

function slugifyHeading(title: string, seenSlugs: Map<string, number>): string {
  const base = title
    .toLowerCase()
    .replace(/[^\w\s-]/g, "")
    .replace(/\s+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "");

  const count = seenSlugs.get(base) ?? 0;
  seenSlugs.set(base, count + 1);

  return count > 0 ? `${base}-${count}` : base;
}

function readMarkdownFile(filePath: string): ContentEntry {
  const raw = fs.readFileSync(filePath, "utf-8");
  const { data, content } = matter(raw);

  // gray-matter may parse date-like strings as Date objects;
  // convert them to strings so React can render them.
  const normalized: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(data)) {
    normalized[key] = value instanceof Date ? value.toISOString().split("T")[0] : value;
  }

  const title = (normalized.title as string) || path.basename(filePath, path.extname(filePath));
  const strippedContent = stripLeadingH1(content);

  return {
    slug: path.basename(filePath, path.extname(filePath)),
    title,
    description: normalized.description as string | undefined,
    content: strippedContent,
    data: normalized,
    toc: extractToc(strippedContent),
  };
}

function readDirectory(
  dir: string,
  pattern: RegExp,
): ContentEntry[] {
  if (!fs.existsSync(dir)) return [];

  const entries: ContentEntry[] = [];
  const files = fs.readdirSync(dir);

  for (const file of files) {
    if (!pattern.test(file)) continue;
    const filePath = path.join(dir, file);
    // Skip symlinks that point to directories
    const stat = fs.statSync(filePath);
    if (stat.isDirectory()) continue;
    entries.push(readMarkdownFile(filePath));
  }

  return entries;
}

// --- Info (symlinked to root) ---
const INFO_FILE_MAP: Record<string, { file: string; title: string }> = {
  roadmap: { file: "TODO.md", title: "Roadmap" },
  ideas: { file: "IDEA.md", title: "Ideas" },
  license: { file: "LICENSE.md", title: "AWFixer Source Available License v0.4" },
};

export function getInfoContent(slug: string): ContentEntry | null {
  const entry = INFO_FILE_MAP[slug];
  if (!entry) return null;

  const filePath = path.join(INFO_DIR, entry.file);
  if (!fs.existsSync(filePath)) return null;

  const result = readMarkdownFile(filePath);
  // Override title with the display name from our map
  result.title = entry.title;
  return result;
}

export function getInfoSlugs(): string[] {
  return Object.keys(INFO_FILE_MAP).filter((slug) => {
    const filePath = path.join(INFO_DIR, INFO_FILE_MAP[slug].file);
    return fs.existsSync(filePath);
  });
}

// --- Legal ---
export function getLegalContent(slug: string): ContentEntry | null {
  const filePath = path.join(CONTENT_DIR, "legal", `${slug}.md`);
  if (!fs.existsSync(filePath)) return null;
  return readMarkdownFile(filePath);
}

export function getAllLegalSlugs(): string[] {
  const legalDir = path.join(CONTENT_DIR, "legal");
  if (!fs.existsSync(legalDir)) return [];
  return fs
    .readdirSync(legalDir)
    .filter((f) => f.endsWith(".md"))
    .map((f) => path.basename(f, ".md"));
}

// --- Docs ---
function isMdxFile(name: string): boolean {
  return name.endsWith(".md") || name.endsWith(".mdx");
}

export function getDocsContent(slug: string): ContentEntry | null {
  // Try slug.mdx first, then slug.md
  for (const ext of [".mdx", ".md"]) {
    const filePath = path.join(DOCS_DIR, `${slug}${ext}`);
    if (fs.existsSync(filePath)) {
      return readMarkdownFile(filePath);
    }
  }

  // Try index.mdx/index.md for directory slugs
  for (const ext of [".mdx", ".md"]) {
    const filePath = path.join(DOCS_DIR, slug, `index${ext}`);
    if (fs.existsSync(filePath)) {
      return readMarkdownFile(filePath);
    }
  }

  return null;
}

export function getDocsSlugs(): string[] {
  if (!fs.existsSync(DOCS_DIR)) return [];

  const slugs: string[] = [];
  const files = fs.readdirSync(DOCS_DIR);

  for (const file of files) {
    if (!isMdxFile(file)) continue;

    const slug = file.replace(/\.mdx?$/, "");
    // "index" maps to the root docs path
    if (slug === "index") continue;
    slugs.push(slug);
  }

  return slugs;
}

export function getAllDocs(): ContentEntry[] {
  return readDirectory(DOCS_DIR, /\.mdx?$/);
}