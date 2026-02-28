# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import msgpack
import pytest

import ormsgpack

from .params import STR_PARAMS


@pytest.mark.parametrize("value", STR_PARAMS)
def test_str(value: str) -> None:
    packed = ormsgpack.packb(value)
    assert packed == msgpack.packb(value)
    assert ormsgpack.unpackb(packed) == value

    obj = {value: True}
    packed = ormsgpack.packb(obj)
    assert packed == msgpack.packb(obj)
    assert ormsgpack.unpackb(packed) == obj


@pytest.mark.parametrize(
    "value",
    (
        pytest.param("\u00b5\u00b7", id="255"),
        pytest.param("\u03b1\u03c9", id="65535"),
        pytest.param("\U0001f680", id="1114111"),
    ),
)
def test_str_max_code_point(value: str) -> None:
    packed = ormsgpack.packb(value)
    assert packed == msgpack.packb(value)
    assert ormsgpack.unpackb(packed) == value


def test_str_surrogates() -> None:
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb("\ud800")
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb("\udc00")
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb("\ud83d\ude80")

    packed = ormsgpack.packb("\ud800", option=ormsgpack.OPT_REPLACE_SURROGATES)
    assert ormsgpack.unpackb(packed) == "?"
