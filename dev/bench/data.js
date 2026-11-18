window.BENCHMARK_DATA = {
  "lastUpdate": 1794997464448,
  "repoUrl": "https://github.com/gxnda/rudis",
  "entries": {
    "Redis Clone Performance": [
      {
        "commit": {
          "author": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "committer": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "distinct": true,
          "id": "3485004091a5fed3e84cbd7c256154bd7008145c",
          "message": "perf: added multi-threading support to redis-benchmark in benchmark.yml",
          "timestamp": "2026-11-18T10:19:50Z",
          "tree_id": "3f50690fa1a76ab317b9785d9ec77e41884d353b",
          "url": "https://github.com/gxnda/rudis/commit/3485004091a5fed3e84cbd7c256154bd7008145c"
        },
        "date": 1795003557613,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE ratio",
            "value": 137.52,
            "unit": "%"
          },
          {
            "name": "PING_MBULK ratio",
            "value": 126.71,
            "unit": "%"
          },
          {
            "name": "SET ratio",
            "value": 143.67,
            "unit": "%"
          },
          {
            "name": "GET ratio",
            "value": 137.46,
            "unit": "%"
          },
          {
            "name": "INCR ratio",
            "value": 143.64,
            "unit": "%"
          },
          {
            "name": "LPUSH ratio",
            "value": 131.22,
            "unit": "%"
          },
          {
            "name": "RPUSH ratio",
            "value": 124.91,
            "unit": "%"
          },
          {
            "name": "LPOP ratio",
            "value": 137.5,
            "unit": "%"
          },
          {
            "name": "RPOP ratio",
            "value": 131.25,
            "unit": "%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "committer": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "distinct": true,
          "id": "af292ee3415e581859859c8916495c3a79fba520",
          "message": "perf: used build in serde serialisation + optimised/fixed incr in storage engine",
          "timestamp": "2026-09-11T12:51:20+01:00",
          "tree_id": "3c856bfc0726bf73c6182d7174a6e2162b38d5e4",
          "url": "https://github.com/gxnda/rudis/commit/af292ee3415e581859859c8916495c3a79fba520"
        },
        "date": 1795003668154,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE ratio",
            "value": 137.36,
            "unit": "%"
          },
          {
            "name": "PING_MBULK ratio",
            "value": 133.4,
            "unit": "%"
          },
          {
            "name": "SET ratio",
            "value": 137.5,
            "unit": "%"
          },
          {
            "name": "GET ratio",
            "value": 146.76,
            "unit": "%"
          },
          {
            "name": "INCR ratio",
            "value": 137.5,
            "unit": "%"
          },
          {
            "name": "LPUSH ratio",
            "value": 123.44,
            "unit": "%"
          },
          {
            "name": "RPUSH ratio",
            "value": 140.11,
            "unit": "%"
          },
          {
            "name": "LPOP ratio",
            "value": 146.7,
            "unit": "%"
          },
          {
            "name": "RPOP ratio",
            "value": 140.07,
            "unit": "%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "committer": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "distinct": true,
          "id": "cce3bc6812b87763f3b42c6869b30c4e5f1eb53f",
          "message": "perf: optimised digits()",
          "timestamp": "2026-09-11T12:24:17+01:00",
          "tree_id": "25c19d1e74d604324ac6a691836ca6f4bb21bd43",
          "url": "https://github.com/gxnda/rudis/commit/cce3bc6812b87763f3b42c6869b30c4e5f1eb53f"
        },
        "date": 1795003780530,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE ratio",
            "value": 159.99,
            "unit": "%"
          },
          {
            "name": "PING_MBULK ratio",
            "value": 111.73,
            "unit": "%"
          },
          {
            "name": "SET ratio",
            "value": 144.4,
            "unit": "%"
          },
          {
            "name": "GET ratio",
            "value": 143.67,
            "unit": "%"
          },
          {
            "name": "INCR ratio",
            "value": 137.42,
            "unit": "%"
          },
          {
            "name": "LPUSH ratio",
            "value": 124.98,
            "unit": "%"
          },
          {
            "name": "RPUSH ratio",
            "value": 131.24,
            "unit": "%"
          },
          {
            "name": "LPOP ratio",
            "value": 131.15,
            "unit": "%"
          },
          {
            "name": "RPOP ratio",
            "value": 143.76,
            "unit": "%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "committer": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "distinct": true,
          "id": "d94497ad9f09ceb2979ca9ef404dc5d04582bcea",
          "message": "Merge branch 'master' of github.com:gxnda/rudis",
          "timestamp": "2026-09-11T02:35:28+01:00",
          "tree_id": "b4e26f688fdfa843ea8aa9af5d6790f29b3aced9",
          "url": "https://github.com/gxnda/rudis/commit/d94497ad9f09ceb2979ca9ef404dc5d04582bcea"
        },
        "date": 1795003905795,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE ratio",
            "value": 129.4,
            "unit": "%"
          },
          {
            "name": "PING_MBULK ratio",
            "value": 129.39,
            "unit": "%"
          },
          {
            "name": "SET ratio",
            "value": 136.87,
            "unit": "%"
          },
          {
            "name": "GET ratio",
            "value": 138.88,
            "unit": "%"
          },
          {
            "name": "INCR ratio",
            "value": 134.89,
            "unit": "%"
          },
          {
            "name": "LPUSH ratio",
            "value": 130.0,
            "unit": "%"
          },
          {
            "name": "RPUSH ratio",
            "value": 119.9,
            "unit": "%"
          },
          {
            "name": "LPOP ratio",
            "value": 131.55,
            "unit": "%"
          },
          {
            "name": "RPOP ratio",
            "value": 131.55,
            "unit": "%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "committer": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "distinct": true,
          "id": "c8da7e8e4493a3b2b54d922c4a25b37f415b72f3",
          "message": "added itoa to Incr",
          "timestamp": "2026-09-11T02:35:25+01:00",
          "tree_id": "e7a9d730b721cf9c3b89baf9385c340f84b0c3bf",
          "url": "https://github.com/gxnda/rudis/commit/c8da7e8e4493a3b2b54d922c4a25b37f415b72f3"
        },
        "date": 1795004020925,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE ratio",
            "value": 166.71,
            "unit": "%"
          },
          {
            "name": "PING_MBULK ratio",
            "value": 146.71,
            "unit": "%"
          },
          {
            "name": "SET ratio",
            "value": 143.71,
            "unit": "%"
          },
          {
            "name": "GET ratio",
            "value": 131.28,
            "unit": "%"
          },
          {
            "name": "INCR ratio",
            "value": 143.74,
            "unit": "%"
          },
          {
            "name": "LPUSH ratio",
            "value": 131.25,
            "unit": "%"
          },
          {
            "name": "RPUSH ratio",
            "value": 124.98,
            "unit": "%"
          },
          {
            "name": "LPOP ratio",
            "value": 131.12,
            "unit": "%"
          },
          {
            "name": "RPOP ratio",
            "value": 131.25,
            "unit": "%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "committer": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "distinct": true,
          "id": "6c746aa31db9c1eec2d55be56b340b237d6d6fec",
          "message": "perf???: added set_nodelay, apparently should speed up small payloads",
          "timestamp": "2026-09-11T02:21:34+01:00",
          "tree_id": "674371eda69d6c61bae3a8e49ab621ca0d11efcc",
          "url": "https://github.com/gxnda/rudis/commit/6c746aa31db9c1eec2d55be56b340b237d6d6fec"
        },
        "date": 1795004139607,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE ratio",
            "value": 149.98,
            "unit": "%"
          },
          {
            "name": "PING_MBULK ratio",
            "value": 143.67,
            "unit": "%"
          },
          {
            "name": "SET ratio",
            "value": 147.0,
            "unit": "%"
          },
          {
            "name": "GET ratio",
            "value": 149.95,
            "unit": "%"
          },
          {
            "name": "INCR ratio",
            "value": 158.79,
            "unit": "%"
          },
          {
            "name": "LPUSH ratio",
            "value": 133.24,
            "unit": "%"
          },
          {
            "name": "RPUSH ratio",
            "value": 141.16,
            "unit": "%"
          },
          {
            "name": "LPOP ratio",
            "value": 135.16,
            "unit": "%"
          },
          {
            "name": "RPOP ratio",
            "value": 143.71,
            "unit": "%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "committer": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "distinct": true,
          "id": "44995cf4b73bb78b5ff4763dd00c4e18ba0acab0",
          "message": "lint: cargo clippy --fix",
          "timestamp": "2026-09-10T19:28:32+01:00",
          "tree_id": "d8dd107b0e37f1f7c3300ece59ecba5f8d1a4830",
          "url": "https://github.com/gxnda/rudis/commit/44995cf4b73bb78b5ff4763dd00c4e18ba0acab0"
        },
        "date": 1795004260301,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE ratio",
            "value": 135.25,
            "unit": "%"
          },
          {
            "name": "PING_MBULK ratio",
            "value": 123.51,
            "unit": "%"
          },
          {
            "name": "SET ratio",
            "value": 131.52,
            "unit": "%"
          },
          {
            "name": "GET ratio",
            "value": 127.86,
            "unit": "%"
          },
          {
            "name": "INCR ratio",
            "value": 131.46,
            "unit": "%"
          },
          {
            "name": "LPUSH ratio",
            "value": 133.3,
            "unit": "%"
          },
          {
            "name": "RPUSH ratio",
            "value": 121.01,
            "unit": "%"
          },
          {
            "name": "LPOP ratio",
            "value": 147.05,
            "unit": "%"
          },
          {
            "name": "RPOP ratio",
            "value": 141.19,
            "unit": "%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "committer": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "distinct": true,
          "id": "c97804972dfacd5c57179d776a2a7c813125b04b",
          "message": "lint: cleaned up on_eof",
          "timestamp": "2026-09-10T19:27:15+01:00",
          "tree_id": "e4eae46b490d4496f76512ead1bcf9bfdb6afc2c",
          "url": "https://github.com/gxnda/rudis/commit/c97804972dfacd5c57179d776a2a7c813125b04b"
        },
        "date": 1795004374806,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE ratio",
            "value": 149.98,
            "unit": "%"
          },
          {
            "name": "PING_MBULK ratio",
            "value": 131.22,
            "unit": "%"
          },
          {
            "name": "SET ratio",
            "value": 133.3,
            "unit": "%"
          },
          {
            "name": "GET ratio",
            "value": 143.81,
            "unit": "%"
          },
          {
            "name": "INCR ratio",
            "value": 135.29,
            "unit": "%"
          },
          {
            "name": "LPUSH ratio",
            "value": 137.39,
            "unit": "%"
          },
          {
            "name": "RPUSH ratio",
            "value": 146.66,
            "unit": "%"
          },
          {
            "name": "LPOP ratio",
            "value": 146.7,
            "unit": "%"
          },
          {
            "name": "RPOP ratio",
            "value": 137.51,
            "unit": "%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "committer": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "distinct": true,
          "id": "e38685ef1d683f3d28b0b616ee8bb5915e7b51d3",
          "message": "lint: renamed handle_empty_stream to on_eof",
          "timestamp": "2026-09-10T19:26:16+01:00",
          "tree_id": "d58e3b75e224ef1e63c97441969fed4d16147d44",
          "url": "https://github.com/gxnda/rudis/commit/e38685ef1d683f3d28b0b616ee8bb5915e7b51d3"
        },
        "date": 1795004502420,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE ratio",
            "value": 147.1,
            "unit": "%"
          },
          {
            "name": "PING_MBULK ratio",
            "value": 135.4,
            "unit": "%"
          },
          {
            "name": "SET ratio",
            "value": 122.71,
            "unit": "%"
          },
          {
            "name": "GET ratio",
            "value": 166.79,
            "unit": "%"
          },
          {
            "name": "INCR ratio",
            "value": 139.92,
            "unit": "%"
          },
          {
            "name": "LPUSH ratio",
            "value": 150.04,
            "unit": "%"
          },
          {
            "name": "RPUSH ratio",
            "value": 133.41,
            "unit": "%"
          },
          {
            "name": "LPOP ratio",
            "value": 138.81,
            "unit": "%"
          },
          {
            "name": "RPOP ratio",
            "value": 119.98,
            "unit": "%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "committer": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "distinct": true,
          "id": "36b873947ba8a8a0af9cb4b20fcbcf7119c926bb",
          "message": "test+lint: added more incomplete command tests to connection + linted read frame",
          "timestamp": "2026-09-10T19:17:10+01:00",
          "tree_id": "d355864e82f2b95f0552a5a1e1dba7550870d548",
          "url": "https://github.com/gxnda/rudis/commit/36b873947ba8a8a0af9cb4b20fcbcf7119c926bb"
        },
        "date": 1795004639082,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE ratio",
            "value": 152.99,
            "unit": "%"
          },
          {
            "name": "PING_MBULK ratio",
            "value": 138.79,
            "unit": "%"
          },
          {
            "name": "SET ratio",
            "value": 147.27,
            "unit": "%"
          },
          {
            "name": "GET ratio",
            "value": 166.65,
            "unit": "%"
          },
          {
            "name": "INCR ratio",
            "value": 160.13,
            "unit": "%"
          },
          {
            "name": "LPUSH ratio",
            "value": 157.77,
            "unit": "%"
          },
          {
            "name": "RPUSH ratio",
            "value": 142.21,
            "unit": "%"
          },
          {
            "name": "LPOP ratio",
            "value": 145.0,
            "unit": "%"
          },
          {
            "name": "RPOP ratio",
            "value": 136.85,
            "unit": "%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "committer": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "distinct": true,
          "id": "eb6ed872d84d3bebe2870ce9c2e86ffeb1273cf8",
          "message": "perf+breaking: reverted timeout to tokio timeout but did not update tests to match",
          "timestamp": "2026-09-10T17:48:14+01:00",
          "tree_id": "eb8a1f6af83a9fa76284367375c47bcd40acb97a",
          "url": "https://github.com/gxnda/rudis/commit/eb6ed872d84d3bebe2870ce9c2e86ffeb1273cf8"
        },
        "date": 1795004769473,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE ratio",
            "value": 170.48,
            "unit": "%"
          },
          {
            "name": "PING_MBULK ratio",
            "value": 138.88,
            "unit": "%"
          },
          {
            "name": "SET ratio",
            "value": 142.18,
            "unit": "%"
          },
          {
            "name": "GET ratio",
            "value": 136.8,
            "unit": "%"
          },
          {
            "name": "INCR ratio",
            "value": 142.09,
            "unit": "%"
          },
          {
            "name": "LPUSH ratio",
            "value": 136.72,
            "unit": "%"
          },
          {
            "name": "RPUSH ratio",
            "value": 129.94,
            "unit": "%"
          },
          {
            "name": "LPOP ratio",
            "value": 147.45,
            "unit": "%"
          },
          {
            "name": "RPOP ratio",
            "value": 136.8,
            "unit": "%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "committer": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "distinct": true,
          "id": "89d9d119ddcbed782a83a156164763bf07a7ee91",
          "message": "fix+perf: fixed test_timeout + RESP serialization uses Bytes natively",
          "timestamp": "2026-09-08T18:30:30+01:00",
          "tree_id": "d53025f4ea4b6da834848a8035ec02630347fdf3",
          "url": "https://github.com/gxnda/rudis/commit/89d9d119ddcbed782a83a156164763bf07a7ee91"
        },
        "date": 1795004904658,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE ratio",
            "value": 144.93,
            "unit": "%"
          },
          {
            "name": "PING_MBULK ratio",
            "value": 131.55,
            "unit": "%"
          },
          {
            "name": "SET ratio",
            "value": 140.03,
            "unit": "%"
          },
          {
            "name": "GET ratio",
            "value": 119.1,
            "unit": "%"
          },
          {
            "name": "INCR ratio",
            "value": 139.93,
            "unit": "%"
          },
          {
            "name": "LPUSH ratio",
            "value": 122.59,
            "unit": "%"
          },
          {
            "name": "RPUSH ratio",
            "value": 134.93,
            "unit": "%"
          },
          {
            "name": "LPOP ratio",
            "value": 135.07,
            "unit": "%"
          },
          {
            "name": "RPOP ratio",
            "value": 152.54,
            "unit": "%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "committer": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "distinct": true,
          "id": "576158039ea6919b2bd6e42c284545c881becffb",
          "message": "chore+docs: Added updated docstring to RespValue::parse() and moved incomplete data handling atomically into parse_buffer",
          "timestamp": "2026-09-04T00:26:22+01:00",
          "tree_id": "b92c22a1201dd70b5b570ebcaf87b39c0d04db30",
          "url": "https://github.com/gxnda/rudis/commit/576158039ea6919b2bd6e42c284545c881becffb"
        },
        "date": 1795005048330,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE ratio",
            "value": 135.02,
            "unit": "%"
          },
          {
            "name": "PING_MBULK ratio",
            "value": 128.54,
            "unit": "%"
          },
          {
            "name": "SET ratio",
            "value": 152.25,
            "unit": "%"
          },
          {
            "name": "GET ratio",
            "value": 142.85,
            "unit": "%"
          },
          {
            "name": "INCR ratio",
            "value": 147.58,
            "unit": "%"
          },
          {
            "name": "LPUSH ratio",
            "value": 123.79,
            "unit": "%"
          },
          {
            "name": "RPUSH ratio",
            "value": 135.09,
            "unit": "%"
          },
          {
            "name": "LPOP ratio",
            "value": 142.89,
            "unit": "%"
          },
          {
            "name": "RPOP ratio",
            "value": 134.86,
            "unit": "%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "committer": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "distinct": true,
          "id": "fd598136ccec8235858ae1b30223f5a07bb55a06",
          "message": "chore: removed redundant start param from internal parses",
          "timestamp": "2026-09-04T00:01:51+01:00",
          "tree_id": "d38d791b95c3c2b8b34d621ea24e71da8eaf11bd",
          "url": "https://github.com/gxnda/rudis/commit/fd598136ccec8235858ae1b30223f5a07bb55a06"
        },
        "date": 1795005191726,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE ratio",
            "value": 131.62,
            "unit": "%"
          },
          {
            "name": "PING_MBULK ratio",
            "value": 131.44,
            "unit": "%"
          },
          {
            "name": "SET ratio",
            "value": 159.88,
            "unit": "%"
          },
          {
            "name": "GET ratio",
            "value": 133.3,
            "unit": "%"
          },
          {
            "name": "INCR ratio",
            "value": 147.71,
            "unit": "%"
          },
          {
            "name": "LPUSH ratio",
            "value": 139.99,
            "unit": "%"
          },
          {
            "name": "RPUSH ratio",
            "value": 134.86,
            "unit": "%"
          },
          {
            "name": "LPOP ratio",
            "value": 152.63,
            "unit": "%"
          },
          {
            "name": "RPOP ratio",
            "value": 133.31,
            "unit": "%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "committer": {
            "email": "gabriellancasterwest@gmail.com",
            "name": "Gabriel Lancaster-West",
            "username": "gxnda"
          },
          "distinct": true,
          "id": "ac156fdd8698d5bd032b015a239f5fc90e5fcba2",
          "message": "fix: fixed nested array parsing with new system",
          "timestamp": "2026-09-03T23:36:10+01:00",
          "tree_id": "634bbee08ba6cdd8427882b035477f8e9a52fd44",
          "url": "https://github.com/gxnda/rudis/commit/ac156fdd8698d5bd032b015a239f5fc90e5fcba2"
        },
        "date": 1795005338279,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE ratio",
            "value": 147.3,
            "unit": "%"
          },
          {
            "name": "PING_MBULK ratio",
            "value": 136.82,
            "unit": "%"
          },
          {
            "name": "SET ratio",
            "value": 160.0,
            "unit": "%"
          },
          {
            "name": "GET ratio",
            "value": 155.07,
            "unit": "%"
          },
          {
            "name": "INCR ratio",
            "value": 154.99,
            "unit": "%"
          },
          {
            "name": "LPUSH ratio",
            "value": 144.87,
            "unit": "%"
          },
          {
            "name": "RPUSH ratio",
            "value": 162.03,
            "unit": "%"
          },
          {
            "name": "LPOP ratio",
            "value": 147.47,
            "unit": "%"
          },
          {
            "name": "RPOP ratio",
            "value": 139.88,
            "unit": "%"
          }
        ]
      }
    ]
  }
}