import { codeToHtml, type BundledLanguage } from "shiki";
import vesperLight from "@/themes/vesper-light.json";

export async function HighlightedCode({
	code,
	lang,
}: {
	code: string;
	lang: BundledLanguage;
}) {
	const html = await codeToHtml(code, {
		lang,
		defaultColor: false,
		themes: {
			light: vesperLight as never,
			dark: "vesper",
		},
	});

	// biome-ignore lint/security/noDangerouslySetInnerHtml: Shiki returns trusted highlighted markup.
	return <div dangerouslySetInnerHTML={{ __html: html }} />;
}
