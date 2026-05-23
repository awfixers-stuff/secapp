import { defineCollections, defineDocs, frontmatterSchema } from "fumadocs-mdx/config";

export const docs = defineDocs({
  dir: "docs",
  docs: {
    schema: frontmatterSchema,
  },
});

export const info = defineCollections({
  type: "doc",
  dir: "info",
  schema: frontmatterSchema,
});

export const legal = defineCollections({
  type: "doc",
  dir: "content/legal",
  schema: frontmatterSchema,
});