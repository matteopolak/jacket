const entryName = "main.jai";
const sourcePathAuthority = Symbol("parsed source path");

export class SourcePath {
  #name;

  constructor(name, authority) {
    if (authority !== sourcePathAuthority) throw new TypeError("Use SourcePath.parse().");
    this.#name = name;
    Object.freeze(this);
  }

  static parse(value) {
    if (typeof value !== "string" || value.startsWith("/") || /[\\\0:]/u.test(value)) {
      throw new TypeError("Use a relative file name with forward slashes.");
    }
    const parts = [];
    for (const part of value.split("/")) {
      if (part === "" || part === ".") continue;
      if (part === "..") {
        if (parts.length === 0) throw new TypeError("File names cannot leave the workspace.");
        parts.pop();
      } else {
        parts.push(part);
      }
    }
    const name = parts.join("/");
    const bytes = new TextEncoder().encode(name);
    if (!name || bytes.length > 4096 || new TextDecoder("utf-8", { ignoreBOM: true }).decode(bytes) !== name) {
      throw new TypeError("Use a nonempty UTF-8 file name of at most 4,096 bytes.");
    }
    return new SourcePath(name, sourcePathAuthority);
  }

  get name() {
    return this.#name;
  }
}

export class Workspace {
  #files = new Map();
  #selected;
  #revision = 0;

  constructor(source) {
    this.add(entryName, source);
  }

  get selected() {
    return this.#files.get(this.#selected);
  }

  get names() {
    return [...this.#files.keys()].sort((a, b) => {
      if (a === entryName) return -1;
      if (b === entryName) return 1;
      return a.localeCompare(b);
    });
  }

  get documents() {
    return this.names.map(name => Object.freeze({ path: name, text: this.#files.get(name).text, version: this.#files.get(name).version }));
  }

  get tree() {
    const root = { kind: "directory", name: "", path: "", children: [] };
    for (const name of this.names) {
      const parts = name.split("/"); let parent = root; let prefix = "";
      for (const component of parts.slice(0, -1)) {
        prefix += (prefix ? "/" : "") + component;
        let child = parent.children.find(node => node.kind === "directory" && node.name === component);
        if (!child) { child = { kind: "directory", name: component, path: prefix, children: [] }; parent.children.push(child); }
        parent = child;
      }
      parent.children.push({ kind: "file", name: parts.at(-1), path: name });
    }
    const sort = node => { node.children.sort((a,b) => a.kind === b.kind ? a.name.localeCompare(b.name) : a.kind === "directory" ? -1 : 1); for (const child of node.children) if (child.kind === "directory") sort(child); return node; };
    return sort(root);
  }

  get canRemoveSelected() {
    return this.#selected !== entryName;
  }

  add(name, text = "") {
    const path = SourcePath.parse(name);
    if (typeof text !== "string") throw new TypeError("Source files must contain text.");
    if (this.#files.has(path.name)) throw new Error("That file already exists.");
    this.#files.set(path.name, Object.freeze({ path, text, version: ++this.#revision }));
    this.#selected = path.name;
  }

  select(name) {
    const path = SourcePath.parse(name);
    if (!this.#files.has(path.name)) throw new Error("That file does not exist.");
    this.#selected = path.name;
  }

  edit(text) {
    if (typeof text !== "string") throw new TypeError("Source files must contain text.");
    if (text === this.selected.text) return;
    this.#files.set(this.#selected, Object.freeze({ path: this.selected.path, text, version: ++this.#revision }));
  }

  removeSelected() {
    if (!this.canRemoveSelected) throw new Error("The entry file cannot be removed.");
    this.#files.delete(this.#selected);
    this.#selected = entryName;
  }

  snapshot() {
    const files = Object.fromEntries([...this.#files]
      .filter(([name]) => name !== entryName)
      .map(([name, file]) => [name, file.text]));
    return Object.freeze({ source: this.#files.get(entryName).text, files: Object.freeze(files) });
  }
}
