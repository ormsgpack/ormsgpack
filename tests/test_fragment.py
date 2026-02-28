import pytest

import ormsgpack

from .params import ARRAY_PARAMS, BIN_PARAMS, INT_PARAMS, MAP_PARAMS, STR_PARAMS


@pytest.mark.parametrize(
    "obj",
    (
        pytest.param(None, id="nil"),
        pytest.param(True, id="true"),
        pytest.param(False, id="false"),
        *INT_PARAMS,
        pytest.param(1.0, id="float"),
        *STR_PARAMS,
        *BIN_PARAMS,
        *ARRAY_PARAMS,
        *MAP_PARAMS,
    ),
)
def test_fragment(obj: object) -> None:
    packed = ormsgpack.packb(obj)
    fragment = ormsgpack.Fragment(packed)
    assert ormsgpack.packb(fragment) == packed
    assert ormsgpack.packb([fragment]) == ormsgpack.packb([obj])
    assert ormsgpack.packb({"f": fragment}) == ormsgpack.packb({"f": obj})

    with pytest.raises(ValueError):
        ormsgpack.Fragment(packed + packed)
