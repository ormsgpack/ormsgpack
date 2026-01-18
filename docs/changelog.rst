Changelog
=========

1.12.2 - 2026-01-18
-------------------

Changed
~~~~~~~

- Add Python 3.14 free-threaded Windows x86_64 wheel
- Improve serialization performance
- Update dependencies

Fixed
~~~~~

- Fix ``unpackb`` crash on Windows with Python 3.14 free-threaded and
  on Linux s390x (:issue:`499`)

1.12.1 - 2025-12-14
-------------------

Changed
~~~~~~~

- Revamp the documentation. The documentation is now generated with
  Sphinx and is available at https://ormsgpack.readthedocs.io

Fixed
~~~~~

- Fix serialization of pydantic models with extra fields (:issue:`472`)

1.12.0 - 2025-11-04
-------------------

Changed
~~~~~~~

- Drop support for Python 3.9
- Add ``packb`` option ``OPT_REPLACE_SURROGATES`` to serialize strings
  that contain surrogate code points

Fixed
~~~~~

- Serialize mutable objects inside a critical section in free threading

1.11.0 - 2025-10-08
-------------------

Changed
~~~~~~~

- Add support for Python 3.14
- Add support for free-threading and subinterpreters
- Add Windows arm64 wheels (:pr:`412` by :user:`JexinSam`)

1.10.0 - 2025-05-24
-------------------

Changed
~~~~~~~

- Port to PyPy 3.11 and GraalPy 3.11
- Add support for ``bytearray`` and ``memoryview`` types
  (:pr:`374` by :user:`littledivy`)
- Add ``packb`` and ``unpackb`` option ``OPT_DATETIME_AS_TIMESTAMP_EXT``
  to serialize aware datetime objects to timestamp extension objects and
  deserialize timestamp extension objects to UTC datetime objects,
  respectively (:issue:`378`)

1.9.1 - 2025-03-28
------------------

Changed
~~~~~~~

- Add musllinux wheels (:issue:`366`)

1.9.0 - 2025-03-23
------------------

Changed
~~~~~~~

- Add ``packb`` option ``OPT_PASSTHROUGH_ENUM`` to enable passthrough of
  Enum objects (:pr:`361` by :user:`hinthornw`)
- Add PyInstaller hook (:issue:`354`)
- Update dependencies

1.8.0 - 2025-02-22
------------------

Fixed
~~~~~

- ``packb`` now rejects dictionary keys with nested dataclasses or
  pydantic models

Changed
~~~~~~~

- Add ``packb`` option ``OPT_PASSTHROUGH_UUID`` to enable passthrough of
  UUID objects (:issue:`338`)
- Update dependencies

1.7.0 - 2024-11-29
------------------

Fixed
~~~~~

- Detect Pydantic 2.10 models (:issue:`311`)
- Fix serialization of dataclasses without ``__slots__`` and with a
  field defined with a descriptor object as default value, a field
  defined with ``init=False`` and a default value, or a cached property

Changed
~~~~~~~

- Drop support for Python 3.8
- Support ``OPT_SORT_KEYS`` also for Pydantic models (:issue:`312`)
- Improve deserialization performance

1.6.0 - 2024-10-18
------------------

Fixed
~~~~~

- Deduplicate map keys also when ``OPT_NON_STR_KEYS`` is set (:issue:`279`)
- Add missing type information for Ext type (:pr:`285` by :user:`trim21`)
- Fix type annotation of unpackb first argument

Changed
~~~~~~~

- Add support for python 3.13
- Improve test coverage

1.5.0 - 2024-04-19
------------------

Changed
~~~~~~~

- Add support for numpy datetime64 and float16 types
- Optimize serialization of dataclasses
- Optimize deserialization of arrays and maps

1.4.2 - 2024-01-28
------------------

Fixed
~~~~~

- Fix crash on termination with Python 3.11 (:issue:`223`)

Changed
~~~~~~~

- Add Linux aarch64 and armv7 wheels (:issue:`100`, :issue:`207`)

1.4.1 - 2023-11-12
------------------

Fixed
~~~~~

- Fix performance regression in dict serialization introduced in 1.3.0

1.4.0 - 2023-11-05
------------------

Fixed
~~~~~

- Fix crash in non optimized builds

Changed
~~~~~~~

- Add support for MessagePack Extension type
- Add support for numpy 16-bit integers

1.3.0 - 2023-10-04
------------------

Changed
~~~~~~~

- Drop support for Python 3.7
- Add support for Python 3.12
- Add support for Pydantic 2
- Add ``packb`` option ``OPT_SORT_KEYS`` to serialize dictionaries
  sorted by key
- Update dependencies

1.2.6 - 2023-04-24
------------------

Fixed
~~~~~

- ``once_cell`` poisoning on parallel initialization
  (:pr:`153` by :user:`Quitlox`)

1.2.5 - 2023-02-02
------------------

Fixed
~~~~~

- aarch64 build on macOS. Took ``src/serialize/writer.rs`` from upstream
  orjson. by @ijl
- Fix release on aarch64 to match orjson’s upstream.

Misc
~~~~

- update dependencies

1.2.4 - 2022-11-16
------------------

Misc
~~~~

- Fix CI (upgrade maturin, warnings, etc.)

1.2.3 - 2022-06-26
------------------

Misc
~~~~

- Updated dependencies. (:pr:`101` partially by :user:`tilman19`)
- Handle clippy warnings.

1.2.2 - 2022-04-19
------------------

Misc
~~~~

- Update dependencies

1.2.1 - 2022-03-01
------------------

Misc
~~~~

- Release 3.10 wheels
- Update dependencies

1.2.0 - 2022-02-14
------------------

Changed
~~~~~~~

- Extended int passthrough to support u64. (:pr:`77` by :user:`pejter`)

Misc
~~~~

- Updated README to include new options. (:pr:`70` by :user:`ThomasTNO`)
- Updated dependencies
- Renamed in ``numpy.rs`` ``from_parent`` to ``to_children`` to fix new
  lint rules

1.1.0 - 2022-01-08
------------------

Added
~~~~~

- Add optional passthrough for tuples. (:pr:`64` by :user:`TariqTNO`)
- Add optional passthrough for ints, that do not fit into an i64.
  (:pr:`64` by :user:`TariqTNO`)

Changed
~~~~~~~

- ``opt`` parameter can be ``None``.

Misc
~~~~

- Updated dependencies.
- Dropped 3.6 CI/CD.
- Added macOS universal builds (M1)

1.0.3 - 2021-12-18
------------------

Misc
~~~~

- Update dependencies

1.0.2 - 2021-10-26
------------------

Misc
~~~~

- Update dependencies

1.0.1 - 2021-10-13
------------------

Fixed
~~~~~

- Decrement refcount for numpy ``PyArrayInterface``. by
  `@ilj <https://github.com/ijl/orjson/commit/4c312a82f5215ff71eed5bd09d28fa004999299b>`__.
- Fix serialization of dataclass inheriting from ``abc.ABC`` and using
  ``__slots__``. by
  `@ilj <https://github.com/ijl/orjson/commit/4c312a82f5215ff71eed5bd09d28fa004999299b>`__

Changed
~~~~~~~

- Updated dependencies.
- ``find_str_kind`` test for 4-byte before latin1. by
  `@ilj <https://github.com/ijl/orjson/commit/05860e1a2ea3e8f90823d6a59e5fc9929a8692b5>`__

1.0.0 - 2021-08-31
------------------

Changed
~~~~~~~

- Aligned to orjson’s flags and features of SIMD. Didn’t include the
  stable compilation part as seems unnecessary.

Misc
~~~~

- Bumped serde, pyo3.
- Fixed pyproject.toml to work with newest maturin version.

0.3.6 - 2021-08-24
------------------

Misc
~~~~

- Update dependencies.

0.3.5 - 2021-08-05
------------------

Fixed
~~~~~

- Fixed clippy warnings for semicolon in macro.

Misc
~~~~

- Bumped serde.rs

0.3.4 - 2021-07-27
------------------

Fixed
~~~~~

- Fixed ``ormsgpack.pyi`` support of str as input for ``unpackb``.

Misc
~~~~

- Fixed Windows CI/CD.

0.3.3 - 2021-07-23
------------------

Misc
~~~~

- Refactored adding objects to the module, creating a ``__all__`` object
  similar to the way PyO3 creates. This solves an issue with upgrading
  to new maturin version.
- Changed < Py3.7 implementation to use automatic range inclusion.
- Added test to validate correct Python method flags are used on
  declare.
- Changed to use PyO3 configurations instead of our own.
  (:pr:`25` by :user:`pejter`)

0.3.2 - 2021-07-13
------------------

Fixed
~~~~~

- Fix memory leak serializing ``datetime.datetime`` with ``tzinfo``.
  (Copied from orjson)

Changed
~~~~~~~

- Update dependencies, PyO3 -> 0.14.1.

Misc
~~~~

- Setup dependabot.

0.3.1 - 2021-06-25
------------------

Changed
~~~~~~~

- ``packb`` of maps and sequences is now almost 2x faster as it
  leverages known size. (:pr:`18` by :user:`ijl`)

Misc
~~~~

- Added ``scripts/bench_target.py`` and ``scripts/profile.sh`` for
  easily benchmarking and profiling. Works only on Linux.
  (:pr:`17` by :user:`ijl`)

0.3.0 - 2021-06-13
------------------

Added
~~~~~

- ``unpackb`` now accepts keyword argument ``option`` with argument
  ``OPT_NON_STR_KEYS``. This option will let ormsgpack unpack
  dictionaries with non-str keys. Be aware that this option is
  considered unsafe and disabled by default in msgpack due to
  possibility of HashDoS.
- ``packb`` now is able to pack dictionaries with tuples as keys.
  ``unpackb`` is able to unpack such dictionaries. Both requires
  ``OPT_NON_STR_KEYS``.

Misc
~~~~

- Grouped benchmarks in a pattern that should make more sense.
- Added pydantic docs to ``README.md``
- Added graphs and benchmark results.

0.2.1 - 2021-06-12
------------------

Fixed
~~~~~

- Depth limit is now enforced for ``ormsgpack.unpackb`` - function
  should be safe for use now.

Removed
~~~~~~~

- Removed ``OPT_SERIALIZE_UUID`` from ormsgpack.pyi as it doesn’t exist.

Misc
~~~~

- Added ``scripts/test.sh`` for running tests.
- Added benchmarks, modified scripts to match new layout.

0.2.0 - 2021-06-10
------------------

Added
~~~~~

- Add support for serializing pydantic’s ``BaseModel`` instances using
  ``ormsgpack.OPT_SERIALIZE_PYDANTIC``.

Fixed
~~~~~

- ``ormsgpack.packb`` with ``option`` argument as
  ``ormsgpack.OPT_NON_STR_KEYS`` serializes bytes key into tuple of
  integers instead of using bin type. This also resulted in asymmetrical
  packb/unpackb.

Misc
~~~~

- Added ``--no-index`` to ``pip install ormsgpack`` to avoid installing
  from PyPI on CI.

0.1.0 - 2021-06-09
------------------

First version, changed orjson to ormsgpack.
