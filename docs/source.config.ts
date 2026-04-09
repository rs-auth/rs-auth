import { defineDocs, defineConfig } from "fumadocs-mdx/config";
import vesperLight from "./src/themes/vesper-light.json" with { type: "json" };

export const docs = defineDocs({
  dir: "content/docs",
  docs: {
    async: true,
  },
});

export default defineConfig({
  mdxOptions: {
    rehypeCodeOptions: {
      defaultColor: false,
      themes: {
        light: vesperLight as never,
        dark: "vesper",
      },
    },
  },
});
