# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import msgpack
import pytest

import ormsgpack

from .params import MAP_PARAMS, DictType


@pytest.mark.parametrize("value", MAP_PARAMS)
def test_dict(dict_type: DictType, value: dict[str, int]) -> None:
    packed = ormsgpack.packb(dict_type(value))
    assert packed == msgpack.packb(value)
    assert ormsgpack.unpackb(packed) == value


def test_dict_large(dict_type: DictType) -> None:
    cache_size = 512
    obj = dict_type({str(i): i for i in range(cache_size + 1)})
    assert ormsgpack.unpackb(ormsgpack.packb(obj)) == obj


def test_dict_large_key(dict_type: DictType) -> None:
    max_cached_key_len = 64
    obj = dict_type({"k" * max_cached_key_len * 2: "value"})
    assert ormsgpack.unpackb(ormsgpack.packb(obj)) == obj


def test_dict_unicode_key(dict_type: DictType) -> None:
    obj = dict_type({"🚀": "value"})
    assert ormsgpack.unpackb(ormsgpack.packb(obj)) == obj


def test_dict_similar_keys(dict_type: DictType) -> None:
    """
    unpackb() similar keys

    This was a regression in 3.4.2 caused by using
    the implementation in wy instead of wyhash.
    """
    obj = dict_type({"cf_status_firefox67": "---", "cf_status_firefox57": "verified"})
    assert ormsgpack.unpackb(ormsgpack.packb(obj)) == obj
