from app import slugify
from form_module import case


def act(text: str) -> str:
    return slugify(text)


a = case(text="Hi", slgu="hi")
