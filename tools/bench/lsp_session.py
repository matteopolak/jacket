"""One language-server session: the connection, the open document, and the scripted measurements."""
from __future__ import annotations

import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable

from bench.lsp_client import (Client, Timeout, Unsupported, file_uri)
from bench.lsp_positions import delete_lines_edit, insert_edit, replace_edit


class ProbeFailed(RuntimeError):
    """One measurement could not be taken; the session goes on."""


def answer(response: dict):
    """The `result` of a response, or the right exception for its error."""
    error = response.get('error')
    if error:
        if error.get('code') == -32601:
            raise Unsupported(f"method not found: {error.get('message')}")
        raise ProbeFailed(f"{error.get('code')}: {error.get('message')}")
    return response.get('result')


@dataclass
class Document:
    """The one open document: its path, text and the places the probes use (see lsp_positions)."""
    path: Path
    text: str
    picks: dict
    also: list[Path] = field(default_factory=list)

    @property
    def lines(self) -> list[str]:
        return self.text.split('\n')


class Session:
    def __init__(self, client: Client, workspace: Path, document: Document, pull_diagnostics: bool,
                 timeout: float, push_wait: float):
        self.client = client
        self.workspace = workspace
        self.document = document
        self.pull = pull_diagnostics
        self.timeout = timeout
        self.push_wait = push_wait
        self.uri = file_uri(document.path)
        self.version = 1
        self.server_capabilities: dict = {}
        self.last_token_result: str | None = None
        self.empty_answers = 0  # answers that were null or empty: a fast one may be an answer to nothing

    # ---------------------------------------------------------------------------- requests

    def request(self, method: str, params: dict | None = None):
        """(seconds until the answer, the result)."""
        seconds, response = self.client.request(method, params, self.timeout)
        result = answer(response)
        if not result:
            self.empty_answers += 1
        return seconds, result

    def at(self, position: dict, **extra) -> dict:
        return {'textDocument': {'uri': self.uri},
                'position': {'line': position['line'], 'character': position['character']}, **extra}

    def whole_document(self, **extra) -> dict:
        return {'textDocument': {'uri': self.uri}, **extra}

    # ------------------------------------------------------------------------------ edits

    def change(self, edits: list[dict]) -> float:
        """Send a didChange; the time it was sent."""
        self.version += 1
        return self.client.notify('textDocument/didChange', {
            'textDocument': {'uri': self.uri, 'version': self.version}, 'contentChanges': edits})

    def diagnostics_since(self, start: float, seen: int) -> float:
        """Seconds from `start` until the server's diagnostics for the document arrive: the next
        publishDiagnostics after notification number `seen`, or the answer to a pull request."""
        if self.pull:
            self.request('textDocument/diagnostic', self.whole_document())
            return time.perf_counter() - start
        try:
            return self.client.wait_notification('textDocument/publishDiagnostics', self.uri, start,
                                                 self.push_wait, seen)
        except Timeout:
            raise ProbeFailed(f'no publishDiagnostics within {self.push_wait:g} s') from None

    def timed_edit(self, edits: list[dict]) -> float:
        """Seconds from sending `edits` until the new diagnostics arrive."""
        seen = len(self.client.notifications)
        return self.diagnostics_since(self.change(edits), seen)

    def apply(self, edits: list[dict]) -> None:
        """Send `edits` and wait for their diagnostics, so nothing from them is pending in the next
        timed window."""
        self.timed_edit(edits)

    def open(self, path: Path, text: str) -> float:
        return self.client.notify('textDocument/didOpen', {'textDocument': {
            'uri': file_uri(path), 'languageId': 'jai', 'version': 1, 'text': text}})

    def supports(self, capability: str | None) -> bool:
        """Whether the server advertised `capability`, a dotted path into its capabilities."""
        if capability is None:
            return True
        node = self.server_capabilities
        for part in capability.split('.'):
            node = node.get(part) if isinstance(node, dict) else None
        return bool(node)


# ------------------------------------------------------------------------------------------- probes

@dataclass(frozen=True)
class Probe:
    """A measurement: `run` returns latency samples in seconds. Probes run in order; each leaves the document as
    it found it."""
    name: str
    description: str
    run: Callable[[Session], list[float]]
    capability: str | None = None


def comment_line(session: Session) -> tuple[int, str]:
    picks = session.document.picks
    return picks['insert_line'], picks['indent']


def keystroke(session: Session) -> list[float]:
    """One typed character at the end of a comment line inside a procedure body."""
    line, indent = comment_line(session)
    column = len(indent) + 3
    session.apply([insert_edit(line, 0, f'{indent}// \n')])
    samples = []
    for _ in range(3):
        samples.append(session.timed_edit([insert_edit(line, column, 'x')]))
        session.apply([replace_edit(line, column, line, column + 1, '')])
    session.apply([delete_lines_edit(line)])
    return samples


def pasted_procedure(session: Session) -> str:
    """The text of the picked procedure with its name changed, ready to paste at the end of the file."""
    picks = session.document.picks
    lines = session.document.lines
    first = picks['declaration']['line']
    name = picks['declaration']['name']
    body = []
    for number in range(first, len(lines)):
        body.append(lines[number].replace(name, f'{name}_pasted', 1) if number == first else lines[number])
        if number > first and lines[number].rstrip() in ('}', '};'):
            break
    return '\n'.join(body) + '\n'


def append_edits(session: Session, text: str) -> tuple[list[dict], list[dict]]:
    """(an edit that appends `text` to the document, the edit that removes it again)."""
    lines = session.document.lines
    end = (len(lines) - 1, len(lines[-1]))
    added = text.split('\n')
    last = (end[0] + len(added) - 1, len(added[-1]) if len(added) > 1 else end[1] + len(text))
    return [replace_edit(*end, *end, text)], [replace_edit(*end, *last, '')]


def edit_and_undo(session: Session, text: str, repeat: int) -> list[float]:
    do, undo = append_edits(session, text)
    samples = []
    for _ in range(repeat):
        samples.append(session.timed_edit(do))
        session.apply(undo)
    return samples


def paste(session: Session) -> list[float]:
    return edit_and_undo(session, '\n' + pasted_procedure(session), 2)


def large_edit(session: Session) -> list[float]:
    """A fifth of the document pasted at its end."""
    lines = session.document.lines
    return edit_and_undo(session, '\n' + '\n'.join(lines[:max(10, len(lines) // 5)]) + '\n', 1)


def hover_first(session: Session) -> list[float]:
    picks = session.document.picks
    seconds, result = session.request('textDocument/hover', session.at(picks['hover']))
    for alternative in picks.get('hover_alternatives', []):
        if result is not None:
            break
        _, result = session.request('textDocument/hover', session.at(alternative))
        if result is not None:
            picks['hover'] = alternative
    return [seconds]


def hover_warm(session: Session) -> list[float]:
    params = session.at(session.document.picks['hover'])
    return [session.request('textDocument/hover', params)[0] for _ in range(5)]


def completion(position: dict, trigger: str | None = None):
    def run(session: Session) -> list[float]:
        context = {'triggerKind': 2, 'triggerCharacter': trigger} if trigger else {'triggerKind': 1}
        return [session.request('textDocument/completion', session.at(position, context=context))[0]]
    return run


def completion_member(session: Session) -> list[float]:
    member = session.document.picks['member']
    if not member:
        raise ProbeFailed('no member access found to complete after')
    return completion(member, '.')(session)


def completion_global(session: Session) -> list[float]:
    """Typing the start of a name inside a procedure body: completion over everything in scope."""
    line, indent = comment_line(session)
    prefix = session.document.picks['hover']['name'][:3]
    text = f'{indent}{prefix}\n'
    session.apply([insert_edit(line, 0, text)])
    try:
        return completion({'line': line, 'character': len(indent) + len(prefix)})(session)
    finally:
        session.apply([delete_lines_edit(line)])


def completion_import_path(session: Session) -> list[float]:
    """Completing a module name inside `#import "`."""
    session.apply([insert_edit(0, 0, '#import "\n')])
    try:
        return completion({'line': 0, 'character': len('#import "')}, '"')(session)
    finally:
        session.apply([delete_lines_edit(0)])


def definition(session: Session) -> list[float]:
    return [session.request('textDocument/definition', session.at(session.document.picks['hover']))[0]]


def references(session: Session) -> list[float]:
    declaration = session.document.picks['declaration']
    return [session.request('textDocument/references', session.at(declaration, context={'includeDeclaration': True}))[0]]


def rename(session: Session) -> list[float]:
    declaration = session.document.picks['declaration']
    return [session.request('textDocument/rename', session.at(declaration, newName=declaration['name'] + '_renamed'))[0]]


def semantic_tokens_full(session: Session) -> list[float]:
    seconds, result = session.request('textDocument/semanticTokens/full', session.whole_document())
    session.last_token_result = (result or {}).get('resultId')
    return [seconds]


def semantic_tokens_delta(session: Session) -> list[float]:
    """Tokens after a one-line edit, as a delta from the full answer before it."""
    previous = getattr(session, 'last_token_result', None)
    if previous is None:
        raise ProbeFailed('the full answer had no resultId to delta from')
    line, indent = comment_line(session)
    session.apply([insert_edit(line, 0, f'{indent}// delta\n')])
    try:
        return [session.request('textDocument/semanticTokens/full/delta',
                                session.whole_document(previousResultId=previous))[0]]
    finally:
        session.apply([delete_lines_edit(line)])


def document_symbols(session: Session) -> list[float]:
    return [session.request('textDocument/documentSymbol', session.whole_document())[0]]


def workspace_symbols(session: Session) -> list[float]:
    query = session.document.picks['declaration']['name'][:6]
    return [session.request('workspace/symbol', {'query': query})[0]]


def inlay_hints(session: Session) -> list[float]:
    first = session.document.picks['declaration']['line']
    last = min(len(session.document.lines) - 1, first + 80)
    rng = {'start': {'line': first, 'character': 0}, 'end': {'line': last, 'character': 0}}
    return [session.request('textDocument/inlayHint', session.whole_document(range=rng))[0]]


def signature_help(session: Session) -> list[float]:
    call = session.document.picks['call']
    if not call:
        raise ProbeFailed('no call found to ask for signature help in')
    return [session.request('textDocument/signatureHelp',
                            session.at(call, context={'triggerKind': 2, 'triggerCharacter': '(', 'isRetrigger': False}))[0]]


def formatting(session: Session) -> list[float]:
    return [session.request('textDocument/formatting',
                            session.whole_document(options={'tabSize': 4, 'insertSpaces': True}))[0]]


def range_formatting(session: Session) -> list[float]:
    first = session.document.picks['declaration']['line']
    rng = {'start': {'line': first, 'character': 0}, 'end': {'line': first + 40, 'character': 0}}
    return [session.request('textDocument/rangeFormatting',
                            session.whole_document(range=rng, options={'tabSize': 4, 'insertSpaces': True}))[0]]


PROBES = [
    Probe('did_change_keystroke', 'one typed character, until diagnostics', keystroke),
    Probe('did_change_paste', 'paste a whole procedure, until diagnostics', paste),
    Probe('did_change_large', 'paste a fifth of the file, until diagnostics', large_edit),
    Probe('completion_member', 'completion after `.`', completion_member, 'completionProvider'),
    Probe('completion_global', 'completion of a name typed in a body', completion_global, 'completionProvider'),
    Probe('completion_import', 'completion inside `#import "`', completion_import_path, 'completionProvider'),
    Probe('hover_first', 'first hover', hover_first, 'hoverProvider'),
    Probe('hover_warm', 'hover again', hover_warm, 'hoverProvider'),
    Probe('definition', 'go to definition', definition, 'definitionProvider'),
    Probe('references', 'references across the workspace', references, 'referencesProvider'),
    Probe('rename', 'rename across the workspace', rename, 'renameProvider'),
    Probe('semantic_tokens_full', 'semantic tokens, full', semantic_tokens_full, 'semanticTokensProvider.full'),
    Probe('semantic_tokens_delta', 'semantic tokens, delta after an edit', semantic_tokens_delta,
          'semanticTokensProvider.full.delta'),
    Probe('document_symbols', 'document symbols', document_symbols, 'documentSymbolProvider'),
    Probe('workspace_symbols', 'workspace symbols', workspace_symbols, 'workspaceSymbolProvider'),
    Probe('inlay_hints', 'inlay hints for one procedure', inlay_hints, 'inlayHintProvider'),
    Probe('signature_help', 'signature help in a call', signature_help, 'signatureHelpProvider'),
    Probe('formatting', 'format the document', formatting, 'documentFormattingProvider'),
    Probe('range_formatting', 'format a range', range_formatting, 'documentRangeFormattingProvider'),
]

# Probes the session runs itself, around initialize, the first open and the end of the script.
SESSION_PROBES = {
    'initialize': 'launch until the initialize response',
    'index_ready': 'initialized until workspace symbols find the document\'s own procedure',
    'did_open': 'didOpen until the first diagnostics',
    'file_watcher': 'a file changed on disk, until its new symbol is searchable',
    'long_session': 'diagnostics latency over a long run of edits (resident set growth in the info)',
}
