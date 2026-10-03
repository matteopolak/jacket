#!/usr/bin/env python3
"""Exercise the actual served playground in installed Playwright WebKit."""
import argparse
import json
import os
from pathlib import Path
from urllib.parse import urlsplit
from playwright.sync_api import sync_playwright

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('url')
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--admission-only', action='store_true')
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=True)
def check_admission(browser):
    page = browser.new_page(viewport={'width': 1280, 'height': 900})
    errors = []
    page.on('pageerror', lambda error: errors.append(str(error)))
    page.goto(args.url)
    page.wait_for_function("document.querySelector('#language-status').textContent.includes('connected')")
    content = page.locator('.cm-content')
    def source(text):
        content.click()
        page.keyboard.press('Meta+A')
        page.keyboard.insert_text(text)
    page.locator('#file-name').fill('lib/invalid:name.jai')
    page.get_by_role('button', name='Add file', exact=True).click()
    assert page.locator('#selected-name').inner_text() == 'main.jai'
    assert 'relative file name' in page.locator('#result').inner_text()
    path = "lib/bang!'()*.jai"
    page.locator('#file-name').fill(path)
    page.get_by_role('button', name='Add file', exact=True).click()
    assert page.locator('#selected-name').inner_text() == path
    source('broken ::')
    page.wait_for_function("expected => [...document.querySelectorAll('.problem-location')].some(node => node.textContent.startsWith(expected + ':'))", arg=path)
    assert page.locator('.cm-lint-marker-error').count() > 0
    page.locator('#remove-file').click()
    source('original :: () -> int { return 42; }\nmain :: () -> int { return original(); }')
    page.wait_for_function("document.querySelector('#problem-count').textContent === '0'")
    page.locator('.cm-line').nth(1).get_by_text('original', exact=True).hover()
    page.locator('.cm-tooltip-hover').wait_for()
    assert 'original' in page.locator('.cm-tooltip-hover').inner_text()
    # This actual edit exceeds the real core document limit but fits the runtime VFS.
    source('/*' + 'x' * (256 * 1024) + '*/\nmain :: () -> int { return 42; }')
    page.wait_for_function("document.querySelector('#language-status').textContent.includes('unavailable')")
    assert 'document byte budget exceeded' in page.locator('#result').inner_text()
    source('original :: () -> int { return 42; }\nmain :: () -> int { return original(); }')
    page.locator('.cm-line').nth(1).get_by_text('original', exact=True).hover()
    page.wait_for_timeout(600)
    assert page.locator('.cm-tooltip-hover').count() == 0, 'Rejected edit must not display old-source hover'
    page.locator('#run').click()
    page.wait_for_function("document.querySelector('#result').textContent.includes('Exit code: 42')")
    assert page.locator('#language-status').inner_text().endswith('unavailable')
    assert not errors, errors
    page.screenshot(path=str(args.output / 'admission.png'), full_page=True)
    return {'colonPathRejected': True, 'canonicalPunctuationUriDiagnostics': True, 'actualOversizedEditFailure': True, 'staleSourceHoverDisabled': True, 'runtimeAfterLanguageFailure': True, 'pageErrors': errors}

with sync_playwright() as playwright:
    options = {'headless': True}
    if os.environ.get('WEBKIT_EXECUTABLE'):
        options['executable_path'] = os.environ['WEBKIT_EXECUTABLE']
    browser = playwright.webkit.launch(**options)
    admission = check_admission(browser)
    (args.output / 'admission-proof.json').write_text(json.dumps(admission, indent=2) + '\n')
    if args.admission_only:
        browser.close()
        print('PASS: actual Wasm UI path identity, rejected document synchronization and stale-source prevention')
        raise SystemExit(0)
    page = browser.new_page(viewport={'width': 1280, 'height': 900})
    errors = []
    page.on('pageerror', lambda error: errors.append(str(error)))
    page.goto(args.url)
    page.wait_for_function("document.querySelector('#runtime-status').textContent === 'Compiler ready'")
    page.wait_for_function("document.querySelector('#language-status').textContent.includes('connected')")
    content = page.locator('.cm-content')
    def source(text):
        content.click()
        page.keyboard.press('Meta+A')
        page.keyboard.insert_text(text)
    def execute(code='42'):
        page.locator('#run').click()
        page.wait_for_function("code => document.querySelector('#result').textContent.includes('Exit code: ' + code)", arg=code)
    assert page.locator('.cm-lineNumbers').is_visible()
    assert page.locator('.cm-line span').count() > 0, 'Rendered syntax colors exist'
    execute()
    source('main :: () -> int { return 43; }')
    page.keyboard.press('Meta+z')
    assert '42' in content.inner_text()
    page.keyboard.press('Meta+Shift+z')
    assert '43' in content.inner_text()
    page.locator('#find').click()
    assert page.locator('.cm-search').is_visible()
    page.keyboard.press('Escape')
    source('main :: () -> int { return answer(); }\nanswer :: () -> int { return 42; }')
    content.click()
    page.keyboard.press('Meta+End')
    page.keyboard.insert_text('\n')
    page.keyboard.insert_text('(')
    assert content.inner_text().rstrip().endswith('()'), 'Actual bracket pairing'
    assert page.locator('.cm-matchingBracket').count() > 0, 'Actual bracket matching'
    source('main :: () {')
    page.keyboard.press('Enter')
    assert '\n    ' in content.inner_text(), 'Actual automatic indentation'
    page.locator('#file-name').fill('lib/helpers.jai')
    page.get_by_role('button', name='Add file', exact=True).click()
    source('answer :: () -> int { return 42; }')
    assert page.locator('#files details summary').inner_text().endswith('lib')
    page.locator('#files button[title="main.jai"]').click()
    source('#load "lib/helpers.jai";\nmain :: () -> int { return answer(); }')
    execute()
    source('main :: () { while true {} }')
    page.locator('#run-options summary').click()
    page.locator('#fuel').fill('4294967295')
    page.locator('#run').click()
    page.wait_for_function("!document.querySelector('#cancel').disabled")
    page.locator('#cancel').click()
    assert page.locator('#result').inner_text() == 'Cancelled.'
    source('main :: () -> int { return 42; }')
    page.locator('#fuel').fill('1000000')
    execute()
    source('main :: () { return ; @ }')
    page.wait_for_function("Number(document.querySelector('#problem-count').textContent) > 0")
    assert page.locator('.cm-lint-marker-error').count() > 0
    page.locator('#problems-tab').click()
    assert page.locator('.problem-row').count() > 0
    source('answer :: () -> int { return 42; }\nmain :: () -> int { return answer(); }')
    page.wait_for_function("document.querySelector('#problem-count').textContent === '0'")
    answer = page.locator('.cm-line').nth(1).get_by_text('answer', exact=True)
    answer.hover()
    page.locator('.cm-tooltip-hover').wait_for()
    assert 'answer' in page.locator('.cm-tooltip-hover').inner_text()
    source('answer :: () -> int { return 42; }\nmain :: () -> int { return ans; }')
    for _ in range(3):
        page.keyboard.press('ArrowLeft')
    page.keyboard.press('Control+Space')
    page.locator('.cm-tooltip-autocomplete').wait_for()
    assert 'answer' in page.locator('.cm-tooltip-autocomplete').inner_text()
    page.keyboard.press('Escape')
    source('answer :: () -> int { return 42; }\nmain :: () -> int { return answer(); }')
    page.locator('#output-tab').click()
    page.locator('#run-options summary').click()
    page.screenshot(path=str(args.output / 'desktop.png'), full_page=True)
    page.set_viewport_size({'width': 390, 'height': 844})
    page.screenshot(path=str(args.output / 'mobile.png'), full_page=True)
    assert page.evaluate('document.documentElement.scrollWidth <= innerWidth'), 'No horizontal mobile overflow'
    assert not errors, errors
    embedded = browser.new_page()
    embedded.goto(args.url)
    # Keep the actual served origin for the parent, with a real child iframe.
    frame_url = args.url + ('&' if '?' in args.url else '?') + 'embed=1'
    embedded.set_content('<style>body{margin:0}iframe{border:0;width:100vw;height:100vh}</style><script>window.messages=[];addEventListener("message",e=>{if(e.origin===location.origin)messages.push(e.data)})</script><iframe title="actual Jai embed" src=' + json.dumps(frame_url) + '></iframe>')
    embedded.wait_for_function("messages.some(m=>m.type==='jai-playground'&&m.state==='ready')")
    message = embedded.evaluate("messages.find(m=>m.state==='ready')")
    assert len(message['revision']) == 40
    assert embedded.frame_locator('iframe').locator('#runtime-status').inner_text() == 'Compiler ready'
    embedded.screenshot(path=str(args.output / 'embed.png'), full_page=True)
    failing = browser.new_page()
    failing.route('**/jai_wasm.wasm', lambda route: route.abort())
    failing.goto(args.url)
    failing.set_content('<style>body{margin:0}iframe{border:0;width:100vw;height:100vh}</style><script>window.messages=[];addEventListener("message",e=>{if(e.origin===location.origin)messages.push(e.data)})</script><iframe src=' + json.dumps(frame_url) + '></iframe>')
    failing.wait_for_function("messages.some(m=>m.type==='jai-playground'&&m.state==='error')")
    failure = failing.evaluate("messages.find(m=>m.state==='error')")
    assert failure['revision'] == message['revision'] and failure.get('message')
    assert not failing.evaluate("messages.some(m=>m.state==='ready')")
    browser.close()
    print('PASS: real rendered Wasm Run/Cancel/VFS, editor behavior, shared LSP, mobile, iframe ready/error')
    (args.output / 'browser-proof.json').write_text(json.dumps({'realWebKit': True, 'actualWasmRun': True, 'multiFileVfs': True, 'cancelRestart': True, 'editorHistory': True, 'find': True, 'bracketPairAndMatch': True, 'autoIndent': True, 'realDiagnostics': True, 'actualHoverCompletion': True, 'embedReadyAndFailure': True, 'responsive': True, 'pageErrors': errors}, indent=2) + '\n')
