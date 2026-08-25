from __future__ import annotations

from pathlib import Path

import pytest

from keylex.core.registry import GrammarError, Registry

CONFIG_DIR = Path(__file__).parent.parent / "src" / "keylex" / "config"


def make_registry(tmp_path: Path, *, actions: str, targets: str, devices: str) -> Registry:
    (tmp_path / "actions.toml").write_text(actions)
    (tmp_path / "targets.toml").write_text(targets)
    (tmp_path / "devices.toml").write_text(devices)
    return Registry(tmp_path)


def test_loads_shipped_config_without_error():
    registry = Registry(CONFIG_DIR)
    registry.load()  # would raise GrammarError if actions/targets/devices disagree

    assert "close.tab" in registry.actions
    assert registry.target_for_process("Code.exe") is not None


def test_action_spec_defaults_when_not_overridden():
    registry = Registry(CONFIG_DIR)
    registry.load()

    spec = registry.action_spec("comment.line")  # valid grammar, no [[action]] override
    assert spec.fallback_tier == "notify_attempt"
    assert spec.fallback_keycode is None


def test_binding_for_matches_key_and_modifier_set_order_independent():
    registry = Registry(CONFIG_DIR)
    registry.load()

    # devices.toml declares modifiers = ["win", "ctrl", "shift"]; a differently
    # ordered but equal set must still match.
    binding = registry.binding_for("main_keyboard", key="b", modifiers=["shift", "win", "ctrl"])
    assert binding is not None
    assert binding["event"] == "system.reset_graphics_driver"

    assert registry.binding_for("main_keyboard", key="w", modifiers=["ctrl"])["event"] == "close.tab"


def test_binding_mode_defaults_to_device_default_then_observe(tmp_path):
    registry = make_registry(
        tmp_path,
        actions='[[verb]]\nname = "close"\nobjects = ["tab"]\n',
        targets="",
        devices="""
[[device]]
id = "kbd"
type = "keyboard"
source = "auto"
default_mode = "grab"

  [[device.binding]]
  key = "a"
  modifiers = []
  event = "close.tab"

[[device]]
id = "kbd_no_default"
type = "keyboard"
source = "auto"

  [[device.binding]]
  key = "b"
  modifiers = []
  event = "close.tab"
""",
    )
    registry.load()

    assert registry.binding_for("kbd", key="a", modifiers=[])["mode"] == "grab"
    assert registry.binding_for("kbd_no_default", key="b", modifiers=[])["mode"] == "observe"


def test_unknown_verb_in_action_id_raises(tmp_path):
    registry = make_registry(
        tmp_path,
        actions='[[verb]]\nname = "close"\nobjects = ["tab"]\n',
        targets="""
[[target]]
program = "x"
match_process = ["x.exe"]
adapter = "socket"

  [target.supports]
  "frobnicate.tab" = "does.not.matter"
""",
        devices="",
    )

    with pytest.raises(GrammarError):
        registry.load()


def test_unknown_object_for_known_verb_raises(tmp_path):
    registry = make_registry(
        tmp_path,
        actions='[[verb]]\nname = "close"\nobjects = ["tab"]\n',
        targets="""
[[target]]
program = "x"
match_process = ["x.exe"]
adapter = "socket"

  [target.supports]
  "close.window" = "does.not.matter"
""",
        devices="",
    )

    with pytest.raises(GrammarError):
        registry.load()


def test_bare_action_without_dot_is_allowed(tmp_path):
    registry = make_registry(
        tmp_path,
        actions='[[action]]\nid = "save"\nfallback_tier = "silent"\nfallback_keycode = "ctrl+s"\n',
        targets="",
        devices="",
    )

    registry.load()  # must not raise: "save" is a declared bare action

    assert registry.action_spec("save").fallback_keycode == "ctrl+s"


def test_system_action_ids_are_exempt_from_verb_object_grammar(tmp_path):
    registry = make_registry(
        tmp_path,
        actions="",
        targets="""
[[system_action]]
id = "system.reset_graphics_driver"
os = "windows"
keycode = "win+ctrl+shift+b"
""",
        devices="""
[[device]]
id = "kbd"
type = "keyboard"
source = "auto"

  [[device.binding]]
  key = "b"
  modifiers = ["win", "ctrl", "shift"]
  event = "system.reset_graphics_driver"
""",
    )

    registry.load()  # must not raise
