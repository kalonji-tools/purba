"""ex11: one experiment at both levels. A test grows by changing its base and declaring `expect`; nothing else changes."""

from typing import Self

from app import Money, parse_amount
from form_record_strict import UNREAD, Experiment, TypedExperiment


class AmountOf(Experiment[Money]):  # small: no `expect` to write
    text: str
    currency: str | None

    def act(self) -> Money:
        return parse_amount(self.text, self.currency)

    def cents(self) -> int:
        return self.product.cents


euros = AmountOf(text="12.50", currency="EUR").expect(cents=1250)
as_text = AmountOf(text="12.50", currency="EUR").expect(cents="1250")  # purba refuses it before the run


class TypedAmountOf(TypedExperiment[Money]):  # grown: the same class, another base, and a declared `expect`
    text: str
    currency: str | None

    def act(self) -> Money:
        return parse_amount(self.text, self.currency)

    def cents(self) -> int:
        return self.product.cents

    def expect(self, *, cents: int = UNREAD) -> Self:
        return self


typed_euros = TypedAmountOf(text="12.50", currency="EUR").expect(cents=1250)
typed_as_text = TypedAmountOf(text="12.50", currency="EUR").expect(cents="1250")  # the editor sees it, and purba
