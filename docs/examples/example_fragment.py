import ormsgpack
ormsgpack.packb(
    {
        "allow": ormsgpack.Fragment(b"\x92\xcd\x01\xbb\xcd\x03\xe1"),
        "deny": ormsgpack.Fragment(b"\x92P\xcc\x8f"),
    }
)
ormsgpack.unpackb(_)
