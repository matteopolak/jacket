import { Workspace } from "./workspace.mjs";

const source = document.querySelector("#source");
const files = document.querySelector("#files");
const selectedName = document.querySelector("#selected-name");
const newFile = document.querySelector("#new-file");
const fileName = document.querySelector("#file-name");
const remove = document.querySelector("#remove-file");
const args = document.querySelector("#arguments");
const fuel = document.querySelector("#fuel");
const result = document.querySelector("#result");
const run = document.querySelector("#run");
const cancel = document.querySelector("#cancel");
const workspace = new Workspace(source.value);
let worker;

function renderFiles() {
  files.replaceChildren(...workspace.names.map(name => {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = name;
    button.setAttribute("aria-pressed", String(name === workspace.selected.path.name));
    button.onclick = () => {
      workspace.select(name);
      source.value = workspace.selected.text;
      renderFiles();
      source.focus();
    };
    return button;
  }));
  selectedName.textContent = workspace.selected.path.name;
  remove.disabled = !workspace.canRemoveSelected;
}

function idle() {
  run.disabled = false;
  cancel.disabled = true;
}

function terminate() {
  worker?.terminate();
  worker = undefined;
}

source.oninput = () => workspace.edit(source.value);
source.onkeydown = event => {
  if (event.key === "Tab") {
    event.preventDefault();
    source.setRangeText("    ", source.selectionStart, source.selectionEnd, "end");
    workspace.edit(source.value);
  }
};

newFile.onsubmit = event => {
  event.preventDefault();
  try {
    workspace.add(fileName.value);
    fileName.value = "";
    source.value = workspace.selected.text;
    renderFiles();
    source.focus();
  } catch (error) {
    result.textContent = error.message;
    fileName.focus();
  }
};

remove.onclick = () => {
  workspace.removeSelected();
  source.value = workspace.selected.text;
  renderFiles();
  source.focus();
};

run.onclick = () => {
  try {
    const argumentsValue = JSON.parse(args.value);
    if (!Array.isArray(argumentsValue) || argumentsValue.some(value => typeof value !== "string")) {
      throw new Error("Arguments must be a JSON array of strings.");
    }
    const stepLimit = Number(fuel.value);
    if (!Number.isInteger(stepLimit) || stepLimit < 0 || stepLimit > 0xffffffff) {
      throw new Error("The execution step limit must be an integer from 0 to 4,294,967,295.");
    }
    worker ??= new Worker("./worker.mjs", { type: "module" });
    worker.onmessage = ({ data }) => {
      const output = data.error ?? `Exit code: ${data.result.exitCode}\nInterpreter steps: ${data.result.steps}`;
      result.textContent = data.consoleText ? `${data.consoleText}\n${output}` : output;
      idle();
    };
    worker.onerror = event => {
      result.textContent = event.message || "Browser worker failed.";
      terminate();
      idle();
    };
    const snapshot = workspace.snapshot();
    run.disabled = true;
    cancel.disabled = false;
    result.textContent = "Checking and running…";
    worker.postMessage({ source: snapshot.source, options: {
      arguments: argumentsValue,
      fuel: stepLimit,
      files: snapshot.files,
    } });
  } catch (error) {
    result.textContent = error.message;
    idle();
  }
};

cancel.onclick = () => {
  terminate();
  result.textContent = "Cancelled.";
  idle();
};

document.addEventListener("keydown", event => {
  if ((event.ctrlKey || event.metaKey) && event.key === "Enter" && !run.disabled) {
    event.preventDefault();
    run.click();
  }
});

renderFiles();
