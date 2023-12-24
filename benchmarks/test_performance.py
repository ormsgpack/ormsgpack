from collections.abc import Callable
from typing import Any

import pytest

import ormsgpack

from .generator import Generator, datasets

GENERATOR = Generator(0)
DATASETS = datasets(GENERATOR)


@pytest.mark.parametrize(
    "dataset",
    [pytest.param(v, id=k) for k, v in DATASETS.items()],
)
def test_packb(benchmark: Callable[..., Any], dataset: object) -> None:
    benchmark(
        lambda: ormsgpack.packb(
            dataset,
            option=ormsgpack.OPT_SERIALIZE_NUMPY | ormsgpack.OPT_SERIALIZE_PYDANTIC,
        )
    )


@pytest.mark.parametrize(
    "dataset",
    [pytest.param(v, id=k) for k, v in DATASETS.items() if k in {"dict"}],
)
def test_unpackb(benchmark: Callable[..., Any], dataset: object) -> None:
    data = ormsgpack.packb(dataset)
    benchmark(ormsgpack.unpackb, data)
