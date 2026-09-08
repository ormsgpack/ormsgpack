# SPDX-License-Identifier: (Apache-2.0 OR MIT)

"""Rejected input must not leak the containers that were built before the error."""

import gc
import tracemalloc
from collections.abc import Callable

import msgpack
import pytest

import ormsgpack

ELEMENTS = 20_000
ROUNDS = 20
# a fully built list of ELEMENTS ints costs roughly ELEMENTS * 32 bytes;
# a leak would retain that much per round.
MAX_RETAINED = ELEMENTS * 32


def _retained_after(decode: Callable[[], None]) -> int:
    tracemalloc.start()
    try:
        gc.collect()
        before, _ = tracemalloc.get_traced_memory()
        for _ in range(ROUNDS):
            with pytest.raises(ormsgpack.MsgpackDecodeError):
                decode()
        gc.collect()
        after, _ = tracemalloc.get_traced_memory()
    finally:
        tracemalloc.stop()
    return after - before


def _truncate(packed: bytes) -> bytes:
    # drop the last element: the container header promises more than is present
    return packed[:-1]


def test_list_error_no_leak() -> None:
    packed = _truncate(msgpack.packb(list(range(ELEMENTS))))

    assert _retained_after(lambda: ormsgpack.unpackb(packed)) < MAX_RETAINED


def test_nested_list_error_no_leak() -> None:
    # the outer list already holds one complete inner list when the second one fails
    packed = _truncate(msgpack.packb([list(range(ELEMENTS)), list(range(ELEMENTS))]))

    assert _retained_after(lambda: ormsgpack.unpackb(packed)) < MAX_RETAINED


def test_dict_str_keys_value_error_no_leak() -> None:
    # value of the second key fails after the first entry was inserted
    packed = _truncate(
        msgpack.packb({"a": list(range(ELEMENTS)), "b": list(range(ELEMENTS))})
    )

    assert _retained_after(lambda: ormsgpack.unpackb(packed)) < MAX_RETAINED


def test_dict_str_keys_key_error_no_leak() -> None:
    # second key is not a str: the dict with its first entry must be released
    packed = msgpack.packb({"a": list(range(ELEMENTS)), 1: 0})

    assert _retained_after(lambda: ormsgpack.unpackb(packed)) < MAX_RETAINED


def test_dict_non_str_keys_error_no_leak() -> None:
    packed = _truncate(
        msgpack.packb({1: list(range(ELEMENTS)), 2: list(range(ELEMENTS))})
    )

    assert (
        _retained_after(
            lambda: ormsgpack.unpackb(packed, option=ormsgpack.OPT_NON_STR_KEYS)
        )
        < MAX_RETAINED
    )


def test_tuple_key_error_no_leak() -> None:
    # array key whose last element is missing
    packed = msgpack.packb({tuple(range(ELEMENTS)): 0})[:-1]

    assert (
        _retained_after(
            lambda: ormsgpack.unpackb(packed, option=ormsgpack.OPT_NON_STR_KEYS)
        )
        < MAX_RETAINED
    )
