import rss from "@astrojs/rss";

export const get = () =>
	rss({
		title: "Rome Blog",
		description: "",
		site: import.meta.env.SITE,
		items: import.meta.glob("./blog/**/*.mdx"),
		customData: `<language>en-us</language>`,
	});
