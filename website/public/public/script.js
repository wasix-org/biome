// @ts-check
"use strict";
//# Responsive width
let isMobile = false;
window.addEventListener("DOMContentLoaded", () => {
	const mobileMatchMedia = matchMedia("(max-width: 768px)");
	isMobile = mobileMatchMedia.matches;

	mobileMatchMedia.addListener((e) => {
		isMobile = e.matches;

		// Close the mobile sidebar when switching from mobile to desktop
		if (isMobileNavVisible && !isMobile && isMobileNavVisible) {
			toggleMobileSidebar();
		}
	});
});

//# Color scheme switcher

function getCurrentTheme() {
	let currentScheme = window.localStorage.getItem("data-theme");
	if (currentScheme == null) {
		const prefersDarkMode = matchMedia("(prefers-color-scheme: dark)").matches;
		currentScheme = prefersDarkMode ? "dark" : "light";
	}
	return currentScheme;
}

function toggleColorSchemeSwitch(evt) {
	const currentScheme = getCurrentTheme();
	const newScheme = currentScheme === "dark" ? "light" : "dark";
	window.localStorage.setItem("data-theme", newScheme);
	evt.currentTarget.setAttribute("aria-checked", newScheme === "dark");
	document.documentElement.classList.add("transition");
	document.documentElement.setAttribute("data-theme", newScheme);
}

const colorSchemeSwitcher = document.querySelector(".color-scheme-switch");
// rome-ignore lint/js/preferOptionalChaining: netlify's node version does not support optional call expressions
if (colorSchemeSwitcher != null) {
	colorSchemeSwitcher.addEventListener("click", toggleColorSchemeSwitch, false);
}

//# Mobile navigation

const mobileSidebarHandle = document.querySelector(".mobile-handle");
const mobileActiveTargets = document.querySelectorAll(
	".page-header, .page-header-mobile, .docs-sidebar",
);
let isMobileNavVisible = false;
function toggleMobileSidebar() {
	isMobileNavVisible = !isMobileNavVisible;
	mobileSidebarHandle.classList.toggle("active");
	document.body.classList.toggle("no-scroll");
	if (isMobileNavVisible) {
		for (const elem of mobileActiveTargets) {
			elem.classList.add("mobile-active");
		}
	} else {
		for (const elem of mobileActiveTargets) {
			elem.classList.remove("mobile-active");
		}
	}
}
// rome-ignore lint/js/preferOptionalChaining: netlify's node version does not support optional call expressions
if (mobileSidebarHandle != null) {
	mobileSidebarHandle.addEventListener(
		"click",
		(event) => {
			event.preventDefault();
			toggleMobileSidebar();
		},
		false,
	);
}

//# Homepage hero scroller
const heroScrollers = document.querySelectorAll(".homepage .h1 li");
if (heroScrollers.length > 0) {
	let activeIndex = 0;

	function next() {
		const activeElem = heroScrollers[activeIndex];
		activeElem.classList.remove("fadein");
		activeElem.classList.add("fadeout");
		activeElem.addEventListener(
			"animationend",
			() => {
				activeElem.setAttribute("hidden", "hidden");

				let nextActiveIndex = activeIndex + 1;
				if (nextActiveIndex === heroScrollers.length) {
					nextActiveIndex = 0;
				}

				const nextActiveElem = heroScrollers[nextActiveIndex];
				nextActiveElem.classList.add("fadein");
				nextActiveElem.removeAttribute("hidden");

				activeIndex = nextActiveIndex;
				queue();
			},
			{ once: true },
		);
	}

	function queue() {
		setTimeout(() => {
			next();
		}, 2500);
	}

	queue();
}

//# Homepage component switcher
const componentSwitcher = Array.from(
	document.querySelectorAll(".component-list li:not(.soon)"),
);
let activeComponentButton = document.querySelector(".component-list li.active");
for (const button of componentSwitcher) {
	button.addEventListener("click", () => {
		if (activeComponentButton != null) {
			activeComponentButton.classList.remove("active");
			const elems = Array.from(
				document.getElementsByClassName(
					activeComponentButton.getAttribute("data-class"),
				),
			);
			for (const elem of elems) {
				elem.setAttribute("hidden", "hidden");
			}
		}

		button.classList.add("active");
		const elems = Array.from(
			document.getElementsByClassName(button.getAttribute("data-class")),
		);
		for (const elem of elems) {
			elem.removeAttribute("hidden");
		}

		activeComponentButton = button;
	});
}

//# Tweet dark mode
const tweets = Array.from(document.querySelectorAll(".twitter-tweet"));
function updateTweetThemes(scheme) {}
if (tweets.length > 0) {
	const scheme = getCurrentTheme();
	for (const elem of tweets) {
		elem.setAttribute("data-theme", scheme);
	}
}

//# Homepage Prettier progress trigger
/** @type {HTMLDivElement | null}*/
const maybePrettierProgressBar = document.querySelector(
	".homepage .progress-bar-bad",
);
if (maybePrettierProgressBar != null) {
	/** @type {HTMLDivElement}*/
	const prettierProgressBar = maybePrettierProgressBar;

	prettierProgressBar.classList.add("transition");

	/** @type {any}*/
	const totalTimeElem = document.querySelector(".homepage .time-bad");

	function start() {
		const start = Date.now();

		const newTimeElem = document.createElement("span");
		newTimeElem.classList.add("time-bad", "time-bad-timer");
		totalTimeElem.parentElement.insertBefore(newTimeElem, totalTimeElem);
		totalTimeElem.classList.add("timer-running");
		prettierProgressBar.style.width = "100%";

		const interval = setInterval(() => {
			const took = Date.now() - start;
			newTimeElem.textContent = `${(took / 1000).toFixed(1)}s`;
		}, 100);

		function end() {
			clearInterval(interval);
			totalTimeElem.classList.remove("timer-running");
			newTimeElem.remove();
		}

		prettierProgressBar.addEventListener("transitioncancel", end, {
			once: true,
		});

		prettierProgressBar.addEventListener("transitionend", end, { once: true });
	}

	const observer = new window.IntersectionObserver(
		([entry]) => {
			if (entry.isIntersecting) {
				start();
				observer.disconnect();
			}
		},
		{
			root: null,
			threshold: 1,
		},
	);

	observer.observe(prettierProgressBar);
}
