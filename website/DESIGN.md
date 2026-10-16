# Website design

Rules for `website/`, so later changes stay coherent. The site follows the
gallery's design language (`examples/gallery/ui.tsx`): graphite surfaces lit
from above, one citrus accent, tinted status colours — neutral and refined,
the look of the UI nsh draws.

## Principles

1. **Say what it is, immediately.** The headline states the purpose in plain
   words: AI-generated UI, native in your app. nsh is a layer inside native
   apps (AI-generated and dynamically delivered UI first, plugins second,
   standalone apps third), never presented as a way to write whole apps, and
   never compared with other frameworks outside the comparison page. Architecture comes further down.
2. **Show the real thing, in its real place.** The hero is a native host app
   (a chat and its composer) whose assistant answers with a module; only the
   dashed, tagged region is nsh, and it runs the real `demos/home-module.tsx`
   (a live watchlist in the gallery's Stocks look, fictional tickers)
   through the web player. The legend says which part is the host and that
   the module is a web preview. Documentation pages run their demos in a
   frame at phone width; the code shown is always the source of what runs.
   The product is written "Native Shell" where it is a name; `nsh` and
   `native-shell` stay as they are in code, commands and URLs.
3. **Specifics over adjectives.** Every capability card names something a
   reader can check in the repository: a crate, an element, a module, a
   command (`Taffy`, `<List>`, `nsh/animated`, `nsh publish`). Claims track
   `README.md` and `AGENTS.md`; update the copy when they change.
4. **Restraint carries the design.** Hierarchy comes from type scale, weight,
   hairlines and surface depth (card, inset, raised), not colour fills. The
   only gradients are the surfaces' own top-to-bottom light.

## Colour

Tokens live in `src/styles/global.css`: `:root` holds the dark theme,
`html.light` its light variant. A pre-paint script in `BaseLayout.astro`
sets `dark` or `light` on `<html>` from `localStorage.theme` ("light",
"dark", or absent for the system's), so nothing flashes; the nav's
System / Light / Dark segmented control (`ThemeSwitch.astro`) sets it.

The light variant keeps the design and changes the surfaces: page
`#f4f4f2`, card `#ffffff` (a `#ffffff → #fafaf8` gradient, a `#e4e4e0`
border, soft shadow), inset `#f0f0ec` recessed, raised `#fbfbf9 → #f0f0ec`
with a top highlight; text `#16171a`, muted `#5f636a`, faint `#9a9ea4`. The
accent token (`--citrus`) becomes `#5a7a12` so it reads as text and lines;
the bright citrus stays as the fill of accent badges and buttons
(`--finish-accent-bg`, dark text). Primary buttons are dark on light.
The values below are the dark theme's.

- **Surfaces**, as in the gallery: page `#101113`, card `#18191c`, inset
  `#0c0d0f`, raised `#24262a`; border `#2c2e33`, hairline `#1f2124`; text
  `#eceef1`, muted `#8d9198`, faint `#5a5e65`.
- **Finishes** (`--finish-*`, the gallery's `finish`): cards take a
  `#1d1e22 → #17181b` gradient, an inner top highlight and a soft shadow;
  insets (code, the demo stage, the search field, segmented tracks) are
  recessed with an inner shadow; raised controls (secondary buttons, the
  current nav and sidebar item, kbd, segmented thumbs) have a top highlight.
  Primary buttons are light (`#fbfbfc → #d4d6d9`, dark text).
- **One accent, citrus `#c8f26d`** (text on it `#13160c`): the logo, links,
  kickers, the current TOC entry, the nsh module's tag in the hero, the
  accent of the diagrams. Never a large fill.
- Status colours: up / success `#45d49a`, danger `#ff6170`, info `#7ea8ff`,
  warn `#ffb547`, as small tinted badges or dots. Callouts take one tint
  each (note info, tip green, important violet, warning amber, caution red)
  over the card colour; nothing else on a page is tinted.
- **Code colours are tokens** (`--code-*`): Shiki emits the macos-classic
  light theme's hex values (`src/light.theme.json`) and `global.css` maps
  them onto the tokens, so fenced blocks, demo sources and the home page's
  code share one palette. Never hand-colour a snippet.
- Diagrams in `public/diagrams` come as a light and a dark file, each with
  `:root` tokens in its theme's palette; the page shows the one of its theme.
- Radii: cards 16px, insets 12px, controls 10px.
- The logo is the app icon's mark (`public/logo.svg`, inline in
  `Logo.astro`): a citrus arch holding a light dot (a dark one on the light
  theme, `--logo-dot`). The favicon
  (`favicon.svg`), `apple-touch-icon.png` and `icon-512.png` (og:image) are
  the full icon on graphite.

## Typography

The platform font (SF Pro, Segoe UI, Roboto; PingFang / Noto Sans SC for
Chinese); `ui-monospace` / SF Mono for code. No webfonts. Base 15px.

- Display: `clamp(2.3rem, 4.6vw, 3.75rem)`, weight 680, tracking −0.045em.
- Section heading: `clamp(1.9rem, 3.4vw, 2.8rem)`, weight 660.
- Kickers: 0.68rem mono, uppercase, wide tracking, faint (citrus on the
  home page's sections). Small mono labels
  — not colour — mark structure (section kicker, sidebar group, "On this page").
- Negative and wide tracking are for Latin only: `html[lang^="zh"]` resets
  them.

## Layout

- Page width 1280px, gutter 1.5rem (1rem under 640px). Nav, hero, bands and
  footer share the same left edge.
- Home bands are full-bleed sections separated only by a hairline
  (`border-top`), content in `.band__inner`.
- Docs: sidebar (232px) · article (prose capped at 46rem) · "On this page"
  (208px). The TOC hides under 1180px; under 960px the sidebar becomes a
  sticky bar naming the section and page, opening a drop-down list; under
  860px the section links move behind the burger.
- Grid columns use `minmax(0, 1fr)`; check phone widths for horizontal
  overflow (the page's `scrollWidth` must equal the viewport).

## The hero story

The hero plays once on load (Replay restarts it): the question rises, the
assistant types, the module's card shows one "Generating module…" row with
a spinner and a thin filling line, then the live module fades in once both
the story (≥3.2s) and the frame have loaded. Entrance motion only, 620ms `cubic-bezier(.16, 1, .3, 1)`;
with reduced motion the final state shows at once. The host app sits in a
card frame of the site's design (a slim bar with the app's name and a "Host
app" badge, the app on an inset surface; no device chrome) and follows the
page's theme, the module too (the player's `?theme=`, then a message on
every change): the user's bubble and the assistant's avatar citrus, the
module on a card with the card finish, its dashed region and tag citrus, the
composer a recessed field with a round primary send button.

Behind the hero, `HeroField.astro` draws the dot grid on a 2D canvas: a slow
wave runs through it, and near the pointer (or the last touch) dots swell
into the accent under an eased glow, faint over the copy. It pauses off
screen and in hidden tabs, draws one static frame with reduced motion,
never takes pointer events, and costs about 0.2 ms a frame (`?fieldperf`
logs it).

## Demo frames

- Demos (`demos/*.tsx`) and the playground run in the dark appearance in
  both themes (the hero's module follows the theme) and
  are drawn with the gallery's palette and finishes too (inline hex values
  or arbitrary Tailwind values, never custom class names: the playground
  compiles without `demos/nsh.json`, which themes `nsh/ui/mobile`).
- A demo is shown in a **frame** (`.phone`, `frameBar()` in
  `src/lib/demo.js`): a card with the card finish, a slim bar (an icon tile,
  the file name, a "Live" badge) and the app on an inset surface, no device
  chrome. The app renders at its real 390×844 points and the frame is
  scaled with `--phone-scale`, so the app's own layout never reflows.
- The demo panel's bar carries the file name and "Open in new tab"; the stage
  is a faint 24px blueprint grid. Wide panels show the frame and source side
  by side (a container query on `.demo`); narrow ones switch with a segmented
  control. View switching lives in the panel's bar, never inside the frame.
- Code shown as "a file you wrote" uses the macOS window frame
  (`.code-window`): traffic lights, centred title.

## Motion

Entrances only: `rise` 620ms `cubic-bezier(.16, 1, .3, 1)`, staggered 70ms,
inside `prefers-reduced-motion: no-preference`. The live dot pulses slowly
(2.4s). Nothing conveys meaning through motion alone.

## Content

- English and Chinese pages are written together; same file names, same
  `order` and `demo` (`tests/content.test.ts`).
- Home copy is the bilingual `copy` object in `src/components/Home.astro`;
  section names and blurbs are in `src/lib/sections.ts`.
- The licence is Apache-2.0; no version number is shown anywhere.
