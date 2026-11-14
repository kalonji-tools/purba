from app import slugify
from form_module import case


def act(text: str) -> str:
    return slugify(text)


wrong = case(text=42, product="42")
