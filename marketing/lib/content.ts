import fs from "node:fs";
import path from "node:path";
import matter from "gray-matter";

export interface ContentEntry {
  slug: string;
  title: string;
  description?: string;
  content: string;
  data: Record<string, unknown>;
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

  return {
    slug: path.basename(filePath, path.extname(filePath)),
    title,
    description: normalized.description as string | undefined,
    content: stripLeadingH1(content),
    data: normalized,
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