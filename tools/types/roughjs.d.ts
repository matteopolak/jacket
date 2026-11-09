// The part of roughjs that dev/readme_banner_rough.ts uses. The package is installed on demand
// in a temporary folder by dev/readme_banner.py, not in tools/node_modules.
declare module 'roughjs/bundled/rough.esm.js' {
  interface RoughPath {
    d: string;
    stroke: string;
    strokeWidth: number;
    fill?: string;
  }
  interface Drawable {
    readonly shape: string;
  }
  interface Generator {
    path(d: string, options?: Record<string, unknown>): Drawable;
    toPaths(drawable: Drawable): RoughPath[];
  }
  const rough: { generator(): Generator };
  export default rough;
}
