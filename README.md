# ormsgpack

![PyPI - Version](https://img.shields.io/pypi/v/ormsgpack)
![PyPI - Python Version](https://img.shields.io/pypi/pyversions/ormsgpack)
![PyPI - Downloads](https://img.shields.io/pypi/dm/ormsgpack)

ormsgpack is a fast [MessagePack](https://msgpack.org/) serialization library for Python
derived from [orjson](https://github.com/ijl/orjson), with native support for various
Python types.

ormsgpack follows semantic versioning and supports CPython, PyPy and GraalPy.

Links:

- [Repository](https://github.com/ormsgpack/ormsgpack)
- [Documentation](https://ormsgpack.readthedocs.io/)

## Installation

pip

```sh
pip install ormsgpack
```

uv

```sh
uv add ormsgpack
```

Installing from a source distribution requires
[Rust](https://www.rust-lang.org/) 1.81 or newer and
[maturin](https://github.com/PyO3/maturin).

## Quickstart

This is an example of serializing and deserializing an object:

```python
>>> import ormsgpack, datetime, numpy
>>> event = {
...     "type": "put",
...     "time": datetime.datetime(1970, 1, 1),
...     "uid": 1,
...     "data": numpy.array([1, 2]),
... }
>>> ormsgpack.packb(event, option=ormsgpack.OPT_SERIALIZE_NUMPY)
b'\x84\xa4type\xa3put\xa4time\xb31970-01-01T00:00:00\xa3uid\x01\xa4data\x92\x01\x02'
>>> ormsgpack.unpackb(_)
{'type': 'put', 'time': '1970-01-01T00:00:00', 'uid': 1, 'data': [1, 2]}
```

## License

orjson was written by ijl <<ijl@mailbox.org>>, copyright 2018 - 2021, licensed
under both the Apache License, Version 2.0, and the MIT License.

ormsgpack was forked from orjson by Aviram Hassan and is now maintained by Emanuele Giaquinta (@exg), licensed
under both the Apache License, Version 2.0, and the MIT License.
