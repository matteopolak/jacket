import { readdirSync } from "node:fs";
import * as path from "node:path";
import Mocha from "mocha";

export function run(): Promise<void> {
  const mocha = new Mocha({ ui: "bdd", timeout: 120_000, color: true });
  for (const file of readdirSync(__dirname)) {
    if (file.endsWith(".e2e.cjs")) mocha.addFile(path.join(__dirname, file));
  }
  return new Promise((resolve, reject) => {
    mocha.run((failures) => (failures ? reject(new Error(`${failures} integration test(s) failed`)) : resolve()));
  });
}
