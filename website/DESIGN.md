# Website design

Rules for `website/`, so later changes stay coherent. The site follows
gpui-kit.com's design language — same restraint, same neutral palette and
code themes — adapted for a mobile framework.

## Principles

1. **Say what it is, immediately.** The headline states the purpose in plain
   words: AI-generated UI, native in your app. nsh is a layer inside native
   apps (AI-generated and dynamically delivered UI first, plugins second,
   standalone apps third), never presented as a way to write whole apps, and
   never compared with other frameworks outside the comparison page. Architecture comes further down.
2. **Show the real thing, in its real place.** The hero is a native host app
   (drawn as such: status bar, navigation bar, chat, input) whose assistant
   answers with a module; only the dashed, tagged region is nsh, and it runs
   the real `demos/home-module.tsx` through the web player. The legend says
   which part is the host and that the module is a web preview. Documentation
   pages run their demos in a phone frame; the code shown is always the
   source of what runs.
3. **Specifics over adjectives.** Every capability card names something a
   reader can check in the repository: a crate, an element, a module, a
   command (`Taffy`, `<list>`, `nsh/animated`, `nsh publish`). Claims track
   `README.md` and `AGENTS.md`; update the copy when they change.
4. **Restraint carries the design.** Hierarchy comes from type scale, weight
   and hairlines, not colour fills or gradients.

## Colour

Tokens live in `src/styles/global.css` (`:root` and `.dark`).

- **The brand is near-black (near-white in dark mode)**: primary buttons,
  focus, the current item. It is the text colour, so it is never used as an
  accent to add interest; kickers and captions use `--muted-foreground`.
- **Blue (`--data-2`, `#3b82f6` / `#419cff`) is for links, the logo's prompt
  and the main-thread hop in the architecture diagram** — things that are
  "live" or "go somewhere". Green (`--success`) marks running demos.
- Callouts take one tint each (note blue, tip green, important violet,
  warning amber, caution red) at 6% fill and a 30% border; nothing else on a
  page is tinted.
- **Code colours come from the Shiki themes** (`src/light.theme.json`,
  `src/dark.theme.json`, macOS classic, shared with gpui-kit). `global.css`
  maps their hex values onto `--code-*` tokens, so fenced blocks, demo
  sources and the home page's code switch theme together. Never hand-colour
  a snippet.
- The phone bezel is dark in both themes: it is the device, not the page.

## Typography

The platform font (SF Pro, Segoe UI, Roboto; PingFang / Noto Sans SC for
Chinese); `ui-monospace` / SF Mono for code. No webfonts. Base 15px.

- Display: `clamp(2.3rem, 4.6vw, 3.75rem)`, weight 680, tracking −0.045em.
- Section heading: `clamp(1.9rem, 3.4vw, 2.8rem)`, weight 660.
- Kickers: 0.68rem mono, uppercase, wide tracking, muted. Small mono labels
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
assistant types, a pending card ticks Compiled → Signature verified →
Sandboxed, then the live module fades in once both the story (≥3.2s) and the
frame have loaded. Entrance motion only, 620ms `cubic-bezier(.16, 1, .3, 1)`;
with reduced motion the final state shows at once. The host app is always
light, like the module, regardless of the page theme.

## Devices and demos

- A demo is shown on a **phone** (`.phone`): the app renders at its real
  390×844 points and the whole device is scaled with `--phone-scale`, so the
  app's own layout never reflows to fit the page.
- The demo panel's bar carries the file name and "Open in new tab"; the stage
  is a faint 24px blueprint grid. Wide panels show the phone and source side
  by side (a container query on `.demo`); narrow ones switch with a segmented
  control. View switching lives in the bar, never inside the device.
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
- The licence is MIT OR Apache-2.0; no version number is shown anywhere.
