import msgpack
import pytest

import ormsgpack

from .params import EXT_PARAMS


@pytest.mark.parametrize("data", EXT_PARAMS)
def test_ext_type(data: bytes) -> None:
    tag = 1
    value = ormsgpack.Ext(tag, data)
    packed = ormsgpack.packb(value)
    assert packed == msgpack.packb(msgpack.ExtType(tag, data))

    unpacked = ormsgpack.unpackb(
        packed,
        ext_hook=lambda x, y: (x, y),
    )
    assert unpacked == (tag, data)

    with pytest.raises(ormsgpack.MsgpackDecodeError):
        ormsgpack.unpackb(packed)

    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb({value: True}, option=ormsgpack.OPT_NON_STR_KEYS)
