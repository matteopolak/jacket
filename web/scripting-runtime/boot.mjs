// No editor package imports here: load failures can still reach the embed host.
const embedded = new URLSearchParams(location.search).get("embed") === "1";
let revision;
try {
  const response = await fetch(new URL("./release.json", import.meta.url));
  if (response.ok) {
    const release = await response.json();
    if (/^[a-f\d]{40}$/u.test(release.commit ?? "")) revision = release.commit;
  }
} catch { /* Standalone development can omit a release pointer. */ }
const pathRevision = new URL(import.meta.url).pathname.split("/").filter(Boolean).at(-2);
const errorRevision = revision ?? (/^[a-f\d]{40}$/u.test(pathRevision ?? "") ? pathRevision : "");
function error(message) {
  const status = document.querySelector("#runtime-status"); if (status) status.textContent = "Compiler unavailable";
  const result = document.querySelector("#result"); if (result) result.textContent = message;
  if (embedded) parent.postMessage({ type: "jai-playground", state: "error", revision: errorRevision, message }, location.origin);
}
if (embedded && !revision) error("This embedded bundle has no verified release revision.");
else {
  window.JAI_PLAYGROUND_BOOT = Object.freeze({ revision });
  try { await import("./editor.mjs"); }
  catch (failure) { error(failure instanceof Error ? failure.message : String(failure)); }
}
