# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import msgpack
import pytest

import ormsgpack

from .params import DictType


@pytest.mark.parametrize("value", (True, False))
def test_bool(value: bool) -> None:
    packed = ormsgpack.packb(value)
    assert packed == msgpack.packb(value)
    assert ormsgpack.unpackb(packed) == value


@pytest.mark.parametrize("value", (True, False))
def test_bool_dict_key(dict_type: DictType, value: bool) -> None:
    obj = dict_type({value: True})
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(obj)
    packed = ormsgpack.packb(obj, option=ormsgpack.OPT_NON_STR_KEYS)
    assert packed == msgpack.packb(obj)
    with pytest.raises(ormsgpack.MsgpackDecodeError):
        ormsgpack.unpackb(packed)
    assert ormsgpack.unpackb(packed, option=ormsgpack.OPT_NON_STR_KEYS) == obj
