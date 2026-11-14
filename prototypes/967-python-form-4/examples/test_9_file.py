"""9: a step every case of this file takes first; a function the test hands to the code; a loop over literal data."""

from app import no_handlers, on, publish
from cases import before_each, case

before_each(no_handlers)  # each case starts with no handler: not even the one that importing `app` added


def test_tagged() -> None:
    seen: list[str] = []

    def tag(event: str) -> None:  # handed to the code, which calls it: its `if` is the code's path, not the test's
        if event.startswith("!"):
            seen.append("loud " + event[1:])
        else:
            seen.append("quiet " + event)

    on(tag)
    for event in ("a", "!b", "c"):  # a loop over literal data runs one path
        publish(event)
    assert seen == ["quiet a", "loud b", "quiet c"]


def test_counted() -> None:
    on(print)
    count = publish("x")
    assert count == 1, "only the test's own handler: before_each removed the import's"


def test_logic_of_its_own(word: str) -> None:
    def shout(text: str) -> str:  # the test calls it itself, so its body is part of the act
        return text.upper() if text else "-"

    seen: list[str] = []
    on(seen.append)
    for event in word.split():  # a loop over computed data has a path for each length
        publish(shout(event))
    assert seen == ["A", "B"]


both = case(test_logic_of_its_own, word="a b")
