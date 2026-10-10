# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import datetime
import zoneinfo
from collections.abc import Callable

import msgpack
import pytest
import pytz
from dateutil import tz
from dateutil.zoneinfo import get_zonefile_instance

import ormsgpack

from .params import TIMESTAMP_EXT_PARAMS, DictType

try:
    import pendulum
except ImportError:
    pendulum_timezone = None
    pendulum_UTC = None
else:
    pendulum_timezone = pendulum.timezone
    pendulum_UTC = pendulum.UTC


TIMEZONE_PARAMS = (
    pytest.param(
        pendulum_timezone,
        id="pendulum",
        marks=pytest.mark.skipif(
            pendulum_timezone is None,
            reason="pendulum not available",
        ),
    ),
    pytest.param(pytz.timezone, id="pytz"),
    pytest.param(get_zonefile_instance().get, id="dateutil"),
    pytest.param(zoneinfo.ZoneInfo, id="zoneinfo"),
)


def test_datetime_naive() -> None:
    """
    datetime.datetime naive prints without offset
    """
    assert ormsgpack.packb(
        [datetime.datetime(2000, 1, 1, 2, 3, 4, 123)]
    ) == msgpack.packb(["2000-01-01T02:03:04.000123"])


def test_datetime_naive_utc() -> None:
    """
    datetime.datetime naive with opt assumes UTC
    """
    assert ormsgpack.packb(
        [datetime.datetime(2000, 1, 1, 2, 3, 4, 123)],
        option=ormsgpack.OPT_NAIVE_UTC,
    ) == msgpack.packb(["2000-01-01T02:03:04.000123+00:00"])


def test_datetime_min() -> None:
    """
    datetime.datetime min range
    """
    assert ormsgpack.packb(
        [datetime.datetime(datetime.MINYEAR, 1, 1, 0, 0, 0, 0)],
        option=ormsgpack.OPT_NAIVE_UTC,
    ) == msgpack.packb(["0001-01-01T00:00:00+00:00"])


def test_datetime_max() -> None:
    """
    datetime.datetime max range
    """
    assert ormsgpack.packb(
        [datetime.datetime(datetime.MAXYEAR, 12, 31, 23, 59, 50, 999999)],
        option=ormsgpack.OPT_NAIVE_UTC,
    ) == msgpack.packb(["9999-12-31T23:59:50.999999+00:00"])


def test_datetime_three_digits() -> None:
    """
    datetime.datetime three digit year
    """
    assert ormsgpack.packb(
        [datetime.datetime(312, 1, 1)],
        option=ormsgpack.OPT_NAIVE_UTC,
    ) == msgpack.packb(["0312-01-01T00:00:00+00:00"])


def test_datetime_two_digits() -> None:
    """
    datetime.datetime two digit year
    """
    assert ormsgpack.packb(
        [datetime.datetime(46, 1, 1)],
        option=ormsgpack.OPT_NAIVE_UTC,
    ) == msgpack.packb(["0046-01-01T00:00:00+00:00"])


def test_datetime_tz_assume() -> None:
    """
    datetime.datetime tz with assume UTC uses tz
    """
    assert ormsgpack.packb(
        [datetime.datetime(2018, 1, 1, 2, 3, 4, 0, tzinfo=tz.gettz("Asia/Shanghai"))],
        option=ormsgpack.OPT_NAIVE_UTC,
    ) == msgpack.packb(["2018-01-01T02:03:04+08:00"])


@pytest.mark.parametrize(
    "timezone",
    (
        pytest.param(datetime.timezone.utc, id="datetime"),
        pytest.param(
            pendulum_UTC,
            id="pendulum",
            marks=pytest.mark.skipif(
                pendulum_UTC is None,
                reason="pendulum not available",
            ),
        ),
        pytest.param(pytz.UTC, id="pytz"),
        pytest.param(tz.UTC, id="dateutil"),
        pytest.param(zoneinfo.ZoneInfo("UTC"), id="zoneinfo"),
    ),
)
def test_datetime_utc(timezone: datetime.tzinfo) -> None:
    """
    datetime.datetime UTC
    """
    assert ormsgpack.packb(
        [datetime.datetime(2018, 6, 1, 2, 3, 4, 0, tzinfo=timezone)]
    ) == msgpack.packb(["2018-06-01T02:03:04+00:00"])


@pytest.mark.parametrize("timezone", TIMEZONE_PARAMS)
def test_datetime_positive(timezone: Callable[[str], datetime.tzinfo]) -> None:
    """
    datetime.datetime positive UTC
    """
    assert ormsgpack.packb(
        [
            datetime.datetime(
                2018,
                1,
                1,
                2,
                3,
                4,
                0,
                tzinfo=timezone("Asia/Shanghai"),
            )
        ]
    ) == msgpack.packb(["2018-01-01T02:03:04+08:00"])


@pytest.mark.parametrize("timezone", TIMEZONE_PARAMS)
def test_datetime_negative_dst(timezone: Callable[[str], datetime.tzinfo]) -> None:
    """
    datetime.datetime negative UTC DST
    """
    assert ormsgpack.packb(
        [
            datetime.datetime(
                2018,
                6,
                1,
                2,
                3,
                4,
                0,
                tzinfo=timezone("America/New_York"),
            )
        ]
    ) == msgpack.packb(["2018-06-01T02:03:04-04:00"])


@pytest.mark.parametrize("timezone", TIMEZONE_PARAMS)
def test_datetime_negative_non_dst(timezone: Callable[[str], datetime.tzinfo]) -> None:
    """
    datetime.datetime negative UTC non-DST
    """
    assert ormsgpack.packb(
        [
            datetime.datetime(
                2018,
                12,
                1,
                2,
                3,
                4,
                0,
                tzinfo=timezone("America/New_York"),
            )
        ]
    ) == msgpack.packb(["2018-12-01T02:03:04-05:00"])


@pytest.mark.parametrize("timezone", TIMEZONE_PARAMS)
def test_datetime_partial_hour(timezone: Callable[[str], datetime.tzinfo]) -> None:
    """
    datetime.datetime UTC offset partial hour
    """
    assert ormsgpack.packb(
        [
            datetime.datetime(
                2018,
                12,
                1,
                2,
                3,
                4,
                0,
                tzinfo=timezone("Australia/Adelaide"),
            )
        ]
    ) == msgpack.packb(["2018-12-01T02:03:04+10:30"])


@pytest.mark.parametrize("timezone", TIMEZONE_PARAMS)
def test_datetime_partial_second(timezone: Callable[[str], datetime.tzinfo]) -> None:
    """
    datetime.datetime UTC offset round seconds

    https://tools.ietf.org/html/rfc3339#section-5.8
    """

    # 0:17:30
    assert ormsgpack.packb(
        [
            datetime.datetime(
                1892,
                5,
                1,
                0,
                0,
                0,
                tzinfo=timezone("Europe/Brussels"),
            )
        ]
    ) == msgpack.packb(["1892-05-01T00:00:00+00:18"])

    # 0:09:21
    assert ormsgpack.packb(
        [
            datetime.datetime(
                1911,
                3,
                10,
                0,
                0,
                0,
                tzinfo=timezone("Europe/Paris"),
            )
        ]
    ) == msgpack.packb(["1911-03-10T00:00:00+00:09"])


def test_datetime_microsecond_max() -> None:
    """
    datetime.datetime microsecond max
    """
    assert ormsgpack.packb(
        datetime.datetime(2000, 1, 1, 0, 0, 0, 999999)
    ) == msgpack.packb("2000-01-01T00:00:00.999999")


def test_datetime_microsecond_min() -> None:
    """
    datetime.datetime microsecond min
    """
    assert ormsgpack.packb(datetime.datetime(2000, 1, 1, 0, 0, 0, 1)) == msgpack.packb(
        "2000-01-01T00:00:00.000001"
    )


def test_datetime_omit_microseconds() -> None:
    """
    datetime.datetime OPT_OMIT_MICROSECONDS
    """
    assert ormsgpack.packb(
        [datetime.datetime(2000, 1, 1, 2, 3, 4, 123)],
        option=ormsgpack.OPT_OMIT_MICROSECONDS,
    ) == msgpack.packb(["2000-01-01T02:03:04"])


def test_datetime_omit_microseconds_naive() -> None:
    """
    datetime.datetime naive OPT_OMIT_MICROSECONDS
    """
    assert ormsgpack.packb(
        [datetime.datetime(2000, 1, 1, 2, 3, 4, 123)],
        option=ormsgpack.OPT_NAIVE_UTC | ormsgpack.OPT_OMIT_MICROSECONDS,
    ) == msgpack.packb(["2000-01-01T02:03:04+00:00"])


def test_time_omit_microseconds() -> None:
    """
    datetime.time OPT_OMIT_MICROSECONDS
    """
    assert ormsgpack.packb(
        [datetime.time(2, 3, 4, 123)], option=ormsgpack.OPT_OMIT_MICROSECONDS
    ) == msgpack.packb(["02:03:04"])


def test_datetime_utc_z_naive_omit() -> None:
    """
    datetime.datetime naive OPT_UTC_Z
    """
    assert ormsgpack.packb(
        [datetime.datetime(2000, 1, 1, 2, 3, 4, 123)],
        option=ormsgpack.OPT_NAIVE_UTC
        | ormsgpack.OPT_UTC_Z
        | ormsgpack.OPT_OMIT_MICROSECONDS,
    ) == msgpack.packb(["2000-01-01T02:03:04Z"])


def test_datetime_utc_z_naive() -> None:
    """
    datetime.datetime naive OPT_UTC_Z
    """
    assert ormsgpack.packb(
        [datetime.datetime(2000, 1, 1, 2, 3, 4, 123)],
        option=ormsgpack.OPT_NAIVE_UTC | ormsgpack.OPT_UTC_Z,
    ) == msgpack.packb(["2000-01-01T02:03:04.000123Z"])


def test_datetime_utc_z_without_tz() -> None:
    """
    datetime.datetime naive OPT_UTC_Z
    """
    assert ormsgpack.packb(
        [datetime.datetime(2000, 1, 1, 2, 3, 4, 123)], option=ormsgpack.OPT_UTC_Z
    ) == msgpack.packb(["2000-01-01T02:03:04.000123"])


def test_datetime_utc_z_with_tz() -> None:
    """
    datetime.datetime OPT_UTC_Z
    """
    assert ormsgpack.packb(
        [datetime.datetime(2000, 1, 1, 0, 0, 0, 1, tzinfo=datetime.timezone.utc)],
        option=ormsgpack.OPT_UTC_Z,
    ) == msgpack.packb(["2000-01-01T00:00:00.000001Z"])

    assert ormsgpack.packb(
        [datetime.datetime(2000, 1, 1, 0, 0, 0, 1, tzinfo=tz.gettz("Europe/Brussels"))],
        option=ormsgpack.OPT_UTC_Z,
    ) == msgpack.packb(["2000-01-01T00:00:00.000001+01:00"])


def test_datetime_roundtrip() -> None:
    """
    datetime.datetime parsed by pendulum
    """
    pendulum = pytest.importorskip("pendulum")
    obj = datetime.datetime(2000, 1, 1, 0, 0, 0, 1, tzinfo=datetime.timezone.utc)
    deserialized = ormsgpack.unpackb(ormsgpack.packb(obj))
    parsed = pendulum.parse(deserialized)
    for attr in ("year", "month", "day", "hour", "minute", "second", "microsecond"):
        assert getattr(obj, attr) == getattr(parsed, attr)


def test_date() -> None:
    """
    datetime.date
    """
    assert ormsgpack.packb([datetime.date(2000, 1, 13)]) == msgpack.packb(
        ["2000-01-13"]
    )


def test_date_min() -> None:
    """
    datetime.date MINYEAR
    """
    assert ormsgpack.packb([datetime.date(datetime.MINYEAR, 1, 1)]) == msgpack.packb(
        ["0001-01-01"]
    )


def test_date_max() -> None:
    """
    datetime.date MAXYEAR
    """
    assert ormsgpack.packb([datetime.date(datetime.MAXYEAR, 12, 31)]) == msgpack.packb(
        ["9999-12-31"]
    )


def test_date_three_digits() -> None:
    """
    datetime.date three digit year
    """
    assert ormsgpack.packb(
        [datetime.date(312, 1, 1)],
    ) == msgpack.packb(["0312-01-01"])


def test_date_two_digits() -> None:
    """
    datetime.date two digit year
    """
    assert ormsgpack.packb(
        [datetime.date(46, 1, 1)],
    ) == msgpack.packb(["0046-01-01"])


def test_time() -> None:
    """
    datetime.time
    """
    assert ormsgpack.packb([datetime.time(12, 15, 59, 111)]) == msgpack.packb(
        ["12:15:59.000111"]
    )
    assert ormsgpack.packb([datetime.time(12, 15, 59)]) == msgpack.packb(["12:15:59"])


def test_time_tz() -> None:
    """
    datetime.time with tzinfo error
    """
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(
            [datetime.time(12, 15, 59, 111, tzinfo=tz.gettz("Asia/Shanghai"))]
        )


def test_time_microsecond_max() -> None:
    """
    datetime.time microsecond max
    """
    assert ormsgpack.packb(datetime.time(0, 0, 0, 999999)) == msgpack.packb(
        "00:00:00.999999"
    )


def test_time_microsecond_min() -> None:
    """
    datetime.time microsecond min
    """
    assert ormsgpack.packb(datetime.time(0, 0, 0, 1)) == msgpack.packb(
        "00:00:00.000001"
    )


def test_passthrough_datetime() -> None:
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(
            datetime.datetime(1970, 1, 1), option=ormsgpack.OPT_PASSTHROUGH_DATETIME
        )


def test_passthrough_date() -> None:
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(
            datetime.date(1970, 1, 1), option=ormsgpack.OPT_PASSTHROUGH_DATETIME
        )


def test_passthrough_time() -> None:
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(
            datetime.time(12, 0, 0), option=ormsgpack.OPT_PASSTHROUGH_DATETIME
        )


def test_passthrough_datetime_default() -> None:
    assert ormsgpack.packb(
        datetime.datetime(1970, 1, 1),
        option=ormsgpack.OPT_PASSTHROUGH_DATETIME,
        default=lambda x: x.strftime("%a, %d %b %Y %H:%M:%S GMT"),
    ) == msgpack.packb("Thu, 01 Jan 1970 00:00:00 GMT")


@pytest.mark.parametrize(("value", "serialized_value"), TIMESTAMP_EXT_PARAMS)
def test_datetime_as_timestamp_ext(
    value: datetime.datetime,
    serialized_value: bytes,
) -> None:
    assert (
        ormsgpack.packb(value, option=ormsgpack.OPT_DATETIME_AS_TIMESTAMP_EXT)
        == serialized_value
    )
    assert (
        ormsgpack.packb(
            value.astimezone(tz=zoneinfo.ZoneInfo("Asia/Shanghai")),
            option=ormsgpack.OPT_DATETIME_AS_TIMESTAMP_EXT,
        )
        == serialized_value
    )
    assert (
        ormsgpack.unpackb(
            serialized_value, option=ormsgpack.OPT_DATETIME_AS_TIMESTAMP_EXT
        )
        == value
    )


@pytest.mark.parametrize(("value", "serialized_value"), TIMESTAMP_EXT_PARAMS)
def test_datetime_as_timestamp_ext_dict_key(
    dict_type: DictType,
    value: datetime.datetime,
    serialized_value: bytes,
) -> None:
    obj = dict_type({value: True})
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(obj)
    packed = ormsgpack.packb(
        obj, option=ormsgpack.OPT_NON_STR_KEYS | ormsgpack.OPT_DATETIME_AS_TIMESTAMP_EXT
    )
    assert packed == b"\x81" + serialized_value + b"\xc3"
    with pytest.raises(ormsgpack.MsgpackDecodeError):
        ormsgpack.unpackb(packed)
    assert (
        ormsgpack.unpackb(
            packed,
            option=ormsgpack.OPT_NON_STR_KEYS | ormsgpack.OPT_DATETIME_AS_TIMESTAMP_EXT,
        )
        == obj
    )


def test_datetime_naive_as_timestamp_ext() -> None:
    assert ormsgpack.packb(
        datetime.datetime(1970, 1, 1),
        option=ormsgpack.OPT_DATETIME_AS_TIMESTAMP_EXT,
    ) == msgpack.packb("1970-01-01T00:00:00")
    assert (
        ormsgpack.packb(
            datetime.datetime(1970, 1, 1),
            option=ormsgpack.OPT_DATETIME_AS_TIMESTAMP_EXT | ormsgpack.OPT_NAIVE_UTC,
        )
        == b"\xd6\xff\x00\x00\x00\x00"
    )
