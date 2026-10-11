import numpy


def test_numpy_sums():
    assert numpy.arange(4).sum() == 6, "numpy summed 0..3 wrong"
