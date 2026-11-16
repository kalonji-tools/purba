import lending


class Stub:
    def lend(self, title):
        return "stub"


def test_assign_module_attr():
    lending.Library = Stub
    got = lending.borrow("Dune")
    assert got == "stub"
