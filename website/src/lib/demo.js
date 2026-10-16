import { createHash } from 'node:crypto';
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { createHighlighter } from 'shiki';
import { themes } from './markdown.js';

// Demos are small nsh apps in `website/demos/<name>.tsx`. A page names one in
// its frontmatter (`demo: counter`); the site shows it running in a phone
// frame, through the web player in `public/play/`, next to its source. Until
// the player is built, pages show the source alone (decided at build time).

const ROOT = process.cwd();
const DEMOS = join(ROOT, 'demos');
const PLAYER = join(ROOT, 'public', 'play', 'index.html');

/** The app's logical size: an iPhone's 390×844 points. */
export const SCREEN = { width: 390, height: 844 };

export function demoFile(name) {
  return join(DEMOS, `${name}.tsx`);
}

export function demoSource(name) {
  try {
    return readFileSync(demoFile(name), 'utf8').replace(/\s+$/, '');
  } catch {
    return null;
  }
}

export function hasPlayer() {
  return existsSync(PLAYER);
}

/**
 * Changes when a demo source or the player's presence changes. It goes into the
 * markdown options, so Astro's content cache (keyed by the config) re-renders
 * pages instead of keeping a placeholder after the player is added.
 */
export function demoState() {
  const hash = createHash('sha1');
  hash.update(String(hasPlayer()));
  // The markup itself, so editing this file re-renders the pages too.
  hash.update(readFileSync(new URL(import.meta.url), 'utf8'));
  try {
    for (const name of readdirSync(DEMOS).sort()) {
      if (name.endsWith('.tsx')) hash.update(`${name}:${statSync(join(DEMOS, name)).mtimeMs}`);
    }
  } catch {}
  return hash.digest('hex').slice(0, 12);
}

let highlighter;

export async function highlight(code, lang = 'tsx') {
  highlighter ??= createHighlighter({ themes: [themes.light, themes.dark], langs: ['tsx', 'sh'] });
  const html = (await highlighter).codeToHtml(code, {
    lang,
    themes: { light: themes.light.name, dark: themes.dark.name },
    defaultColor: 'light',
  });
  // The same class Astro gives fenced blocks, so the same token rules apply.
  return html.replace('<pre class="shiki', '<pre class="astro-code shiki');
}

const escape = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

const COPY = {
  en: {
    preview: 'Preview',
    source: 'Source',
    open: 'Open in new tab',
    edit: 'Edit in Playground',
    frame: (name) => `Live demo: ${name}`,
    pending: 'The live preview is not built yet.',
    runLocally: 'Run it on a simulator or device:',
    missing: (name) => `Demo “${name}” was not found in website/demos.`,
    label: 'Demo',
  },
  'zh-CN': {
    preview: '预览',
    source: '源码',
    open: '在新标签页打开',
    edit: '在线编辑',
    frame: (name) => `在线示例：${name}`,
    pending: '在线预览尚未构建。',
    runLocally: '在模拟器或真机上运行：',
    missing: (name) => `website/demos 中没有名为“${name}”的示例。`,
    label: '示例',
  },
};

export function playerHref(name, base = '/') {
  return `${base}play/index.html?theme=dark#${encodeURIComponent(name)}`;
}

/**
 * A phone at 390×844 points, scaled to fit (`--phone-scale`), with the demo
 * running inside, or a placeholder screen when there is no player.
 */
export function phoneHtml(name, { base = '/', lang = 'en', eager = false } = {}) {
  const t = COPY[lang] ?? COPY.en;
  const screen = hasPlayer()
    ? `<iframe src="${escape(playerHref(name, base))}" title="${escape(t.frame(name))}" loading="${eager ? 'eager' : 'lazy'}" width="${SCREEN.width}" height="${SCREEN.height}"></iframe>`
    : `<div class="phone__placeholder">
        <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="6" y="2.5" width="12" height="19" rx="2.5"/><path d="M10.5 18.5h3"/></svg>
        <p class="phone__placeholder-title">${escape(t.pending)}</p>
        <p>${escape(t.runLocally)}</p>
        <code>nsh run website/demos/${escape(name)}.tsx</code>
      </div>`;
  return `<div class="phone" style="--phone-w:${SCREEN.width};--phone-h:${SCREEN.height}"><div class="phone__screen">${screen}</div></div>`;
}

/** The demo block of a documentation page: phone and source behind a segmented control. */
export async function demoPanelHtml(name, { base = '/', lang = 'en' } = {}) {
  const t = COPY[lang] ?? COPY.en;
  const source = demoSource(name);
  if (source === null) {
    return `<div class="demo demo--missing" role="note">${escape(t.missing(name))}</div>`;
  }
  const code = await highlight(source);
  const file = `${name}.tsx`;
  const id = `demo-${name.replace(/[^a-z0-9_-]/gi, '-')}`;

  if (!hasPlayer()) {
    return `<figure class="demo demo--source-only" data-demo="${escape(name)}" data-pagefind-ignore>
      <figcaption class="demo__bar">
        <span class="demo__label">${escape(t.label)}</span>
        <span class="demo__file">${escape(file)}</span>
      </figcaption>
      <div class="demo__code">${code}</div>
    </figure>`;
  }

  return `<figure class="demo" data-demo="${escape(name)}" data-view="preview" data-pagefind-ignore>
    <figcaption class="demo__bar">
      <div class="segmented" role="tablist" aria-label="${escape(t.label)}">
        <button type="button" role="tab" id="${id}-tab-preview" aria-controls="${id}-preview" aria-selected="true" data-demo-view="preview">${escape(t.preview)}</button>
        <button type="button" role="tab" id="${id}-tab-source" aria-controls="${id}-source" aria-selected="false" tabindex="-1" data-demo-view="source">${escape(t.source)}</button>
      </div>
      <span class="demo__file">${escape(file)}</span>
      <a class="demo__open" href="${escape(`${base}${lang === 'zh-CN' ? 'zh-CN/' : ''}playground#demo=${encodeURIComponent(name)}`)}">${escape(t.edit)}<svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 20h9M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z"/></svg></a>
      <a class="demo__open" href="${escape(playerHref(name, base))}" target="_blank" rel="noopener">${escape(t.open)}<svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M7 17 17 7M8 7h9v9"/></svg></a>
    </figcaption>
    <div class="demo__body">
    <div class="demo__stage" id="${id}-preview" role="tabpanel" aria-labelledby="${id}-tab-preview" data-active>${phoneHtml(name, { base, lang })}</div>
    <div class="demo__code" id="${id}-source" role="tabpanel" aria-labelledby="${id}-tab-source">${code}</div>
    </div>
  </figure>`;
}
