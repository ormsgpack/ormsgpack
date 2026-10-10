# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import collections

import msgpack
import pytest

import ormsgpack

from .params import DictType


class SubStr(str):
    pass


class SubInt(int):
    pass


class SubDict(dict[str, object]):
    pass


class SubList(list[object]):
    pass


class SubFloat(float):
    pass


class SubTuple(tuple[object, object]):
    pass


def test_subclass_str() -> None:
    assert ormsgpack.unpackb(ormsgpack.packb(SubStr("zxc"))) == "zxc"


def test_subclass_str_invalid() -> None:
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(SubStr("\ud800"))

    packed = ormsgpack.packb(SubStr("\ud800"), option=ormsgpack.OPT_REPLACE_SURROGATES)
    assert ormsgpack.unpackb(packed) == "?"


def test_subclass_int() -> None:
    assert ormsgpack.unpackb(ormsgpack.packb(SubInt(1))) == 1


def test_subclass_int_64() -> None:
    for val in (9223372036854775807, -9223372036854775807):
        assert ormsgpack.packb(SubInt(val)) == msgpack.packb(val)


def test_subclass_dict(dict_subclass: DictType) -> None:
    assert ormsgpack.packb(dict_subclass({"a": "b"})) == msgpack.packb({"a": "b"})


def test_subclass_list() -> None:
    assert ormsgpack.packb(SubList(["a", "b"])) == msgpack.packb(["a", "b"])

    ref = [True] * 512
    assert ormsgpack.unpackb(ormsgpack.packb(SubList(ref))) == ref


def test_subclass_float() -> None:
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(SubFloat(1.1))


def test_subclass_tuple() -> None:
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(SubTuple((1, 2)))


def test_namedtuple() -> None:
    Point = collections.namedtuple("Point", ["x", "y"])
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(Point(1, 2))


def test_subclass_circular_dict() -> None:
    obj = SubDict({})
    obj["obj"] = obj
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(obj)


def test_subclass_circular_list() -> None:
    obj = SubList([])
    obj.append(obj)
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(obj)


def test_subclass_circular_nested(dict_subclass: DictType) -> None:
    values = SubList([])
    obj = dict_subclass({"list": values})
    values.append(dict_subclass({"obj": obj}))
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(obj)


def test_subclass_passthrough() -> None:
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(SubStr("zxc"), option=ormsgpack.OPT_PASSTHROUGH_SUBCLASS)


def test_subclass_int_passthrough() -> None:
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(SubInt(1), option=ormsgpack.OPT_PASSTHROUGH_SUBCLASS)


def test_subclass_dict_passthrough(dict_subclass: DictType) -> None:
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(
            dict_subclass({"a": "b"}), option=ormsgpack.OPT_PASSTHROUGH_SUBCLASS
        )


def test_subclass_list_passthrough() -> None:
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(SubList(["a", "b"]), option=ormsgpack.OPT_PASSTHROUGH_SUBCLASS)
