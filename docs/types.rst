.. _types:

Types
=====

none
----

The :py:obj:`None` object is serialized as nil.

bool
----

:py:obj:`bool` instances are serialized as booleans.

int
---

Instances of :py:obj:`int` and of subclasses of :py:obj:`int` are serialized as
integers. The minimum and maximum representable values are
-9223372036854775808 and 18446744073709551615, respectively.

float
-----

:py:obj:`float` instances are serialized as IEEE 754 double precision floating point
numbers.

str
---

Instances of :py:obj:`str` and of subclasses of :py:obj:`str` are serialized as strings.

bytes
-----

:py:obj:`bytes`, :py:obj:`bytearray` and :py:obj:`memoryview` instances are serialized
as binary objects.

list
----

Instances of :py:obj:`list` and of subclasses of :py:obj:`list` are serialized as
arrays.

tuple
-----

:py:obj:`tuple` instances are serialized as arrays.

dict
----

Instances of :py:obj:`dict` and of subclasses of :py:obj:`dict` are serialized as maps.

dataclass
---------

:py:mod:`dataclasses` are serialized as maps. The fields are serialized in the order they are
defined in the class. All variants of dataclasses are supported, including dataclasses
with :py:data:`object.__slots__`, frozen dataclasses and dataclasses with
descriptor-typed fields.

.. literalinclude:: examples/example_dataclass.txt

date
----

:py:obj:`datetime.date` instances are serialized as `RFC 3339
<https://tools.ietf.org/html/rfc3339>`__ strings.

.. literalinclude:: examples/example_date.txt

time
----

Naive :py:obj:`datetime.time` instances are serialized as `RFC 3339
<https://tools.ietf.org/html/rfc3339>`__ strings. Aware :py:obj:`datetime.time`
instances are not supported.

.. literalinclude:: examples/example_time.txt

datetime
--------

Naive :py:obj:`datetime.datetime` instances are serialized as `RFC 3339
<https://tools.ietf.org/html/rfc3339>`__ strings. Aware :py:obj:`datetime.datetime`
instances are serialized as `RFC 3339 <https://tools.ietf.org/html/rfc3339>`__ strings
or alternatively as MessagePack timestamp extension objects, by using the
:py:data:`ormsgpack.OPT_DATETIME_AS_TIMESTAMP_EXT` option.

.. literalinclude:: examples/example_datetime.txt

Errors with :py:attr:`datetime.datetime.tzinfo` result in
:py:exc:`ormsgpack.MsgpackEncodeError` being raised.

The serialization can be customized using the
:py:data:`ormsgpack.OPT_NAIVE_UTC`,
:py:data:`ormsgpack.OPT_OMIT_MICROSECONDS`, and
:py:data:`ormsgpack.OPT_UTC_Z` options.

enum
----

Enum members are serialized as their values. Options apply to their
values. All subclasses of :py:obj:`enum.Enum` are supported.

.. literalinclude:: examples/example_enum.txt

uuid
----

:py:obj:`uuid.UUID` instances are serialized as `RFC
4122 <https://tools.ietf.org/html/rfc4122>`__ strings.

.. literalinclude:: examples/example_uuid.txt

numpy
-----

``numpy.bool``, ``numpy.float16``, ``numpy.float32``, ``numpy.float64``,
``numpy.int8``, ``numpy.int16``, ``numpy.int32``, ``numpy.int64``,
``numpy.intp``, ``numpy.uint8``, ``numpy.uint16``, ``numpy.uint32``,
``numpy.uint64``, ``numpy.uintp`` instances are serialized as the
corresponding builtin types.

``numpy.datetime64`` instances are serialized as `RFC 3339
<https://tools.ietf.org/html/rfc3339>`__ strings.
The serialization can be customized using the
:py:data:`ormsgpack.OPT_NAIVE_UTC`,
:py:data:`ormsgpack.OPT_OMIT_MICROSECONDS`, and
:py:data:`ormsgpack.OPT_UTC_Z` options.

``numpy.ndarray`` instances are serialized as arrays. The array must be
a C-contiguous array (``C_CONTIGUOUS``) and of a supported data type.
Unsupported arrays can be serialized using ``default``, by converting
the array to a list with the ``numpy.ndarray.tolist()`` method.

The serialization of numpy types is disabled by default and can be
enabled by using the :py:data:`ormsgpack.OPT_SERIALIZE_NUMPY` option.

.. literalinclude:: examples/example_numpy.txt

pydantic
--------

``pydantic.BaseModel`` instances are serialized as maps based on the runtime
types of their field values, as with ``model.model_dump(serialize_as_any=True)``
in Pydantic V2 or ``model.dict()`` in Pydantic V1. Field aliases, exclusions and
serializers are not applied.

The serialization of pydantic models is disabled by default and can be
enabled by using the :py:data:`ormsgpack.OPT_SERIALIZE_PYDANTIC` option.

.. literalinclude:: examples/example_pydantic.txt
