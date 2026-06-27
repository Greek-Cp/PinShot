import { invoke } from '@tauri-apps/api/core';

// The saved-image preview toast. Slides in at the screen's bottom-right with a
// thumbnail of the just-saved pin, auto-dismisses after a few seconds, and on
// double-click offers Show in Finder / Copy Path / Delete.

const path = decodeURIComponent(new URLSearchParams(location.search).get('path') ?? '');
const filename = path.split('/').pop() ?? 'image.png';

let dismissTimer = 0;

const card = document.createElement('div');
const menu = document.createElement('div');

async function closeWindow(): Promise<void> {
  try {
    await invoke('close_preview');
  } catch (e) {
    console.error('close_preview failed', e);
  }
}

function slideOutThenClose(): void {
  card.style.transform = 'translateX(120%)';
  card.style.opacity = '0';
  setTimeout(() => void closeWindow(), 280);
}

function scheduleDismiss(ms = 4500): void {
  clearTimeout(dismissTimer);
  dismissTimer = window.setTimeout(slideOutThenClose, ms);
}

function makeMenuItem(label: string, onClick: () => void): HTMLButtonElement {
  const b = document.createElement('button');
  b.textContent = label;
  b.style.cssText =
    'display:block;width:100%;text-align:left;appearance:none;border:0;background:none;color:#fff;' +
    'font:13px/1 system-ui,sans-serif;padding:9px 14px;cursor:pointer;';
  b.addEventListener('mouseenter', () => (b.style.background = 'rgba(255,255,255,0.12)'));
  b.addEventListener('mouseleave', () => (b.style.background = 'none'));
  b.addEventListener('click', (e) => {
    e.stopPropagation();
    onClick();
  });
  return b;
}

function showMenu(show: boolean): void {
  menu.style.display = show ? 'block' : 'none';
  if (show) {
    clearTimeout(dismissTimer); // keep it open while the menu is up
  } else {
    scheduleDismiss(2500);
  }
}

function build(): void {
  card.style.cssText =
    'position:absolute;left:12px;top:12px;right:12px;bottom:12px;background:#1c1c1e;' +
    'border:1px solid rgba(255,255,255,0.10);border-radius:14px;box-shadow:0 12px 34px rgba(0,0,0,0.55);' +
    'display:flex;flex-direction:column;overflow:hidden;' +
    'transform:translateX(120%);opacity:0;transition:transform .28s cubic-bezier(.2,.8,.2,1),opacity .28s;';

  const header = document.createElement('div');
  header.style.cssText =
    'display:flex;align-items:center;gap:8px;padding:10px 12px 8px;color:#fff;flex:0 0 auto;';
  const check = document.createElement('span');
  check.textContent = '✓';
  check.style.cssText =
    'display:inline-grid;place-items:center;width:18px;height:18px;border-radius:50%;' +
    'background:#34c759;color:#fff;font:700 11px/1 system-ui;flex:0 0 auto;';
  const titleWrap = document.createElement('div');
  titleWrap.style.cssText = 'min-width:0;';
  const title = document.createElement('div');
  title.textContent = 'Saved to PinShots';
  title.style.cssText = 'font:600 12px/1.2 system-ui;';
  const sub = document.createElement('div');
  sub.textContent = filename;
  sub.style.cssText =
    'font:11px/1.3 system-ui;color:rgba(255,255,255,0.55);overflow:hidden;text-overflow:ellipsis;white-space:nowrap;';
  titleWrap.append(title, sub);
  header.append(check, titleWrap);

  const thumbWrap = document.createElement('div');
  thumbWrap.style.cssText =
    'flex:1 1 auto;margin:0 12px 12px;border-radius:8px;overflow:hidden;background:rgba(255,255,255,0.04);' +
    'display:grid;place-items:center;';
  const img = document.createElement('img');
  img.style.cssText = 'max-width:100%;max-height:100%;object-fit:contain;display:block;';
  thumbWrap.appendChild(img);

  // Action menu (hidden until double-click).
  menu.style.cssText =
    'position:absolute;left:16px;bottom:16px;background:#2c2c2e;border:1px solid rgba(255,255,255,0.12);' +
    'border-radius:10px;box-shadow:0 8px 24px rgba(0,0,0,0.5);padding:4px;display:none;z-index:5;min-width:160px;';
  menu.append(
    makeMenuItem('Show in Finder', () => {
      void invoke('reveal_in_finder', { path }).catch((e) => console.error(e));
      showMenu(false);
    }),
    makeMenuItem('Copy Path', () => {
      void invoke('copy_path', { path }).catch((e) => console.error(e));
      showMenu(false);
    }),
    makeMenuItem('Delete', () => {
      void invoke('delete_file', { path }).catch((e) => console.error(e));
    }),
  );

  card.append(header, thumbWrap, menu);
  document.body.appendChild(card);

  // Double-click anywhere on the card toggles the action menu.
  card.addEventListener('dblclick', (e) => {
    e.preventDefault();
    showMenu(menu.style.display !== 'block');
  });
  // A single click elsewhere dismisses an open menu.
  card.addEventListener('click', (e) => {
    if (menu.style.display === 'block' && !menu.contains(e.target as Node)) {
      showMenu(false);
    }
  });
  // Hovering keeps the toast around; leaving restarts the dismiss countdown.
  card.addEventListener('mouseenter', () => clearTimeout(dismissTimer));
  card.addEventListener('mouseleave', () => {
    if (menu.style.display !== 'block') {
      scheduleDismiss(2000);
    }
  });

  void loadThumb(img);
}

async function loadThumb(img: HTMLImageElement): Promise<void> {
  try {
    img.src = await invoke<string>('read_image', { path });
  } catch (e) {
    console.error('read_image failed', e);
  }
}

function init(): void {
  build();
  // Trigger the slide-in on the next frame.
  requestAnimationFrame(() => {
    card.style.transform = 'translateX(0)';
    card.style.opacity = '1';
  });
  scheduleDismiss();
}

init();
