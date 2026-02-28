# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import msgpack
import pytest

import ormsgpack

from .params import INT_PARAMS


@pytest.mark.parametrize("value", INT_PARAMS)
def test_int_64(value: int) -> None:
    packed = ormsgpack.packb(value)
    assert packed == msgpack.packb(value)
    assert ormsgpack.unpackb(packed) == value

    obj = {value: True}
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(obj)
    packed = ormsgpack.packb(obj, option=ormsgpack.OPT_NON_STR_KEYS)
    assert packed == msgpack.packb(obj)
    with pytest.raises(ormsgpack.MsgpackDecodeError):
        ormsgpack.unpackb(packed)
    assert ormsgpack.unpackb(packed, option=ormsgpack.OPT_NON_STR_KEYS) == obj


@pytest.mark.parametrize(
    "value",
    (
        -9223372036854775809,
        18446744073709551616,
    ),
)
def test_int_128(value: int) -> None:
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(value)


@pytest.mark.parametrize(
    "value",
    (
        -9223372036854775807,
        9223372036854775807,
        18446744073709551615,
    ),
)
def test_int_64_passthrough(value: int) -> None:
    assert (
        ormsgpack.unpackb(
            ormsgpack.packb(value, option=ormsgpack.OPT_PASSTHROUGH_BIG_INT)
        )
        == value
    )


@pytest.mark.parametrize(
    "value",
    (
        -9223372036854775809,
        18446744073709551616,
    ),
)
def test_int_128_passthrough(value: int) -> None:
    result = ormsgpack.unpackb(
        ormsgpack.packb(
            value,
            option=ormsgpack.OPT_PASSTHROUGH_BIG_INT,
            default=lambda x: {"int": x.to_bytes(16, "little", signed=True)},
        )
    )
    assert list(result.keys()) == ["int"]
    assert int.from_bytes(result["int"], "little", signed=True) == value
