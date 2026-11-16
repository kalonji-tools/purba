def half(n):
    assert n % 2 == 0, "odd"
    return n // 2

def mode(flag):
    return "debug" if flag else "optimized"

def own_mode():
    return "debug" if __debug__ else "optimized"
