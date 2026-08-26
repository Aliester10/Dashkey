import { getContext, setContext } from "svelte";

export type ThemeMode = "dark" | "light";

const STORAGE_KEY = "dashkey-theme";

function initialTheme(): ThemeMode {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === "dark" || stored === "light") return stored;
    return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
  } catch {
    return "dark";
  }
}

export function createThemeCtx() {
  let theme = $state<ThemeMode>(initialTheme());

  function apply() {
    try {
      document.documentElement.dataset.theme = theme;
      localStorage.setItem(STORAGE_KEY, theme);
    } catch {
      /* penyimpanan tidak tersedia */
    }
  }

  $effect(() => {
    apply();
  });

  return {
    get theme() {
      return theme;
    },
    set(mode: ThemeMode) {
      theme = mode;
    },
    toggle() {
      theme = theme === "dark" ? "light" : "dark";
    },
  };
}

export type ThemeCtx = ReturnType<typeof createThemeCtx>;

const THEME_KEY = Symbol("theme");

export function setThemeCtx(ctx: ThemeCtx) {
  setContext(THEME_KEY, ctx);
}

export function getThemeCtx(): ThemeCtx {
  return getContext<ThemeCtx>(THEME_KEY);
}
