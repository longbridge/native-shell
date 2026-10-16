/** The page's theme, as the player takes it (`?theme=`). */
export const siteTheme = () => (document.documentElement.classList.contains('light') ? 'light' : 'dark');

/** Calls `change` with the page's theme whenever it changes. */
export function onThemeChange(change: (theme: 'light' | 'dark') => void) {
  let last = siteTheme();
  new MutationObserver(() => {
    const now = siteTheme();
    if (now !== last) change((last = now));
  }).observe(document.documentElement, { attributes: true, attributeFilter: ['class'] });
}

/** Demos follow the page's theme: the player's `?theme=` before they load
 * (and in "Open in new tab" links), a message when the theme changes, so a
 * running demo switches without reloading. */
function themeDemos(root: ParentNode) {
  const themed = (url: string) => url.replace(/([?&]theme=)(dark|light)/, `$1${siteTheme()}`);
  const frames = [...root.querySelectorAll<HTMLIFrameElement>('.phone__screen iframe[src*="theme="]')];
  const links = [...root.querySelectorAll<HTMLAnchorElement>('a[href*="play/index.html?theme="]')];
  const apply = () => {
    const want = siteTheme();
    for (const link of links) link.href = themed(link.href);
    for (const frame of frames) {
      if (frame.dataset.theme === want) continue;
      // Running: tell the player; not loaded yet (lazy): change the URL.
      if (frame.dataset.loaded) frame.contentWindow?.postMessage({ theme: want }, '*');
      else frame.setAttribute('src', themed(frame.getAttribute('src')!));
      frame.dataset.theme = want;
    }
  };
  for (const frame of frames) frame.dataset.theme = /theme=light/.test(frame.getAttribute('src')!) ? 'light' : 'dark';
  for (const frame of frames) frame.addEventListener('load', () => (frame.dataset.loaded = '1'));
  apply();
  onThemeChange(apply);
}

/** The Preview / Source switch of demo panels (markup from `demo.js`). */
export function initDemos(root: ParentNode = document) {
  themeDemos(root);
  for (const demo of root.querySelectorAll<HTMLElement>('.demo[data-view]')) {
    const tabs = [...demo.querySelectorAll<HTMLButtonElement>('[data-demo-view]')];
    const show = (view: string, focus = false) => {
      demo.dataset.view = view;
      for (const tab of tabs) {
        const selected = tab.dataset.demoView === view;
        tab.setAttribute('aria-selected', String(selected));
        tab.tabIndex = selected ? 0 : -1;
        const panel = document.getElementById(tab.getAttribute('aria-controls') ?? '');
        // Not `hidden`: on wide panels CSS shows both, which `hidden` (an
        // !important preflight rule) would forbid.
        if (panel) panel.toggleAttribute('data-active', selected);
        if (selected && focus) tab.focus();
      }
    };
    for (const tab of tabs) {
      tab.addEventListener('click', () => show(tab.dataset.demoView!));
      tab.addEventListener('keydown', (event) => {
        if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return;
        event.preventDefault();
        const at = tabs.indexOf(tab);
        const next = tabs[(at + (event.key === 'ArrowRight' ? 1 : tabs.length - 1)) % tabs.length];
        show(next.dataset.demoView!, true);
      });
    }
  }
}
