# purba: suppress-module[computed-expected] the module once computed its values


# purba: suppress[private-name] the test once read a private name
def test_renamed():
    value = int("3")
    assert value == 3


# purba: suppress[swallowed-raise] the act once swallowed a raise
def test_failed():
    value = int("3")
    assert value == 4


# purba: suppress[second-path] the act once branched
def test_raised():
    value = int("x")
    assert value == 3
