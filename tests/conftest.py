# SPDX-License-Identifier: (Apache-2.0 OR MIT)

from typing import cast

import pytest

from .params import DICT_TYPE_PARAMS, DictType


@pytest.fixture(params=DICT_TYPE_PARAMS, ids=lambda x: x.__name__)
def dict_type(request: pytest.FixtureRequest) -> DictType:
    return cast(DictType, request.param)


@pytest.fixture
def dict_subclass(dict_type: DictType) -> DictType:
    return cast(DictType, type("SubDict", (dict_type,), {}))
