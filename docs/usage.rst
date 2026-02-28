Usage
=====

ormsgpack provides the :py:func:`ormsgpack.packb` and :py:func:`ormsgpack.unpackb`
functions to serialize and deserialize a Python object to and from MessagePack format,
respectively. This is an example of serializing and deserializing an object:

.. literalinclude:: examples/example.txt

The ``option`` argument of :py:func:`ormsgpack.packb` and :py:func:`ormsgpack.unpackb`
can be used to modify their behavior, by specifying one or more options. Options are
integers and can be combined using the bitwise OR operator, e.g.,
``ormsgpack.OPT_DATETIME_AS_TIMESTAMP_EXT | ormsgpack.OPT_NON_STR_KEYS``. The above
example uses the :py:data:`ormsgpack.OPT_SERIALIZE_NUMPY` option to enable the
serialization of numpy types.

By default, ormsgpack only accepts :py:obj:`dict` objects with string keys. The
:py:data:`ormsgpack.OPT_NON_STR_KEYS` option can be used to serialize and deserialize
:py:obj:`dict` objects with keys of other types:

.. literalinclude:: examples/example_opt_non_str_keys.txt

Be aware that, when using this option, a serialized map may contain elements with the
same key, as different :py:obj:`dict` keys may be serialized to the same object. In
such a case, a MessagePack deserializer will presumably keep only one element for any
given key, as is the case for :py:func:`ormsgpack.unpackb`:

.. literalinclude:: examples/example_opt_non_str_keys_2.txt

ormsgpack supports various types natively, including dataclasses and Pydantic models.
The ``default`` argument of :py:func:`ormsgpack.packb` can be used to handle objects
that are not natively serializable:

.. literalinclude:: examples/example_default.txt

ormsgpack provides the :py:class:`ormsgpack.Ext` type to serialize objects as
MessagePack extension types. The ``ext_hook`` argument of :py:func:`ormsgpack.unpackb`
can be used to deserialize extension types:

.. literalinclude:: examples/example_ext_hook.txt

If an object is not handled, ``default`` and ``ext_hook`` should raise an exception.
Otherwise, the object is serialized or deserialized as :py:obj:`None`, because of Python
implicit `call return value
<https://docs.python.org/3/reference/expressions.html#calls>`__:

.. literalinclude:: examples/example_default_2.txt

``default`` can also be used to serialize some supported types to a custom format by
enabling the corresponding passthrough options, e.g.:

.. literalinclude:: examples/example_opt_passthrough_uuid.txt

ormsgpack also provides the :py:class:`ormsgpack.Fragment` type to include objects in
MessagePack format as is in the serialization:

.. literalinclude:: examples/example_fragment.txt

This can be useful in cases where an object to serialize contains nested objects that are
provided as serialized, e.g. from a cache or an extension module, as the nested objects do
not need to be deserialized.

See the :ref:`api` and :ref:`types` sections for more details.
