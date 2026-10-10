# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import builtins
import datetime
import sys
from typing import TypeAlias

import pytest

if sys.version_info >= (3, 15):
    DictType: TypeAlias = type[
        dict[object, object] | builtins.frozendict[object, object]
    ]
    DICT_TYPE_PARAMS: list[DictType] = [dict, builtins.frozendict]
else:
    DictType: TypeAlias = type[dict[object, object]]
    DICT_TYPE_PARAMS = [dict]

INT_PARAMS = (
    pytest.param(1, id="positive fixint"),
    pytest.param(128, id="uint 8"),
    pytest.param(256, id="uint 16"),
    pytest.param(65536, id="uint 32"),
    pytest.param(4294967296, id="uint 64"),
    pytest.param(18446744073709551615, id="uint 64 max"),
    pytest.param(-1, id="negative fixint"),
    pytest.param(-128, id="int 8"),
    pytest.param(-256, id="int 16"),
    pytest.param(-65536, id="int 32"),
    pytest.param(-4294967296, id="int 64"),
    pytest.param(-9223372036854775808, id="int 64 min"),
)

STR_PARAMS = (
    pytest.param("a", id="fixstr"),
    pytest.param("a" * 32, id="str 8"),
    pytest.param("a" * 256, id="str 16"),
    pytest.param("a" * 65536, id="str 32"),
)

BIN_PARAMS = (
    pytest.param(b"a" * 32, id="bin 8"),
    pytest.param(b"a" * 256, id="bin 16"),
    pytest.param(b"a" * 65536, id="bin 32"),
)

ARRAY_PARAMS = (
    pytest.param([0], id="fixarray"),
    pytest.param([i for i in range(16)], id="array 16"),
    pytest.param([i for i in range(65536)], id="array 32"),
)

MAP_PARAMS = (
    pytest.param({"0": 0}, id="fixmap"),
    pytest.param({str(i): i for i in range(16)}, id="map 16"),
    pytest.param({str(i): i for i in range(65536)}, id="map 32"),
)

EXT_PARAMS = (
    pytest.param(b"a" * 1, id="fixext 1"),
    pytest.param(b"a" * 2, id="fixext 2"),
    pytest.param(b"a" * 4, id="fixext 4"),
    pytest.param(b"a" * 8, id="fixext 8"),
    pytest.param(b"a" * 16, id="fixext 16"),
    pytest.param(b"a" * 32, id="ext 8"),
    pytest.param(b"a" * 256, id="ext 16"),
    pytest.param(b"a" * 65536, id="ext 32"),
)

TIMESTAMP_EXT_PARAMS = (
    pytest.param(
        datetime.datetime(1970, 1, 1, tzinfo=datetime.timezone.utc),
        b"\xd6\xff\x00\x00\x00\x00",
        id="timestamp32 min",
    ),
    pytest.param(
        datetime.datetime(2106, 2, 7, 6, 28, 15, tzinfo=datetime.timezone.utc),
        b"\xd6\xff\xff\xff\xff\xff",
        id="timestamp32 max",
    ),
    pytest.param(
        datetime.datetime(2106, 2, 7, 6, 28, 16, tzinfo=datetime.timezone.utc),
        b"\xd7\xff\x00\x00\x00\x01\x00\x00\x00\x00",
        id="timestamp64 required",
    ),
    pytest.param(
        datetime.datetime(2514, 5, 30, 1, 53, 3, tzinfo=datetime.timezone.utc),
        b"\xd7\xff\x00\x00\x00\x03\xff\xff\xff\xff",
        id="timestamp64 max seconds",
    ),
    pytest.param(
        datetime.datetime(2514, 5, 30, 1, 53, 4, tzinfo=datetime.timezone.utc),
        b"\xc7\x0c\xff\x00\x00\x00\x00\x00\x00\x00\x04\x00\x00\x00\x00",
        id="timestamp96 required",
    ),
    pytest.param(
        datetime.datetime(9999, 12, 4, 15, 30, 7, tzinfo=datetime.timezone.utc),
        b"\xc7\x0c\xff\x00\x00\x00\x00\x00\x00\x00:\xff\xd01\x7f",
        id="timestamp96 datetime max year",
    ),
)
