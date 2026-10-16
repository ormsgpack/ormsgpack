ormsgpack
=========

.. toctree::
   :hidden:

   usage
   api
   types
   changelog

ormsgpack is a fast `MessagePack <https://msgpack.org/>`__ serialization library for
Python derived from `orjson <https://github.com/ijl/orjson>`__, with native support for
various Python types.

ormsgpack follows semantic versioning and supports CPython, PyPy and GraalPy.

Installation
------------

pip

.. code:: sh

   pip install ormsgpack

uv

.. code:: sh

   uv add ormsgpack

Installing from a source distribution requires
`Rust <https://www.rust-lang.org/>`__ 1.88 or newer and
`maturin <https://github.com/PyO3/maturin>`__.

License
-------

ormsgpack is licensed under both the Apache License, Version 2.0, and the MIT License.
