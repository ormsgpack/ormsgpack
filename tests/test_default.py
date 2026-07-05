# SPDX-License-Identifier: (Apache-2.0 OR MIT)

from decimal import Decimal

import msgpack
import pytest

import ormsgpack


def test_default_not_callable() -> None:
    """
    packb() default not callable
    """
    with pytest.raises(ormsgpack.MsgpackEncodeError) as exc_info:
        ormsgpack.packb(object(), default=True)  # type: ignore[arg-type]
    assert str(exc_info.value) == "Type is not msgpack serializable: object"


def test_default_function() -> None:
    """
    packb() default function
    """
    ref = {1, 2}

    def default(obj: object) -> object:
        return str(obj)

    assert ormsgpack.packb(ref, default=default) == msgpack.packb(str(ref))


def test_default_raises_exception() -> None:
    """
    packb() default function raises exception
    """

    def default(obj: object) -> object:
        raise NotImplementedError

    with pytest.raises(ormsgpack.MsgpackEncodeError) as exc_info:
        ormsgpack.packb(object(), default=default)
    assert str(exc_info.value) == "Type is not msgpack serializable: object"


def test_default_returns_invalid_string() -> None:
    """
    packb() default function returns invalid string
    """
    ref = object()

    def default(obj: object) -> object:
        return "\ud800"

    with pytest.raises(ormsgpack.MsgpackEncodeError):
        ormsgpack.packb(ref, default=default)


def test_default_lambda() -> None:
    """
    packb() default lambda
    """
    ref = {1, 2}
    assert ormsgpack.packb(ref, default=lambda x: str(x)) == msgpack.packb(str(ref))


def test_default_callable() -> None:
    """
    packb() default callable
    """
    ref = {1, 2}

    class Default:
        def __call__(self, obj: object) -> object:
            return str(obj)

    assert ormsgpack.packb(ref, default=Default()) == msgpack.packb(str(ref))


def test_default_recursion() -> None:
    """
    packb() default recursion limit
    """

    assert ormsgpack.packb(
        Decimal(254),
        default=lambda x: x - 1 if x > 0 else 0,
    ) == msgpack.packb(0)


def test_default_recursion_reset() -> None:
    """
    packb() default recursion limit reset
    """
    assert ormsgpack.packb(
        [Decimal(254), {"a": "b"}, Decimal(254), Decimal(254)],
        default=lambda x: x - 1 if x > 0 else 0,
    ) == msgpack.packb([0, {"a": "b"}, 0, 0])


def test_default_recursion_infinite() -> None:
    """
    packb() default infinite recursion
    """
    ref = object()

    def default(obj: object) -> object:
        return obj

    with pytest.raises(ormsgpack.MsgpackEncodeError) as exc_info:
        ormsgpack.packb(ref, default=default)
    assert str(exc_info.value) == "Recursion limit for default hook reached"
