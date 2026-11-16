"""The code under test of the patches in test_patch.py."""


class Library:
    def lend(self, title):
        return f"lent {title}"


def borrow(title):
    return Library().lend(title)
