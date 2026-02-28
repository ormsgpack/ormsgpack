.. _api:

API Reference
=============

.. py:module:: ormsgpack

.. py:function:: packb(obj, /, default=None, option=None)

   Serializes a Python object to a binary object in MessagePack format.

   The global interpreter lock (GIL) is held for the duration of the call.

   The serialization is based on the mappings defined in section :ref:`types`.

   :param typing.Any obj: The object to serialize
   :param typing.Callable[[typing.Any], typing.Any] | None default:
      if set, a callable object for serializing objects that are not serializable.
      ``default`` is called with one argument, an object to serialize, and its return
      value is used as the serializable representation of the object. If the return
      value is not serializable, ``default`` is called recursively, up to 254 times
   :param int | None option:
      if set, one of the ``OPT_*`` integer constants or a combination of them using the
      bitwise OR operator
   :raises MsgpackEncodeError:
      if an object is not serializable
   :raises MsgpackEncodeError:
      if a :py:obj:`str` instance contains surrogate code points and
      :py:data:`OPT_REPLACE_SURROGATES` is not specified
   :raises MsgpackEncodeError:
      if a :py:obj:`dict` key is not a :py:obj:`str` instance and
      :py:data:`OPT_NON_STR_KEYS` is not specified
   :raises MsgpackEncodeError:
      if ``default`` is called recursively more than 254 times
   :raises MsgpackEncodeError:
      if an object contains a circular reference
   :raises MsgpackEncodeError:
      if a :py:attr:`datetime.datetime.tzinfo` attribute is of an unsupported type
   :rtype: bytes

.. py:function:: unpackb(obj, /, *, ext_hook=None, option=None)

   Deserializes a binary object in MessagePack format to a Python object.

   The global interpreter lock (GIL) is held for the duration of the call.

   The deserialization is based on the following mappings:

   - nil is deserialized as the :py:obj:`None` object
   - boolean objects are deserialized as :py:obj:`bool` instances
   - integer objects are deserialized as :py:obj:`int` instances
   - float objects are deserialized as :py:obj:`float` instances
   - string objects are deserialized as :py:obj:`str` instances
   - binary objects are deserialized as :py:obj:`bytes` instances
   - array objects are deserialized as :py:obj:`tuple` instances, if the object
     is a map key, and as :py:obj:`list` instances otherwise
   - map objects are deserialized as :py:obj:`dict` instances
   - timestamp extension objects are deserialized as UTC
     :py:obj:`datetime.datetime` instances, if
     :py:data:`OPT_DATETIME_AS_TIMESTAMP_EXT` is specified

   :param bytes | bytearray | memoryview obj:
      The object to deserialize
   :param typing.Callable[[int, bytes], typing.Any] | None ext_hook:
      if set, a callable object for deserializing extension types. ``ext_hook`` is
      called with two arguments, the extension type and value, and its return value is
      used as the deserialized object
   :param int | None option:
      if set, :py:data:`OPT_DATETIME_AS_TIMESTAMP_EXT`, :py:data:`OPT_NON_STR_KEYS` or
      their combination using the bitwise OR operator
   :raises MsgpackDecodeError:
      if the object is of an invalid type or is not valid MessagePack
   :raises MsgpackDecodeError:
      if a map key is not a string and :py:data:`OPT_NON_STR_KEYS` is not specified
   :rtype: Any

.. py:exception:: MsgpackEncodeError

   a subclass of :py:exc:`TypeError`

.. py:exception:: MsgpackDecodeError

   a subclass of :py:exc:`ValueError`

.. py:data:: OPT_DATETIME_AS_TIMESTAMP_EXT

   In :py:func:`packb`, serialize aware :py:obj:`datetime.datetime` instances as
   timestamp extension objects

   In :py:func:`unpackb`, deserialize timestamp extension objects to UTC
   :py:obj:`datetime.datetime` instances

.. py:data:: OPT_NAIVE_UTC

   Serialize naive :py:obj:`datetime.datetime` objects and ``numpy.datetime64`` objects
   as UTC. This has no effect on aware :py:obj:`datetime.datetime` objects.

   .. literalinclude:: examples/example_opt_naive_utc.txt

.. py:data:: OPT_NON_STR_KEYS

   In :py:func:`packb`, serialize :py:obj:`dict` keys of type
   :py:obj:`None`,
   :py:obj:`bool`,
   :py:obj:`int`,
   :py:obj:`float`,
   :py:obj:`str`,
   :py:obj:`bytes`,
   :py:obj:`datetime.date`,
   :py:obj:`datetime.time`,
   :py:obj:`datetime.datetime`,
   :py:obj:`enum.Enum`, and
   :py:obj:`uuid.UUID`.
   All options other than the passthrough ones are supported. :py:obj:`dict` keys of
   unsupported types are not handled using ``default`` and result in
   :py:exc:`MsgpackEncodeError` being raised.

   In :py:func:`unpackb`, deserialize map keys of type other than string. Be aware that
   this option is considered unsafe and disabled by default in msgpack due to the
   possibility of a Hash DoS.

   .. literalinclude:: examples/example_opt_non_str_keys.txt

   This option is not compatible with :py:data:`OPT_SORT_KEYS`.

.. py:data:: OPT_OMIT_MICROSECONDS

   Do not serialize the microsecond component of :py:obj:`datetime.datetime`,
   :py:obj:`datetime.time` and ``numpy.datetime64`` instances.

   .. literalinclude:: examples/example_opt_omit_microseconds.txt

.. py:data:: OPT_PASSTHROUGH_BIG_INT

   Enable passthrough of :py:obj:`int` instances smaller than
   -9223372036854775807 or larger than 18446744073709551615 to ``default``.

   .. literalinclude:: examples/example_opt_passthrough_big_int.txt

.. py:data:: OPT_PASSTHROUGH_DATACLASS

   Enable passthrough of dataclasses to ``default``.

.. py:data:: OPT_PASSTHROUGH_DATETIME

   Enable passthrough of :py:obj:`datetime.datetime`, :py:obj:`datetime.date`, and
   :py:obj:`datetime.time` instances to ``default``.

.. py:data:: OPT_PASSTHROUGH_ENUM

   Enable passthrough of enum members to ``default``.

.. py:data:: OPT_PASSTHROUGH_SUBCLASS

   Enable passthrough of subclasses of :py:obj:`str`, :py:obj:`int`,
   :py:obj:`dict` and :py:obj:`list` to ``default``.

.. py:data:: OPT_PASSTHROUGH_TUPLE

   Enable passthrough of :py:obj:`tuple` instances to ``default``.

.. py:data:: OPT_PASSTHROUGH_UUID

   Enable passthrough of :py:obj:`uuid.UUID` instances to ``default``.

.. py:data:: OPT_REPLACE_SURROGATES

   Serialize :py:obj:`str` instances that contain surrogate code points by replacing the
   surrogates with the ``?`` character.

.. py:data:: OPT_SERIALIZE_NUMPY

   Serialize instances of numpy types.

.. py:data:: OPT_SERIALIZE_PYDANTIC

   Serialize ``pydantic.BaseModel`` instances.

.. py:data:: OPT_SORT_KEYS

   Serialize :py:obj:`dict` keys and pydantic model fields in sorted order. The default
   is to serialize in an unspecified order.

   This can be used to ensure the order is deterministic for hashing or tests. It has a
   substantial performance penalty and is not recommended in general.

   .. literalinclude:: examples/example_opt_sort_keys.txt

   The sorting is not collation/locale-aware:

   .. literalinclude:: examples/example_opt_sort_keys_2.txt

   This option is not supported for dataclasses.

.. py:data:: OPT_UTC_Z

   Serialize a UTC timezone on :py:obj:`datetime.datetime` and ``numpy.datetime64``
   instances as ``Z`` instead of ``+00:00``.

   .. literalinclude:: examples/example_opt_utc_z.txt

.. py:class:: Ext(tag: int, data: bytes)

   A class whose instances are serialized as MessagePack extension types. The
   instantiation arguments are an integer in the range ``[0, 127]`` and a ``bytes``
   object, defining the type and value, respectively

.. py:class:: Fragment(data: bytes)

   A class whose instances are objects in MessagePack format and are serialized as is. The
   instantiation argument is a ``bytes`` object and is validated to contain a single
   object in MessagePack format.
