# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import ctypes
import datetime
import inspect
import platform
import re

import msgpack
import pytest

import ormsgpack

SIMPLE_TYPES = (1, 1.0, -1, None, "str", True, False)


def test_simple_msgpack() -> None:
    """
    packb() equivalent to msgpack on simple types
    """
    for obj in SIMPLE_TYPES:
        assert ormsgpack.packb(obj, option=None) == msgpack.packb(obj)


def test_simple_round_trip() -> None:
    """
    packb(), unpackb() round trip on simple types
    """
    for obj in SIMPLE_TYPES:
        assert ormsgpack.unpackb(ormsgpack.packb(obj, option=None), option=None) == obj


def test_unpackb_invalid_type() -> None:
    """
    unpackb() invalid type
    """
    val: object
    for val in (1, 3.14, [], {}, None):
        with pytest.raises(ormsgpack.MsgpackDecodeError):
            ormsgpack.unpackb(val)  # type: ignore[arg-type]


def test_unpackb_bytes() -> None:
    assert ormsgpack.unpackb(b"\x90") == []


def test_unpackb_bytearray() -> None:
    assert ormsgpack.unpackb(bytearray(b"\x90")) == []


def test_unpackb_memoryview() -> None:
    assert ormsgpack.unpackb(memoryview(b"\x90")) == []


def test_bytes_round_trip() -> None:
    assert (
        ormsgpack.unpackb(ormsgpack.packb(b"\x01\x02\x03"), option=None)
        == b"\x01\x02\x03"
    )


def test_bytearray_round_trip() -> None:
    assert (
        ormsgpack.unpackb(ormsgpack.packb(bytearray(b"\x01\x02\x03")), option=None)
        == b"\x01\x02\x03"
    )


def test_memoryview_round_trip() -> None:
    assert (
        ormsgpack.unpackb(ormsgpack.packb(memoryview(b"\x01\x02\x03")), option=None)
        == b"\x01\x02\x03"
    )


@pytest.mark.parametrize(
    "value",
    (
        b"\xa2a\xc0",
        b"\xa2a",
        b"\xd9\x02a\xc0",
        b"\xd9\x02a",
        b"\xda\x02a\xc0",
        b"\xda\x02a",
        b"\xdb\x02a\xc0",
        b"\xdb\x02a",
        b"\xc4\x02a",
        b"\xc5\x02a",
        b"\xc6\x02a",
        b"\xc1",
        b"\x91",
        b"\x91\xc1",
        b"\xdc\x00\x01",
        b"\xdc\x00\x01\xc1",
        b"\xdd\x00\x00\x00\x01",
        b"\xdd\x00\x00\x00\x01\xc1",
        b"\x81",
        b"\x81\xc1\x00",
        b"\xde\x00\x01",
        b"\xde\x00\x01\xc1\x00",
        b"\xdf\x00\x00\x00\x01",
        b"\xdf\x00\x00\x00\x01\xc1\x00",
    ),
)
def test_invalid_data(value: bytes) -> None:
    with pytest.raises(ormsgpack.MsgpackDecodeError):
        ormsgpack.unpackb(value)

    with pytest.raises(ValueError):
        ormsgpack.Fragment(value)


def test_unpackb_recursion() -> None:
    """
    unpackb() recursion limit
    """
    with pytest.raises(ormsgpack.MsgpackDecodeError):
        ormsgpack.unpackb(b"\x91" * (1024 * 1024))


def test_version() -> None:
    """
    __version__
    """
    assert re.match(r"^\d+\.\d+(\.\d+)?$", ormsgpack.__version__)


def test_valueerror() -> None:
    """
    ormsgpack.MsgpackDecodeError is a subclass of ValueError
    """
    with pytest.raises(ormsgpack.MsgpackDecodeError):
        ormsgpack.unpackb(b"\x91")
    with pytest.raises(ValueError):
        ormsgpack.unpackb(b"\x91")


@pytest.mark.parametrize(
    "args",
    (
        [],
        [None],
        [None, None],
        [None, None, None],
    ),
)
@pytest.mark.parametrize(
    "kwargs",
    (
        {},
        {"default": None},
        {"option": None},
        {"default": None, "option": None},
    ),
)
def test_packb_args(args: list[None], kwargs: dict[str, None]) -> None:
    if (
        args
        and not (len(args) >= 2 and "default" in kwargs)
        and not (len(args) >= 3 and "option" in kwargs)
    ):
        assert ormsgpack.packb(*args, **kwargs) == b"\xc0"
    else:
        with pytest.raises(TypeError):
            ormsgpack.packb(*args, **kwargs)


def test_packb_unknown_kwarg() -> None:
    with pytest.raises(TypeError):
        ormsgpack.packb(None, zxc=None)  # type: ignore[call-arg]


def test_packb_non_interned_kwarg() -> None:
    assert (
        ormsgpack.packb(
            None,
            **{
                b"default".decode(): None,
                b"option".decode(): None,
            },
        )
        == b"\xc0"
    )


@pytest.mark.parametrize(
    "args",
    (
        [],
        [b"\xc0"],
    ),
)
@pytest.mark.parametrize(
    "kwargs",
    (
        {},
        {"ext_hook": None},
        {"option": None},
        {"ext_hook": None, "option": None},
    ),
)
def test_unpackb_args(args: list[bytes], kwargs: dict[str, None]) -> None:
    if args:
        assert ormsgpack.unpackb(*args, **kwargs) is None
    else:
        with pytest.raises(ValueError):
            ormsgpack.unpackb(*args, **kwargs)


def test_unpackb_unknown_kwarg() -> None:
    with pytest.raises(ValueError):
        ormsgpack.unpackb(b"\xc0", zxc=None)  # type: ignore[call-arg]


def test_unpackb_non_interned_kwarg() -> None:
    assert (
        ormsgpack.unpackb(
            b"\xc0",
            **{
                b"ext_hook".decode(): None,
                b"option".decode(): None,
            },
        )
        is None
    )


@pytest.mark.parametrize(
    "option",
    (
        1 << 16,
        True,
        -1,
        9223372036854775809,
    ),
)
def test_packb_invalid_option(option: int) -> None:
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(True, option=option)


@pytest.mark.parametrize(
    "option",
    (
        ormsgpack.OPT_NAIVE_UTC,
        ormsgpack.OPT_OMIT_MICROSECONDS,
        ormsgpack.OPT_PASSTHROUGH_BIG_INT,
        ormsgpack.OPT_PASSTHROUGH_DATACLASS,
        ormsgpack.OPT_PASSTHROUGH_DATETIME,
        ormsgpack.OPT_PASSTHROUGH_SUBCLASS,
        ormsgpack.OPT_PASSTHROUGH_TUPLE,
        ormsgpack.OPT_SERIALIZE_NUMPY,
        ormsgpack.OPT_SERIALIZE_PYDANTIC,
        ormsgpack.OPT_SORT_KEYS,
        ormsgpack.OPT_UTC_Z,
        True,
        -1,
        9223372036854775809,
    ),
)
def test_unpackb_invalid_option(option: int) -> None:
    with pytest.raises(ormsgpack.MsgpackDecodeError):
        ormsgpack.unpackb(b"\x00", option=option)


def test_opts_multiple() -> None:
    """
    packb() multiple option
    """
    assert ormsgpack.packb(
        [1, datetime.datetime(2000, 1, 1, 2, 3, 4)],
        option=ormsgpack.OPT_SERIALIZE_NUMPY | ormsgpack.OPT_NAIVE_UTC,
    ) == msgpack.packb([1, "2000-01-01T02:03:04+00:00"])


@pytest.mark.skipif(
    platform.python_implementation() == "GraalVM",
    reason="function signatures are missing on GraalPy",
)
def test_packb_signature() -> None:
    """
    packb() valid __text_signature__
    """
    assert (
        str(inspect.signature(ormsgpack.packb)) == "(obj, /, default=None, option=None)"
    )
    inspect.signature(ormsgpack.packb).bind("str")
    inspect.signature(ormsgpack.packb).bind("str", default=None, option=1)


@pytest.mark.skipif(
    platform.python_implementation() == "GraalVM",
    reason="function signatures are missing on GraalPy",
)
def test_unpackb_signature() -> None:
    """
    unpackb() valid __text_signature__
    """
    assert (
        str(inspect.signature(ormsgpack.unpackb))
        == "(obj, /, *, ext_hook=None, option=None)"
    )
    inspect.signature(ormsgpack.unpackb).bind("[]")


def test_packb_module_str() -> None:
    """
    ormsgpack.packb.__module__ is a str
    """
    assert ormsgpack.packb.__module__ == "ormsgpack.ormsgpack"


def test_unpackb_module_str() -> None:
    """
    ormsgpack.unpackb.__module__ is a str
    """
    assert ormsgpack.unpackb.__module__ == "ormsgpack.ormsgpack"


def test_bytes_buffer() -> None:
    """
    packb() trigger buffer growing where length is greater than growth
    """
    a = "a" * 900
    b = "b" * 4096
    c = "c" * 4096 * 4096
    assert ormsgpack.packb([a, b, c]) == msgpack.packb([a, b, c])


@pytest.mark.skipif(
    not hasattr(ctypes, "pythonapi"),
    reason="ctypes.pythonapi not available",
)
def test_function_flags() -> None:
    """
    Make sure we use fastcall
    """
    FASTCALL = 0x0080
    KEYWORDS = 0x0002
    ctypes.pythonapi.PyCFunction_GetFlags.argtypes = [ctypes.py_object]
    packb_flags = ctypes.pythonapi.PyCFunction_GetFlags(ormsgpack.packb)
    unpackb_flags = ctypes.pythonapi.PyCFunction_GetFlags(ormsgpack.unpackb)
    flags = FASTCALL | KEYWORDS
    assert packb_flags & flags == flags
    assert unpackb_flags & flags == flags
