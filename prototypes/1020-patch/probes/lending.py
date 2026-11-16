class Library:
    def lend(self, title):
        return f"lent {title}"


def borrow(title):
    return Library().lend(title)


def today():
    import datetime

    return datetime.date.today().isoformat()
