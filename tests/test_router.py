from __future__ import annotations

from unittest.mock import MagicMock

from keylex.core.registry import ActionSpec, Registry, Target
from keylex.core.router import Router


def make_registry_stub(*, target: Target | None, spec: ActionSpec) -> Registry:
    registry = MagicMock(spec=Registry)
    registry.target_for_process.return_value = target
    registry.action_spec.return_value = spec
    return registry


def test_dispatch_uses_native_adapter_when_target_supports_action():
    target = Target(
        program="vscode", match_process=["Code.exe"], adapter="socket",
        supports={"close.tab": "workbench.action.closeActiveEditor"},
    )
    registry = make_registry_stub(target=target, spec=ActionSpec(id="close.tab"))
    adapter = MagicMock()
    router = Router(registry, {"socket": adapter}, MagicMock(), MagicMock())

    result = router.dispatch("close.tab", focused_process="Code.exe")

    assert result.status == "native"
    assert result.detail == "workbench.action.closeActiveEditor"
    adapter.send.assert_called_once_with(target, "workbench.action.closeActiveEditor")


def test_dispatch_falls_back_when_target_does_not_support_action():
    target = Target(program="vscode", match_process=["Code.exe"], adapter="socket", supports={})
    spec = ActionSpec(id="save", fallback_tier="silent", fallback_keycode="ctrl+s")
    registry = make_registry_stub(target=target, spec=spec)
    fallback_sender = MagicMock()
    notifier = MagicMock()
    router = Router(registry, {"socket": MagicMock()}, notifier, fallback_sender)

    result = router.dispatch("save", focused_process="Code.exe")

    assert result.status == "fallback"
    assert result.detail == "ctrl+s"
    fallback_sender.send.assert_called_once_with("ctrl+s")
    notifier.show.assert_not_called()  # silent tier: no popup


def test_dispatch_notifies_on_fallback_attempt_tier():
    spec = ActionSpec(id="duplicate.line", fallback_tier="notify_attempt", fallback_keycode="ctrl+shift+d")
    registry = make_registry_stub(target=None, spec=spec)
    notifier = MagicMock()
    fallback_sender = MagicMock()
    router = Router(registry, {}, notifier, fallback_sender)

    result = router.dispatch("duplicate.line", focused_process="unknown.exe")

    assert result.status == "fallback"
    fallback_sender.send.assert_called_once_with("ctrl+shift+d")
    notifier.show.assert_called_once()


def test_dispatch_reports_unsupported_when_notify_only_and_no_target():
    spec = ActionSpec(id="go_to.definition", fallback_tier="notify_only", fallback_keycode=None)
    registry = make_registry_stub(target=None, spec=spec)
    notifier = MagicMock()
    router = Router(registry, {}, notifier, MagicMock())

    result = router.dispatch("go_to.definition", focused_process="chrome.exe")

    assert result.status == "unsupported"
    notifier.show.assert_called_once()


def test_dispatch_reports_unsupported_when_native_adapter_missing():
    target = Target(
        program="vscode", match_process=["Code.exe"], adapter="socket",
        supports={"close.tab": "workbench.action.closeActiveEditor"},
    )
    registry = make_registry_stub(target=target, spec=ActionSpec(id="close.tab"))
    router = Router(registry, {}, MagicMock(), MagicMock())  # no "socket" adapter registered

    result = router.dispatch("close.tab", focused_process="Code.exe")

    assert result.status == "unsupported"
