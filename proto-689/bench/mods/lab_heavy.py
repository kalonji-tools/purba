import asyncio, json, decimal, email.parser, unittest
import http.client, sqlite3, logging, dataclasses, typing

CONFIG = {"mode": "default"}


def test():
    assert json.loads('{"a": 1}')["a"] == 1
