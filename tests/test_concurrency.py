# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import concurrent.futures


def test_concurrency() -> None:
    def run(obj: object) -> object:
        import ormsgpack

        return ormsgpack.unpackb(ormsgpack.packb(obj))

    obj = {str(i): i for i in range(1024)}
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as executor:
        futures = [executor.submit(run, obj) for _ in range(256)]
        for future in concurrent.futures.as_completed(futures):
            assert future.result() == obj
