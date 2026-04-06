// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// Markdown Plugins
import mermaid from 'astro-mermaid';
import remarkMath from 'remark-math';
import rehypeMathjax from 'rehype-mathjax';

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
						link: "/getting-started",
						icon: "seti:html",
						items: [
							{
								label: "Getting Started",
								items: [
										{
											label: 'Gettting started',
											link: "/getting-started",
										},						{
											label: 'Installation',
											link: "/getting-started/installation",
										},
								]
							},
							{
								label: 'Usage',
								items: [
									{
										label: "Basic Usage",
										link:"/getting-started/basic-usage"
									},
									{
										label: "Advanced Usage",
										link:"/getting-started/advanced-usage"
									}
								]
							},
						],

					},
					{
						label: "Benchmark Reference",
						link: "/benchmark-reference",
						icon: "document",
						
						items: [
							{
								label: "Basics",
								link: "/benchmark-reference"
							},
							{
								label: 'Actions',
								autogenerate: { directory: "benchmark-reference/actions" },
							},
														{
								label: 'Expandables',
								autogenerate: { directory: "benchmark-reference/expandables" },
							},
						],
					},

					{
						label: "CLI Reference",
						link: "/cli",
						icon: "seti:shell",
						
						items: [
							{
								label: "Introduction",
								link: "/cli"
							},
							{
								label: 'Comparisons & Reports',
								autogenerate: { directory: "cli/comparisons" },
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
		}),
	],
	markdown: {
		remarkPlugins: [remarkMath],
		rehypePlugins: [rehypeMathjax],
	},
});
