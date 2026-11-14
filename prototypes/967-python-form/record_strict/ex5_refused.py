"""ex5: what the parse refuses before anything runs. Lines marked `# ty` are seen by a type checker too."""

from app import Money, adult_count, check_age, parse_amount, slugify
from form_record_strict import Experiment


class Nested(Experiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text.strip())


a1 = Nested(text="Hi").expect(product="hi")


class Literal(Experiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text, "_")  # passes under --literals


a2 = Literal(text="a b").expect(product="a_b")


class TypoKey(Experiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text)


a3 = TypoKey(text="Hi").expect(prodcut="hi")  # ty


class ProductOnRaise(Experiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text)


a4 = ProductOnRaise(text=" ").raises(ValueError, "empty").expect(product="")


class ReadsLiteral(Experiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text)

    def echoed(self) -> str:
        return self.text


a5 = ReadsLiteral(text="Hi").expect(product="hi", echoed="Hi")


class MissingValue(Experiment[Money]):
    text: str
    currency: str

    def act(self) -> Money:
        return parse_amount(self.text, self.currency)

    def cents(self) -> int:
        return self.product.cents

    def unit(self) -> str:
        return self.product.currency


a6 = MissingValue(text="12.50", currency="EUR").expect(cents=1250)


class ReadingNotLiteral(Experiment[Money]):
    text: str
    currency: str

    def act(self) -> Money:
        return parse_amount(self.text, self.currency)


a7 = ReadingNotLiteral(text="12.50", currency="EUR").expect(product="12.50 EUR")  # ty


def age_of(text: str) -> int:  # a helper of this test file: logic the parse never reads
    return check_age(int(text))


class Aged(Experiment[int]):
    text: str

    def act(self) -> int:
        return age_of(self.text)


a8 = Aged(text="30").expect(product=30)


class Adults(Experiment[int]):
    ages: list[int]

    def act(self) -> int:
        return adult_count(self.ages)


two = Adults(ages=[30, 40]).expect(product=2)
one = Adults(ages=[30]).expect(product=1)  # app returns True: == passes it, the same-type compare does not
