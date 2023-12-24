import dataclasses
import enum
import random
import string
import uuid
from datetime import datetime, timedelta, timezone

import numpy
from numpy.typing import NDArray
from pydantic import BaseModel


@dataclasses.dataclass
class Group:
    name: str
    score: float
    uid: uuid.UUID


class UserType(enum.Enum):
    User = 1
    Admin = 2
    System = 3


@dataclasses.dataclass
class User:
    active: bool
    created_time: datetime
    groups: list[Group]
    name: str
    score: int
    type: UserType
    uid: uuid.UUID


class GroupModel(BaseModel):
    name: str
    score: float
    uid: uuid.UUID


class UserModel(BaseModel):
    active: bool
    created_time: datetime
    groups: list[GroupModel]
    name: str
    score: int
    type: UserType
    uid: uuid.UUID


class Generator:
    def __init__(self, seed: int) -> None:
        self.rng = random.Random(seed)
        self.numpy_rng = numpy.random.default_rng(seed)
        self.alphabets = (
            string.ascii_letters,
            "".join(chr(c) for c in range(0x3040, 0x309F)),
        )
        self.min_time = datetime(1970, 1, 1, tzinfo=timezone.utc)
        self.max_time = datetime(2514, 1, 1, tzinfo=timezone.utc)

    def bool_array(self, size: int) -> NDArray[numpy.bool]:
        return self.numpy_rng.choice((True, False), size=size)

    def float_array(self, size: int) -> NDArray[numpy.float64]:
        return self.numpy_rng.random(size=size)

    def int_array(self, size: int) -> NDArray[numpy.int64]:
        return self.numpy_rng.integers(low=-(2**63), high=2**63, size=size)

    def datetime(self) -> datetime:
        high = (self.max_time - self.min_time) // timedelta(microseconds=1)
        return self.min_time + timedelta(microseconds=self.rng.randint(0, high))

    def string(self) -> str:
        return "".join(
            self.rng.choices(
                self.rng.choice(self.alphabets),
                k=self.rng.randint(1, 64),
            )
        )

    def uuid4(self) -> uuid.UUID:
        return uuid.UUID(bytes=self.rng.randbytes(16), version=4)

    def user(self) -> User:
        name = self.string()
        return User(
            active=self.rng.choice((True, False)),
            created_time=self.datetime(),
            groups=[
                Group(
                    name=self.string(),
                    score=self.rng.random(),
                    uid=self.uuid4(),
                )
                for _ in range(self.rng.randint(1, 4))
            ],
            name=name,
            score=self.rng.randrange(-(2**63), 2**63),
            type=self.rng.choice(list(UserType)),
            uid=self.uuid4(),
        )


def datasets(generator: Generator) -> dict[str, object]:
    user_dataclasses = [generator.user() for _ in range(1000)]
    user_dicts = [dataclasses.asdict(user) for user in user_dataclasses]
    user_models = [UserModel(**user) for user in user_dicts]
    bool_array = generator.bool_array(100_000)
    float_array = generator.float_array(100_000)
    int_array = generator.int_array(100_000)
    return {
        "dataclass": user_dataclasses,
        "dict": user_dicts,
        "pydantic": user_models,
        "bool": bool_array.tolist(),
        "float": float_array.tolist(),
        "int": int_array.tolist(),
        "numpy.bool": bool_array,
        "numpy.float": float_array,
        "numpy.int": int_array,
    }
