# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import gc
from collections.abc import Iterator
from typing import cast

import pytest

import ormsgpack.ormsgpack as _ormsgpack  # type: ignore[import-not-found]

from .params import DICT_TYPE_PARAMS, DictType


@pytest.fixture(params=DICT_TYPE_PARAMS, ids=lambda x: x.__name__)
def dict_type(request: pytest.FixtureRequest) -> DictType:
    return cast(DictType, request.param)


@pytest.fixture
def dict_subclass(dict_type: DictType) -> DictType:
    return cast(DictType, type("SubDict", (dict_type,), {}))


def pytest_addoption(parser: pytest.Parser) -> None:
    parser.addoption(
        "--check-leaks",
        action="store_true",
        help="Check for leaked objects",
    )


def pytest_configure(config: pytest.Config) -> None:
    if config.getoption("--check-leaks"):
        if not config.pluginmanager.hasplugin("memray"):
            raise pytest.UsageError("--check-leaks requires pytest-memray")

        if not hasattr(_ormsgpack, "is_cached_key"):
            raise pytest.UsageError(
                "--check-leaks requires an extension built with cfg(check_leaks)"
            )


@pytest.hookimpl(hookwrapper=True, tryfirst=True)
def pytest_runtest_call(item: pytest.Item) -> Iterator[None]:
    if item.config.getoption("--check-leaks") and isinstance(item, pytest.Function):
        item.runtest()
        gc.collect()
        item.add_marker(
            pytest.mark.limit_leaked_objects(
                filter_fn=lambda obj: not _ormsgpack.is_cached_key(obj)
            )
        )
    yield
