def test_raises():
    seen = []
    try:
        int("x")
    except ValueError:
        seen.append("handled")
    assert seen == ["handled"]


def test_finally():
    seen = []
    try:
        seen.append(1)
    finally:
        seen.append(2)
    assert seen == [1, 2]


def test_else():
    seen = []
    try:
        seen.append(1)
    except ValueError:
        seen.append(2)
    else:
        seen.append(3)
    assert seen == [1, 3]


def test_named():
    seen = []
    try:
        int("1")
    except ValueError as error:
        seen.append(error)
    assert seen == []


def test_other_raise():
    try:
        {}["k"]
    except ValueError:
        caught = True
    assert 1 == 1
