# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import pytest

import ormsgpack

from .params import DictType


def test_circular_dict() -> None:
    """
    packb() circular reference dict
    """
    obj: dict[str, object] = {}
    obj["obj"] = obj
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(obj)


def test_circular_list() -> None:
    """
    packb() circular reference list
    """
    obj: list[object] = []
    obj.append(obj)
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(obj)


def test_circular_nested(dict_type: DictType) -> None:
    """
    packb() circular reference nested dict, list
    """
    values: list[object] = []
    obj = dict_type({"list": values})
    values.append(dict_type({"obj": obj}))
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(obj)
