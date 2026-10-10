import math

import msgpack
import pytest

import ormsgpack

from .params import DictType


@pytest.mark.parametrize(
    "value",
    (
        -1.1234567893,
        -1.234567893,
        -1.34567893,
        -1.4567893,
        -1.567893,
        -1.67893,
        -1.7893,
        -1.893,
        -1.3,
        1.1234567893,
        1.234567893,
        1.34567893,
        1.4567893,
        1.567893,
        1.67893,
        1.7893,
        1.893,
        1.3,
    ),
)
def test_float(value: float) -> None:
    packed = ormsgpack.packb(value)
    assert packed == msgpack.packb(value)
    assert ormsgpack.unpackb(packed) == value


def test_float_dict_key(dict_type: DictType) -> None:
    obj = dict_type({1.1: True})
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(obj)
    packed = ormsgpack.packb(obj, option=ormsgpack.OPT_NON_STR_KEYS)
    assert packed == msgpack.packb(obj)
    with pytest.raises(ormsgpack.MsgpackDecodeError):
        ormsgpack.unpackb(packed)
    assert ormsgpack.unpackb(packed, option=ormsgpack.OPT_NON_STR_KEYS) == obj


def test_float_infinity() -> None:
    value = float("Infinity")
    packed = ormsgpack.packb(value)
    assert packed == msgpack.packb(value)
    assert ormsgpack.unpackb(packed) == value


def test_float_nan() -> None:
    value = float("NaN")
    packed = ormsgpack.packb(value)
    assert packed == msgpack.packb(value)
    assert math.isnan(ormsgpack.unpackb(packed))


@pytest.mark.parametrize(
    "value",
    (
        31.245270191439438,
        -31.245270191439438,
        121.48791951161945,
        -121.48791951161945,
        100.78399658203125,
        -100.78399658203125,
    ),
)
def test_float_precision(value: float) -> None:
    packed = ormsgpack.packb(value)
    assert packed == msgpack.packb(value)
    assert ormsgpack.unpackb(packed) == value


@pytest.mark.parametrize(
    "value",
    (
        0.8701,
        0.0000000000000000000000000000000000000000000000000123e50,
        0.4e5,
        0.00e-00,
        0.4e-001,
        0.123456789e-12,
        1.234567890e34,
        23456789012e66,
    ),
)
def test_float_edge(value: float) -> None:
    packed = ormsgpack.packb(value)
    assert packed == msgpack.packb(value)
    assert ormsgpack.unpackb(packed) == value


@pytest.mark.parametrize(
    "value",
    (
        "1.337E40",
        "1.337e+40",
        "1337e40",
        "1.337E-4",
    ),
)
def test_float_notation(value: float) -> None:
    packed = ormsgpack.packb(value)
    assert packed == msgpack.packb(value)
    assert ormsgpack.unpackb(packed) == value
