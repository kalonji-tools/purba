"""The adapter: one test's view of a resource, rolled back at the test's end."""

from subject import Database

rollback = True  # run_generic.py clears it to plant an adapter that leaks


class Tx[D: Database]:
    def __init__(self, db: D) -> None:
        self.db = db
        self.saved = dict(db.rows)

    def end(self) -> None:
        if rollback:
            self.db.rows.clear()
            self.db.rows.update(self.saved)
