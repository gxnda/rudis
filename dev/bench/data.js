window.BENCHMARK_DATA = {
  "lastUpdate": 1785962416766,
  "repoUrl": "https://github.com/gxnda/rudis",
  "entries": {
    "Rudis Performance": [
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
          "id": "cb14c9862d8bdfd3904b994b85a539ce7260948c",
          "message": "chore: replaced \\r with \\n in benchmark parsing",
          "timestamp": "2026-08-05T17:49:47+01:00",
          "tree_id": "6c0479722a77e05cbe82c2ba1cfd00b429e83ed4",
          "url": "https://github.com/gxnda/rudis/commit/cb14c9862d8bdfd3904b994b85a539ce7260948c"
        },
        "date": 1785948662062,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE",
            "value": 58892.82,
            "unit": "undefined"
          },
          {
            "name": "PING_MBULK",
            "value": 58754.41,
            "unit": "undefined"
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
          "id": "06c91bf31e4e4604172e0627d5ab7582b9f8c17f",
          "message": "added TODOs to README",
          "timestamp": "2026-08-05T20:53:05+01:00",
          "tree_id": "66629e13f334498a559ccf8dba6370f4d94b1508",
          "url": "https://github.com/gxnda/rudis/commit/06c91bf31e4e4604172e0627d5ab7582b9f8c17f"
        },
        "date": 1785959661639,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE",
            "value": 55617.35,
            "unit": "undefined"
          },
          {
            "name": "PING_MBULK",
            "value": 55865.92,
            "unit": "undefined"
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
          "id": "2f86f9aab55d5269930c2c5d7369f91c139fa294",
          "message": "chore: updated dependancies + added memchr",
          "timestamp": "2026-08-05T20:56:54+01:00",
          "tree_id": "01a016b2b65fcf233042313d890f6c5bb5fa38cd",
          "url": "https://github.com/gxnda/rudis/commit/2f86f9aab55d5269930c2c5d7369f91c139fa294"
        },
        "date": 1785959882974,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE",
            "value": 57736.72,
            "unit": "undefined"
          },
          {
            "name": "PING_MBULK",
            "value": 56179.78,
            "unit": "undefined"
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
          "id": "16b683c433aec72a9d776e280e77051fd5bc5b71",
          "message": "perf: hopefully optimised parse_until_crlf in RESP using memchr::memmem",
          "timestamp": "2026-08-05T20:57:35+01:00",
          "tree_id": "935db3facb9b8e3a3411e4bf42bb1c63b1db4d99",
          "url": "https://github.com/gxnda/rudis/commit/16b683c433aec72a9d776e280e77051fd5bc5b71"
        },
        "date": 1785959931080,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE",
            "value": 58927.52,
            "unit": "undefined"
          },
          {
            "name": "PING_MBULK",
            "value": 58719.91,
            "unit": "undefined"
          }
        ]
      }
    ],
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
          "id": "99aa0c3699bf878f3cbf6c65e397d56928d53724",
          "message": "perf: hopefully made expiry (de)serialisation faster by reducing time calls",
          "timestamp": "2026-08-05T21:29:05+01:00",
          "tree_id": "0ca804769ee400594e7a7e1a928908541e626d4e",
          "url": "https://github.com/gxnda/rudis/commit/99aa0c3699bf878f3cbf6c65e397d56928d53724"
        },
        "date": 1785961817839,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE",
            "value": 57372.34,
            "unit": "undefined"
          },
          {
            "name": "PING_MBULK",
            "value": 56561.09,
            "unit": "undefined"
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
          "id": "b51c0bbfab208f99cb8d84210554c8685c8cc969",
          "message": "chore: removed unused imports",
          "timestamp": "2026-08-05T21:38:49+01:00",
          "tree_id": "6cac59e86a82de65452979c8db58893ebe64b305",
          "url": "https://github.com/gxnda/rudis/commit/b51c0bbfab208f99cb8d84210554c8685c8cc969"
        },
        "date": 1785962416480,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE",
            "value": 59665.87,
            "unit": "undefined"
          },
          {
            "name": "PING_MBULK",
            "value": 60459.49,
            "unit": "undefined"
          }
        ]
      }
    ]
  }
}