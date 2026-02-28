# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import pytest

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
