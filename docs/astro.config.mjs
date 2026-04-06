// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import mermaid from 'astro-mermaid';

// Starlight plugins
import starlightUiTweaks from 'starlight-ui-tweaks'
import starlightSidebarSwipe from 'starlight-sidebar-swipe'
import starlightSidebarTopics from 'starlight-sidebar-topics'
import starlightContextualMenu from "starlight-contextual-menu";

// https://astro.build/config
export default defineConfig({
	base: "/floodr",
	site: 'https://kieranwood.ca',
	integrations: [
		starlight({
			title: 'Floodr',
			customCss: [
				'./src/theme.css',
			],
			social: [
				{
					icon: 'github',
					label: 'GitHub',
					href: 'https://github.com/descent098/floodr'
				}
			],
			plugins: [
				starlightUiTweaks({
					navbarLinks: [
						{ 
							label: "Getting Started", 
							href: "/floodr/getting-started" 
						},
						{ 
							label: "CLI", 
							href: "/floodr/cli" 
						},
						// { label: "Library", href: "/floodr/utilities" }, // TODO: Add crate URL when it's available
					],
				}),
				// Enables swiping in the menu from the side on moble
				starlightSidebarSwipe(),

				// Enables view-as and copy-page options
				starlightContextualMenu({
					actions: ["copy", "view", "chatgpt", "claude"]
				}),


				starlightSidebarTopics([
					{
						label: "Basics",
						link: "/floodr/getting-started",
						icon: "seti:html",
						items: [
							{
								label: 'Gettting started',
								autogenerate: { directory: "getting-starte" },
							},
							{
								label: 'Command Line Interface',
								autogenerate: { directory: "cli" },
							},
						],

					},

					{
						label: "Reference",
						link: "/floodr/benchmark-reference",
						icon: "seti:html",
						items: [
							{
								label: 'Benchmark File Reference',
								autogenerate: { directory: "benchmark-reference" },
							},
						],

					},
				],
				), // end of sidebar config
			],
		}),
		mermaid({
			theme: "default",
			autoTheme: true
		})
	],
});
