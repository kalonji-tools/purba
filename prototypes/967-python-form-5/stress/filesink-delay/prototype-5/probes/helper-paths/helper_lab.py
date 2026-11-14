"""A helper in another module, with two paths and an assert."""


def check_dir_like(names: list[str], size: int | None = None) -> None:
    if size is not None:
        assert len(names) == size


def two_paths(n: int) -> int:
    if n > 0:
        return n
    return -n
