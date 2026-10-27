import asyncio, json, decimal, email.parser, unittest
import http.client, sqlite3, logging, dataclasses, typing
import os

with open(os.environ["LAB_COUNT"], "a") as count:  # an effect outside the process
    count.write("x")
CONFIG = {"mode": "default"}


def test():
    assert json.loads('{"a": 1}')["a"] == 1
