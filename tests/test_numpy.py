# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import msgpack
import pytest

import ormsgpack

numpy = pytest.importorskip("numpy")


@pytest.mark.parametrize(
    "dtype",
    (
        numpy.int8,
        numpy.int16,
        numpy.int32,
        numpy.int64,
        numpy.intp,
        numpy.uint8,
        numpy.uint16,
        numpy.uint32,
        numpy.uint64,
        numpy.uintp,
    ),
)
def test_numpy_array_d1_integer(dtype: type) -> None:
    info = numpy.iinfo(dtype)
    assert ormsgpack.packb(
        numpy.array([info.min, info.max], dtype),
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb([info.min, info.max])


def test_numpy_array_d1_f16() -> None:
    array = numpy.array([1.0, 65504.0], numpy.float16)
    assert ormsgpack.packb(
        array,
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb([1.0, 65504.0], use_single_float=True)


def test_numpy_array_d1_f32() -> None:
    array = numpy.array([1.0, 3.4028235e38], numpy.float32)
    assert ormsgpack.packb(
        array,
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb([1.0, 3.4028235e38], use_single_float=True)


def test_numpy_array_d1_f64() -> None:
    assert ormsgpack.packb(
        numpy.array([1.0, 1.7976931348623157e308], numpy.float64),
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb([1.0, 1.7976931348623157e308])


def test_numpy_array_d1_bool() -> None:
    assert ormsgpack.packb(
        numpy.array([True, False, False, True]),
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb([True, False, False, True])


def test_numpy_array_d1_datetime64_years() -> None:
    assert ormsgpack.packb(
        numpy.array(
            [
                numpy.datetime64("2021"),
                numpy.datetime64("2022"),
            ],
        ),
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb(
        [
            "2021-01-01T00:00:00",
            "2022-01-01T00:00:00",
        ],
    )


def test_numpy_array_d1_datetime64_months() -> None:
    assert ormsgpack.packb(
        numpy.array(
            [
                numpy.datetime64("1969-12"),
                numpy.datetime64("2021-01"),
                numpy.datetime64("2022-01"),
            ],
        ),
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb(
        [
            "1969-12-01T00:00:00",
            "2021-01-01T00:00:00",
            "2022-01-01T00:00:00",
        ],
    )


def test_numpy_array_d1_datetime64_days() -> None:
    assert ormsgpack.packb(
        numpy.array(
            [
                numpy.datetime64("2021-01-01"),
                numpy.datetime64("2022-01-01"),
            ],
        ),
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb(
        [
            "2021-01-01T00:00:00",
            "2022-01-01T00:00:00",
        ],
    )


def test_numpy_array_d1_datetime64_hours() -> None:
    assert ormsgpack.packb(
        numpy.array(
            [
                numpy.datetime64("2021-01-01T00"),
                numpy.datetime64("2022-01-01T00"),
            ],
        ),
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb(
        [
            "2021-01-01T00:00:00",
            "2022-01-01T00:00:00",
        ],
    )


def test_numpy_array_d1_datetime64_minutes() -> None:
    assert ormsgpack.packb(
        numpy.array(
            [
                numpy.datetime64("2021-01-01T00:00"),
                numpy.datetime64("2022-01-01T00:00"),
            ],
        ),
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb(
        [
            "2021-01-01T00:00:00",
            "2022-01-01T00:00:00",
        ],
    )


def test_numpy_array_d1_datetime64_seconds() -> None:
    assert ormsgpack.packb(
        numpy.array(
            [
                numpy.datetime64("2021-01-01T00:00:00"),
                numpy.datetime64("2022-01-01T00:00:00"),
            ],
        ),
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb(
        [
            "2021-01-01T00:00:00",
            "2022-01-01T00:00:00",
        ],
    )


def test_numpy_array_d1_datetime64_milliseconds() -> None:
    assert ormsgpack.packb(
        numpy.array(
            [
                numpy.datetime64("1969-12-31T23:59:59.999"),
                numpy.datetime64("2021-01-01T00:00:00"),
                numpy.datetime64("2022-01-01T00:00:00.123"),
            ],
        ),
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb(
        [
            "1969-12-31T23:59:59.999000",
            "2021-01-01T00:00:00",
            "2022-01-01T00:00:00.123000",
        ],
    )


def test_numpy_array_d1_datetime64_microseconds() -> None:
    assert ormsgpack.packb(
        numpy.array(
            [
                numpy.datetime64("1969-12-31T23:59:59.999999"),
                numpy.datetime64("2021-01-01T00:00:00"),
                numpy.datetime64("2022-01-01T00:00:00.123456"),
            ],
        ),
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb(
        [
            "1969-12-31T23:59:59.999999",
            "2021-01-01T00:00:00",
            "2022-01-01T00:00:00.123456",
        ],
    )


def test_numpy_array_d1_datetime64_nanoseconds() -> None:
    assert ormsgpack.packb(
        numpy.array(
            [
                numpy.datetime64("1969-12-31T23:59:59.999999999"),
                numpy.datetime64("2021-01-01T00:00:00"),
                numpy.datetime64("2022-01-01T00:00:00.123456789"),
            ],
        ),
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb(
        [
            "1969-12-31T23:59:59.999999",
            "2021-01-01T00:00:00",
            "2022-01-01T00:00:00.123456",
        ],
    )


def test_numpy_array_d2_i64() -> None:
    assert ormsgpack.packb(
        numpy.array([[1, 2, 3], [4, 5, 6]], numpy.int64),
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb([[1, 2, 3], [4, 5, 6]])


def test_numpy_array_d2_f64() -> None:
    assert ormsgpack.packb(
        numpy.array([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]], numpy.float64),
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]])


def test_numpy_array_d3_i8() -> None:
    assert ormsgpack.packb(
        numpy.array([[[1, 2], [3, 4]], [[5, 6], [7, 8]]], numpy.int8),
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb([[[1, 2], [3, 4]], [[5, 6], [7, 8]]])


def test_numpy_array_d3_u8() -> None:
    assert ormsgpack.packb(
        numpy.array([[[1, 2], [3, 4]], [[5, 6], [7, 8]]], numpy.uint8),
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb([[[1, 2], [3, 4]], [[5, 6], [7, 8]]])


def test_numpy_array_d3_i32() -> None:
    assert ormsgpack.packb(
        numpy.array([[[1, 2], [3, 4]], [[5, 6], [7, 8]]], numpy.int32),
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb([[[1, 2], [3, 4]], [[5, 6], [7, 8]]])


def test_numpy_array_d3_i64() -> None:
    assert ormsgpack.packb(
        numpy.array([[[1, 2], [3, 4], [5, 6], [7, 8]]], numpy.int64),
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb([[[1, 2], [3, 4], [5, 6], [7, 8]]])


def test_numpy_array_d3_f64() -> None:
    assert ormsgpack.packb(
        numpy.array(
            [[[1.0, 2.0], [3.0, 4.0]], [[5.0, 6.0], [7.0, 8.0]]], numpy.float64
        ),
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb([[[1.0, 2.0], [3.0, 4.0]], [[5.0, 6.0], [7.0, 8.0]]])


def test_numpy_array_fortran() -> None:
    array = numpy.array([[1, 2], [3, 4]], order="F")
    assert array.flags["F_CONTIGUOUS"] is True
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(array, option=ormsgpack.OPT_SERIALIZE_NUMPY)
    assert ormsgpack.packb(
        array, default=lambda x: x.tolist(), option=ormsgpack.OPT_SERIALIZE_NUMPY
    ) == ormsgpack.packb(array.tolist())


def test_numpy_array_non_contiguous_message() -> None:
    array = numpy.array([[1, 2], [3, 4]], order="F")
    assert array.flags["F_CONTIGUOUS"] is True
    try:
        ormsgpack.packb(array, option=ormsgpack.OPT_SERIALIZE_NUMPY)
        assert False
    except TypeError as exc:
        assert (
            str(exc)
            == "numpy array is not C contiguous; use ndarray.tolist() in default"
        )


def test_numpy_array_unsupported_dtype() -> None:
    array = numpy.array([[1, 2], [3, 4]], numpy.csingle)
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(array, option=ormsgpack.OPT_SERIALIZE_NUMPY)


def test_numpy_array_d1() -> None:
    array = numpy.array([1])
    assert (
        ormsgpack.unpackb(
            ormsgpack.packb(
                array,
                option=ormsgpack.OPT_SERIALIZE_NUMPY,
            )
        )
        == array.tolist()
    )


def test_numpy_array_d2() -> None:
    array = numpy.array([[1]])
    assert (
        ormsgpack.unpackb(
            ormsgpack.packb(
                array,
                option=ormsgpack.OPT_SERIALIZE_NUMPY,
            )
        )
        == array.tolist()
    )


def test_numpy_array_d3() -> None:
    array = numpy.array([[[1]]])
    assert (
        ormsgpack.unpackb(
            ormsgpack.packb(
                array,
                option=ormsgpack.OPT_SERIALIZE_NUMPY,
            )
        )
        == array.tolist()
    )


def test_numpy_array_d4() -> None:
    array = numpy.array([[[[1]]]])
    assert (
        ormsgpack.unpackb(
            ormsgpack.packb(
                array,
                option=ormsgpack.OPT_SERIALIZE_NUMPY,
            )
        )
        == array.tolist()
    )


def test_numpy_array_4_stride() -> None:
    array = numpy.random.rand(4, 4, 4, 4)
    assert (
        ormsgpack.unpackb(
            ormsgpack.packb(
                array,
                option=ormsgpack.OPT_SERIALIZE_NUMPY,
            )
        )
        == array.tolist()
    )


def test_numpy_array_dimension_zero() -> None:
    array = numpy.array(0)
    assert array.ndim == 0
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(array, option=ormsgpack.OPT_SERIALIZE_NUMPY)

    array = numpy.empty((0, 4, 2))
    assert (
        ormsgpack.unpackb(
            ormsgpack.packb(
                array,
                option=ormsgpack.OPT_SERIALIZE_NUMPY,
            )
        )
        == array.tolist()
    )

    array = numpy.empty((4, 0, 2))
    assert (
        ormsgpack.unpackb(
            ormsgpack.packb(
                array,
                option=ormsgpack.OPT_SERIALIZE_NUMPY,
            )
        )
        == array.tolist()
    )

    array = numpy.empty((2, 4, 0))
    assert (
        ormsgpack.unpackb(
            ormsgpack.packb(
                array,
                option=ormsgpack.OPT_SERIALIZE_NUMPY,
            )
        )
        == array.tolist()
    )


def test_numpy_array_dimension_max() -> None:
    array = numpy.random.rand(
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
    )
    assert array.ndim == 32
    assert (
        ormsgpack.unpackb(
            ormsgpack.packb(
                array,
                option=ormsgpack.OPT_SERIALIZE_NUMPY,
            )
        )
        == array.tolist()
    )


@pytest.mark.parametrize(
    "dtype",
    (
        numpy.int8,
        numpy.int16,
        numpy.int32,
        numpy.int64,
        numpy.intp,
        numpy.uint8,
        numpy.uint16,
        numpy.uint32,
        numpy.uint64,
        numpy.uintp,
    ),
)
def test_numpy_scalar_integer(dtype: type) -> None:
    info = numpy.iinfo(dtype)
    assert ormsgpack.packb(
        dtype(info.min), option=ormsgpack.OPT_SERIALIZE_NUMPY
    ) == msgpack.packb(info.min)

    assert ormsgpack.packb(
        dtype(info.max), option=ormsgpack.OPT_SERIALIZE_NUMPY
    ) == msgpack.packb(info.max)


def test_numpy_scalar_float16() -> None:
    assert ormsgpack.packb(
        numpy.float16(1.0), option=ormsgpack.OPT_SERIALIZE_NUMPY
    ) == msgpack.packb(1.0, use_single_float=True)


def test_numpy_scalar_float32() -> None:
    assert ormsgpack.packb(
        numpy.float32(1.0), option=ormsgpack.OPT_SERIALIZE_NUMPY
    ) == msgpack.packb(1.0, use_single_float=True)


def test_numpy_scalar_float64() -> None:
    assert ormsgpack.packb(
        numpy.float64(123.123), option=ormsgpack.OPT_SERIALIZE_NUMPY
    ) == msgpack.packb(123.123)


def test_numpy_bool() -> None:
    data = {"a": numpy.bool_(True), "b": numpy.bool_(False)}
    assert ormsgpack.packb(
        data,
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb({"a": True, "b": False})


def test_numpy_datetime64() -> None:
    data = [
        numpy.datetime64("2021"),
        numpy.datetime64("2021-01"),
        numpy.datetime64("2021-01-01"),
        numpy.datetime64("2021-01-01T00"),
        numpy.datetime64("2021-01-01T00:00"),
        numpy.datetime64("2021-01-01T00:00:00"),
        numpy.datetime64("2021-01-01T00:00:00.123"),
        numpy.datetime64("2021-01-01T00:00:00.123456"),
        numpy.datetime64("2021-01-01T00:00:00.123456789"),
    ]
    assert ormsgpack.packb(
        data,
        option=ormsgpack.OPT_SERIALIZE_NUMPY,
    ) == msgpack.packb(
        [
            "2021-01-01T00:00:00",
            "2021-01-01T00:00:00",
            "2021-01-01T00:00:00",
            "2021-01-01T00:00:00",
            "2021-01-01T00:00:00",
            "2021-01-01T00:00:00",
            "2021-01-01T00:00:00.123000",
            "2021-01-01T00:00:00.123456",
            "2021-01-01T00:00:00.123456",
        ],
    )


def test_numpy_datetime64_naive_utc() -> None:
    data = [
        numpy.datetime64("2021"),
        numpy.datetime64("2021-01"),
        numpy.datetime64("2021-01-01"),
        numpy.datetime64("2021-01-01T00"),
        numpy.datetime64("2021-01-01T00:00"),
        numpy.datetime64("2021-01-01T00:00:00"),
        numpy.datetime64("2021-01-01T00:00:00.123"),
        numpy.datetime64("2021-01-01T00:00:00.123456"),
        numpy.datetime64("2021-01-01T00:00:00.123456789"),
    ]
    assert ormsgpack.packb(
        data,
        option=ormsgpack.OPT_SERIALIZE_NUMPY | ormsgpack.OPT_NAIVE_UTC,
    ) == msgpack.packb(
        [
            "2021-01-01T00:00:00+00:00",
            "2021-01-01T00:00:00+00:00",
            "2021-01-01T00:00:00+00:00",
            "2021-01-01T00:00:00+00:00",
            "2021-01-01T00:00:00+00:00",
            "2021-01-01T00:00:00+00:00",
            "2021-01-01T00:00:00.123000+00:00",
            "2021-01-01T00:00:00.123456+00:00",
            "2021-01-01T00:00:00.123456+00:00",
        ],
    )


def test_numpy_datetime64_utc_z() -> None:
    data = [
        numpy.datetime64("2021"),
        numpy.datetime64("2021-01"),
        numpy.datetime64("2021-01-01"),
        numpy.datetime64("2021-01-01T00"),
        numpy.datetime64("2021-01-01T00:00"),
        numpy.datetime64("2021-01-01T00:00:00"),
        numpy.datetime64("2021-01-01T00:00:00.123"),
        numpy.datetime64("2021-01-01T00:00:00.123456"),
        numpy.datetime64("2021-01-01T00:00:00.123456789"),
    ]
    assert ormsgpack.packb(
        data,
        option=ormsgpack.OPT_SERIALIZE_NUMPY
        | ormsgpack.OPT_NAIVE_UTC
        | ormsgpack.OPT_UTC_Z,
    ) == msgpack.packb(
        [
            "2021-01-01T00:00:00Z",
            "2021-01-01T00:00:00Z",
            "2021-01-01T00:00:00Z",
            "2021-01-01T00:00:00Z",
            "2021-01-01T00:00:00Z",
            "2021-01-01T00:00:00Z",
            "2021-01-01T00:00:00.123000Z",
            "2021-01-01T00:00:00.123456Z",
            "2021-01-01T00:00:00.123456Z",
        ],
    )


def test_numpy_datetime64_omit_microseconds() -> None:
    data = [
        numpy.datetime64("2021"),
        numpy.datetime64("2021-01"),
        numpy.datetime64("2021-01-01"),
        numpy.datetime64("2021-01-01T00"),
        numpy.datetime64("2021-01-01T00:00"),
        numpy.datetime64("2021-01-01T00:00:00"),
        numpy.datetime64("2021-01-01T00:00:00.123"),
        numpy.datetime64("2021-01-01T00:00:00.123456"),
        numpy.datetime64("2021-01-01T00:00:00.123456789"),
    ]
    assert ormsgpack.packb(
        data,
        option=ormsgpack.OPT_SERIALIZE_NUMPY | ormsgpack.OPT_OMIT_MICROSECONDS,
    ) == msgpack.packb(
        [
            "2021-01-01T00:00:00",
            "2021-01-01T00:00:00",
            "2021-01-01T00:00:00",
            "2021-01-01T00:00:00",
            "2021-01-01T00:00:00",
            "2021-01-01T00:00:00",
            "2021-01-01T00:00:00",
            "2021-01-01T00:00:00",
            "2021-01-01T00:00:00",
        ],
    )


def test_numpy_datetime64_unsupported_unit() -> None:
    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(
            numpy.datetime64("2022-01-01T00:00:00.123456789123"),
            option=ormsgpack.OPT_SERIALIZE_NUMPY,
        )

    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(
            numpy.datetime64("NaT"),
            option=ormsgpack.OPT_SERIALIZE_NUMPY,
        )


@pytest.mark.parametrize("unit", ("Y", "M", "W", "D", "h", "m", "s", "ms", "us"))
def test_numpy_datetime64_unrepresentable(unit: str) -> None:
    value = numpy.datetime64(numpy.iinfo(numpy.int64).max, unit)

    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(value, option=ormsgpack.OPT_SERIALIZE_NUMPY)

    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(numpy.array([value]), option=ormsgpack.OPT_SERIALIZE_NUMPY)


@pytest.mark.parametrize("unit", ("Y", "M", "W", "D", "h", "m", "s", "ms", "us", "ns"))
def test_numpy_datetime64_nat(unit: str) -> None:
    value = numpy.datetime64("NaT", unit)

    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(value, option=ormsgpack.OPT_SERIALIZE_NUMPY)

    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(numpy.array([value]), option=ormsgpack.OPT_SERIALIZE_NUMPY)
