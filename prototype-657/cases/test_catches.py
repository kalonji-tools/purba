def test_catches_everything():
    try:
        assert 2 == 2
    except BaseException:
        pass
