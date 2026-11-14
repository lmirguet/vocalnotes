#!/usr/bin/env python3
"""Opt-in AT-SPI recipe against a fresh, isolated real Wayland application.

Requires python3-pyatspi in a GNOME Wayland session and a current binary built
with --features tauri/custom-protocol. Never selects a dictation mode, invokes a
provider, or touches keyring controls. AT-SPI actions do not certify physical
keyboard shortcuts; the keyboard section uses the existing user ydotoold.
Pointer behaviour and human usability require separate observations.
The keyboard helper requires the reference French AZERTY layout.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

os.environ["GDK_BACKEND"] = "wayland"
import pyatspi
from gi.repository import GLib

REPO = Path(__file__).resolve().parents[1]


def walk(node):
    if node is None:
        return
    yield node
    try:
        for child in node:
            if child:
                yield from walk(child)
    except GLib.GError:
        # WebKit replaces its accessibility subtree when a webview is destroyed.
        return


def wait(check, timeout=12):
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        value = check()
        if value:
            return value
        time.sleep(.02)
    raise AssertionError("Native UI condition timed out")


class Recipe:
    def __init__(self, process, root, socket):
        self.socket = socket
        self.process = process
        self.root = root
        self.app = wait(lambda: next((a for a in pyatspi.Registry.getDesktop(0)
                                     if a.get_process_id() == process.pid), None))
        self.results = []

    def find(self, name, role=None, scope=None):
        matches = [n for n in walk(scope or self.app)
                   if n.name == name and (role is None or n.getRoleName() == role)]
        assert len(matches) == 1, (name, role, len(matches))
        return matches[0]

    def frames(self):
        return [n for n in self.app if n.getRoleName() in ('frame', 'dialog', 'file chooser', 'alert')]

    def dialog(self, name):
        return next((n for n in self.frames() if n.name == name), None)

    def activate(self, name, role=None, scope=None):
        node = self.find(name, role, scope)
        actions = node.queryAction()
        names = [actions.getName(i) for i in range(actions.nActions)]
        index = next((i for i, value in enumerate(names) if value in ('click', 'press', 'activate')), 0)
        assert actions.doAction(index), (name, 'action refused')

    def menu(self, name):
        self.activate(name, 'menu item')

    def key(self, *codes):
        assert any(n.getState().contains(pyatspi.STATE_ACTIVE) for n in self.frames()), "Recipe window lost desktop focus"
        env = {**os.environ, 'YDOTOOL_SOCKET': self.socket}
        subprocess.run(['ydotool', 'key', *[str(c) + ':1' for c in codes],
                        *[str(c) + ':0' for c in reversed(codes)]], env=env, check=True,
                       stdout=subprocess.DEVNULL)

    def type(self, value):
        # This recipe explicitly targets the reference French AZERTY session.
        # GDK's keymap in a helper without a focused surface can remain US.
        rows = [('azertyuiop', 16), ('qsdfghjklm', 30), ('wxcvbn', 44)]
        mapping = {char: start + i for row, start in rows for i, char in enumerate(row)}
        mapping.update({'-': 7, '_': 9, ' ': 57, '\n': 28})
        for char in value:
            if char.isascii() and char.isdigit():
                self.key(42, 11 if char == '0' else int(char) + 1)
            elif char in '/.':
                self.key(42, 52 if char == '/' else 51)
            elif char.isascii() and char.isupper():
                self.key(42, mapping[char.lower()])
            else:
                self.key(mapping[char])

    def text(self, name, value, scope=None):
        node = self.find(name, 'entry', scope)
        assert node.queryComponent().grabFocus()
        wait(lambda: node.getState().contains(pyatspi.STATE_FOCUSED))
        self.key(29, 16)  # Ctrl+A (A on the French layout)
        self.type(value)
        time.sleep(.15)
        assert self.value(name, scope).removesuffix('\n') == value, (name, self.value(name, scope), value)

    def value(self, name, scope=None):
        node = self.find(name, 'entry', scope)
        return node.queryText().getText(0, -1)

    def dump(self):
        def visit(node, depth=0):
            print(' ' * depth + repr(node.name) + ' ' + node.getRoleName() + ((' VALUE=' + repr(node.queryText().getText(0, -1))) if node.getRoleName() in ('entry', 'text', 'status bar') else ''), flush=True)
            if depth < 16:
                for child in node:
                    if child:
                        visit(child, depth + 1)
        visit(self.app)

    def passed(self, name):
        self.results.append(name)
        print('PASS ' + name, flush=True)

    def preferences(self):
        wait(lambda: any(n.name == 'Note visuelle' for n in walk(self.app)))
        # Establish focus in the owned application before activating a native menu.
        self.find('Note visuelle', 'entry').queryComponent().grabFocus()
        time.sleep(.2)
        menus = [n for n in walk(self.app) if n.getRoleName() == 'menu bar']
        assert len(menus) == 1
        assert [n.name for n in menus[0]] == ['Fichier', 'Edition', 'Aide']
        assert [n.name for menu in menus[0] for n in menu] == [
            'Nouvelle note', 'Ouvrir', 'Enregistrer', 'Enregistrer sous',
            'Exporter en markdown', 'Quitter', 'Préférence', 'Annuler',
            'Rétablir', 'A propos']
        self.passed('one native menu bar, three menus and ten ordered commands')
        for i in range(10):
            print('Preferences path ' + str(i + 1) + '/10', flush=True)
            self.menu('Préférence')
            dialog = wait(lambda: self.dialog('Préférences'))
            wait(lambda: any(n.name == 'Modèle Transcription' for n in walk(self.dialog('Préférences'))))
            wait(lambda: self.value('Modèle Transcription', dialog))
            assert dialog.getState().contains(pyatspi.STATE_MODAL)
            assert len(self.frames()) == 2
            before = self.value('Modèle Transcription', dialog)
            self.text('Modèle Transcription', 'synthetic-model-' + str(i), dialog)
            if i % 4 == 0:
                self.activate('OK', 'button', dialog)
            elif i % 4 == 1:
                self.activate('Annuler', 'button', dialog)
            elif i % 4 == 2:
                self.key(1)
            else:
                self.activate('Close', 'button', dialog)
            wait(lambda: len(self.frames()) == 1)
            time.sleep(.2)
            self.menu('Préférence')
            dialog = wait(lambda: self.dialog('Préférences'))
            wait(lambda: any(n.name == 'Modèle Transcription' for n in walk(self.dialog('Préférences'))))
            wait(lambda: self.value('Modèle Transcription', dialog))
            assert self.value('Modèle Transcription', dialog) == (
                'synthetic-model-' + str(i) if i % 4 == 0 else before)
            self.activate('Annuler', 'button', dialog)
            wait(lambda: len(self.frames()) == 1)
            time.sleep(.2)
        self.passed('ten native preferences paths: OK, cancel, Escape, system close, persisted/dropped drafts, modal state')

    def choose(self, command, path):
        self.menu(command)
        dialog = wait(lambda: next((n for n in self.frames() if n.getRoleName() == 'file chooser'), None))
        assert dialog.getState().contains(pyatspi.STATE_MODAL)
        self.key(29, 38)  # Ctrl+L
        entry = wait(lambda: next((n for n in walk(dialog) if n.getRoleName() in ('entry', 'text')
                                  and n.getState().contains(pyatspi.STATE_FOCUSED)), None))
        assert entry.queryEditableText().setTextContents(str(path))
        time.sleep(.2)
        self.activate('Ouvrir' if command == 'Ouvrir' else 'Enregistrer', 'button', dialog)
        wait(lambda: len(self.frames()) == 1)
        wait(lambda: not any(n.name == 'Opération documentaire' for n in walk(self.app)))
        time.sleep(.3)

    def files(self):
        wait(lambda: any(n.name == 'Note visuelle' for n in walk(self.app)))
        self.find('Note visuelle', 'entry').queryComponent().grabFocus()
        time.sleep(.2)
        notes = self.root / 'notes'
        notes.mkdir()
        manifest = []
        for i in range(20):
            body = ['', '# Été 👋\r\n\r\nopaque  \n', '<!-- vocalnotes:intention: texte littéral\r\n',
                    'e\u0301 et espaces  \r\n', '```text\nopaque\n```\n'][i % 5]
            profile = dict(purpose='ancien' if i % 2 else '', form='courriel', tone='',
                           context='Contexte --> & <été>' if i % 3 else '')
            metadata = json.dumps(profile, ensure_ascii=False, separators=(',', ':'))
            metadata = metadata.replace('&', r'\u0026').replace('<', r'\u003c').replace('>', r'\u003e')
            encoded = ('<!-- vocalnotes:intention:v1\n' + metadata + '\n-->\n\n' + body).encode()
            original = notes / ('input-' + str(i) + '.VON')
            saved = notes / ('saved-' + str(i) + '.VON')
            exported = notes / ('export-' + str(i) + '.MD')
            original.write_bytes(encoded)
            self.choose('Ouvrir', original)
            self.choose('Enregistrer sous', saved)
            wait(saved.exists)
            assert saved.read_bytes() == encoded
            self.choose('Ouvrir', saved)
            self.choose('Exporter en markdown', exported)
            wait(exported.exists)
            assert exported.read_bytes() == body.encode()
            assert original.read_bytes() == encoded
            assert self.value('Contexte de la note') == profile['context']
            manifest.append(dict(index=i, source=body, intention=profile,
                                 saved_sha256=hashlib.sha256(encoded).hexdigest()))
            print('Native cycle ' + str(i + 1) + '/20', flush=True)
        (notes / 'manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2))
        self.passed('twenty native VON open/save-as/reopen/export cycles: exact bytes and four intention fields')
        legacy = notes / 'legacy.MD'
        legacy.write_bytes(b'Legacy content\r\n')
        self.choose('Ouvrir', legacy)
        self.text('Note visuelle', 'legacy changed')
        assert legacy.read_bytes() == b'Legacy content\r\n'
        self.menu('Enregistrer')
        wait(lambda: legacy.read_bytes() == b'legacy changed')
        self.text('Contexte de la note', 'contexte ajoute')
        migrated = notes / 'migrated.von'
        self.choose('Enregistrer', migrated)
        wait(migrated.exists)
        assert legacy.read_bytes() == b'legacy changed'
        assert b'contexte ajoute' in migrated.read_bytes()
        self.passed('native MD explicit save without autosave, then VON migration preserving original MD')

    def keyboard(self):
        wait(lambda: any(n.name == 'Note visuelle' for n in walk(self.app)))
        for name in ['Note visuelle', 'Contexte de la note']:
            self.find(name, 'entry').queryComponent().grabFocus()
            time.sleep(.1)
            self.key(68)  # F10
            self.key(108)  # Down
            wait(lambda: any(n.getRoleName() == 'menu item' and n.getState().contains(pyatspi.STATE_SHOWING)
                             for n in walk(self.app)))
            self.key(1)
            self.key(1)
            self.find(name, 'entry').queryComponent().grabFocus()
            self.key(56, 33)  # Alt+F
            wait(lambda: self.find('Ouvrir', 'menu item').getState().contains(pyatspi.STATE_SHOWING))
            self.key(1)
        self.passed('F10/arrows and Alt+F from content and context')
        self.text('Contexte de la note', 'contexte test')
        time.sleep(.3)
        before = self.value('Contexte de la note')
        self.key(29, 17)  # Ctrl+Z on AZERTY
        wait(lambda: self.value('Contexte de la note') != before)
        self.key(29, 21)  # Ctrl+Y
        wait(lambda: self.value('Contexte de la note') == before)
        self.passed('native Ctrl+Z/Ctrl+Y in context')
        for codes in [(29, 31), (29, 42, 31), (29, 24)]:  # save/save-as/open
            self.find('Note visuelle', 'entry').queryComponent().grabFocus()
            self.key(*codes)
            dialog = wait(lambda: next((n for n in self.frames() if n.getRoleName() == 'file chooser'), None))
            self.activate('Annuler', 'button', dialog)
            wait(lambda: len(self.frames()) == 1)
        self.passed('Ctrl+S, Ctrl+Shift+S and Ctrl+O dispatch one cancellable chooser')
        for trigger in ['new', 'quit', 'close']:
            self.find('Contexte de la note', 'entry').queryComponent().grabFocus()
            if trigger == 'new':
                self.key(29, 49)  # Ctrl+N
            elif trigger == 'quit':
                self.key(56, 62)  # Alt+F4
            else:
                self.activate('Close', 'button', self.frames()[0])
            dialog = wait(lambda: self.dialog('Vocalnotes'))
            for label in ['Enregistrer', 'Ne pas enregistrer', 'Annuler']:
                self.find(label, 'button', dialog)
            self.activate('Annuler', 'button', dialog)
            wait(lambda: len(self.frames()) == 1)
            assert self.value('Contexte de la note') == before
        self.passed('Ctrl+N, Alt+F4 and system cross: native three-way confirmation and cancellation without loss')
        self.menu('A propos')
        dialog = wait(lambda: self.dialog('A propos'))
        assert any('0.1.0' in n.name for n in walk(dialog))
        self.key(1)
        wait(lambda: len(self.frames()) == 1)
        self.passed('native About version and Escape')

    def latency(self):
        wait(lambda: any(n.name == 'Note visuelle' for n in walk(self.app)))
        self.find('Note visuelle', 'entry').queryComponent().grabFocus()
        time.sleep(.2)
        elapsed = []
        for i in range(30):
            started = time.perf_counter()
            self.menu('Préférence')
            dialog = wait(lambda: self.dialog('Préférences'))
            wait(lambda: any(n.name == 'Modèle Transcription' and n.getState().contains(pyatspi.STATE_SHOWING)
                             for n in walk(self.dialog('Préférences'))))
            wait(lambda: self.value('Modèle Transcription', dialog))
            elapsed.append(round((time.perf_counter() - started) * 1000, 2))
            self.activate('Annuler', 'button', dialog)
            wait(lambda: len(self.frames()) == 1)
            time.sleep(.3)
            print('Visible preferences trial ' + str(i + 1) + ': ' + str(elapsed[-1]) + ' ms', flush=True)
        (self.root / 'latency.json').write_text(json.dumps(elapsed) + '\n')
        count = sum(value < 300 for value in elapsed)
        assert count >= 29, (count, 'of 30 below 300 ms')
        self.passed('SC-007: ' + str(count) + '/30 loaded preferences visible below 300 ms (AT-SPI observation)')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dump', action='store_true')
    parser.add_argument('--section', choices=['preferences', 'files', 'keyboard', 'latency'], default='preferences')
    parser.add_argument('--keyboard-socket', required=True, help='Existing user ydotoold socket')
    args = parser.parse_args()
    assert os.environ.get('XDG_SESSION_TYPE') == 'wayland'
    assert 'GNOME' in os.environ.get('XDG_CURRENT_DESKTOP', '')
    assert subprocess.check_output(['gsettings', 'get', 'org.gnome.desktop.input-sources', 'sources'], text=True).strip() == "[('xkb', 'fr')]", 'Recipe requires French AZERTY'
    binary = REPO / 'target/gnome-acceptance/vocalnotes'
    root = REPO / 'target/gnome-acceptance' / time.strftime('%Y%m%d-%H%M%S')
    root.mkdir(parents=True, exist_ok=False)
    env = {**os.environ, 'GDK_BACKEND': 'wayland'}
    for key, folder in [('XDG_CONFIG_HOME', 'config'), ('XDG_DATA_HOME', 'data'), ('XDG_CACHE_HOME', 'cache')]:
        (root / folder).mkdir()
        env[key] = str(root / folder)
    report = {'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
              'session': 'GNOME/Wayland', 'root': str(root)}
    with (root / 'application.log').open('w') as output:
        process = subprocess.Popen([str(binary)], env=env, stdout=output, stderr=output)
        recipe = None
        try:
            recipe = Recipe(process, root, args.keyboard_socket)
            if args.dump:
                wait(lambda: any(n.name == 'Note visuelle' for n in walk(recipe.app)))
                recipe.dump()
            else:
                getattr(recipe, args.section)()
        except Exception as error:
            report['failure'] = repr(error)
            if recipe:
                try:
                    recipe.dump()
                except GLib.GError:
                    print("Accessibility subtree disappeared during failure dump", flush=True)
            raise
        finally:
            if recipe:
                report['passed'] = recipe.results
            (root / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
            # This child has only synthetic data and isolated settings.
            process.terminate()
            process.wait(timeout=10)
            print('Report: ' + str(root / 'report.json'), flush=True)


if __name__ == '__main__':
    main()
