import type { MDXComponents } from "mdx/types";
import defaultComponents from "fumadocs-ui/mdx";

import { Callout } from "@/components/callout";
import { CodeFrame } from "@/components/code-frame";
import { Endpoint } from "@/components/endpoint";
import { MethodBadge } from "@/components/method-badge";

export function getMDXComponents(): MDXComponents {
  return {
    ...defaultComponents,
    Callout,
    CodeFrame,
    Endpoint,
    MethodBadge,
  };
}
