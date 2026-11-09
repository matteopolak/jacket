// What the site scripts hang on `window` for one another and for the page's inline scripts.
interface Window {
  jacketRestore: () => void;
  /** Opens the folds that fit and restores the reading position; `load` on a first load. */
  jacketPage: (load?: boolean) => void;
  /** Finishes the sidebar; `state` is what site.js saved from the page being replaced. */
  jacketSide: (state?: SideState | null) => void;
}

/** The sidebar as a reader left it: its list (`k`), scroll (`t`) and open groups (`o`). */
interface SideState {
  k: string | null;
  t: number;
  o?: string[];
}
