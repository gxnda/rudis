window.BENCHMARK_DATA = {
  "lastUpdate": 1786198475786,
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
          "id": "b56f7db5710fc34074e903b5221e99c0fd54ac95",
          "message": "perf: reduced total number of now() calls by checking after Some expiry",
          "timestamp": "2026-08-06T13:48:02+01:00",
          "tree_id": "108e0e4124a8659891a68a319b21dd6913b22602",
          "url": "https://github.com/gxnda/rudis/commit/b56f7db5710fc34074e903b5221e99c0fd54ac95"
        },
        "date": 1786020560436,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE",
            "value": 57636.89,
            "unit": "undefined"
          },
          {
            "name": "PING_MBULK",
            "value": 58823.53,
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
          "id": "51412206dc5029efb27a76b724e59ad21166e8e4",
          "message": "perf: reduced now() calls in memory and commands",
          "timestamp": "2026-08-06T14:42:15+01:00",
          "tree_id": "89fd882cfff1e49733e747e53ae84153212cab39",
          "url": "https://github.com/gxnda/rudis/commit/51412206dc5029efb27a76b724e59ad21166e8e4"
        },
        "date": 1786023816098,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE",
            "value": 58788.95,
            "unit": "undefined"
          },
          {
            "name": "PING_MBULK",
            "value": 58105.75,
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
          "id": "3cceb22d92f86242b6b95c72cb3ce094c06240be",
          "message": "removed newline (ikr!)",
          "timestamp": "2026-08-06T15:22:52+01:00",
          "tree_id": "e19f3a4891468ac15a38ef7d567373da64dd9632",
          "url": "https://github.com/gxnda/rudis/commit/3cceb22d92f86242b6b95c72cb3ce094c06240be"
        },
        "date": 1786026248087,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE",
            "value": 56915.2,
            "unit": "undefined"
          },
          {
            "name": "PING_MBULK",
            "value": 58962.27,
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
          "id": "48900e34e42bc73881613f75d46b12ff6740eb8d",
          "message": "chore: unticked TODO, now() is not efficient enough",
          "timestamp": "2026-08-08T13:28:06+01:00",
          "tree_id": "a3be9b1b32dfdbdb46d351db3b3d2dd16fdec6ac",
          "url": "https://github.com/gxnda/rudis/commit/48900e34e42bc73881613f75d46b12ff6740eb8d"
        },
        "date": 1786192165973,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE",
            "value": 58788.95,
            "unit": "undefined"
          },
          {
            "name": "PING_MBULK",
            "value": 59772.86,
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
          "id": "85e141bac8a9cf22c06ed56a9d9de031f8d79a30",
          "message": "test: added more benchmarks to GH bench action",
          "timestamp": "2026-08-08T13:33:43+01:00",
          "tree_id": "6b8a68576f4a0411abb10e00b2d466c653c7b5d4",
          "url": "https://github.com/gxnda/rudis/commit/85e141bac8a9cf22c06ed56a9d9de031f8d79a30"
        },
        "date": 1786192512486,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE",
            "value": 58241.12,
            "unit": "undefined"
          },
          {
            "name": "PING_MBULK",
            "value": 59453.03,
            "unit": "undefined"
          },
          {
            "name": "SET",
            "value": 58479.53,
            "unit": "undefined"
          },
          {
            "name": "GET",
            "value": 59594.76,
            "unit": "undefined"
          },
          {
            "name": "INCR",
            "value": 59171.59,
            "unit": "undefined"
          },
          {
            "name": "LPUSH",
            "value": 59488.4,
            "unit": "undefined"
          },
          {
            "name": "RPUSH",
            "value": 58892.82,
            "unit": "undefined"
          },
          {
            "name": "LPOP",
            "value": 59066.75,
            "unit": "undefined"
          },
          {
            "name": "RPOP",
            "value": 58343.06,
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
          "id": "d6b6f8db75cfd0fbc42d6331bf3ceb1ab874a020",
          "message": "perf!: active BG expiration runs every 5 (was 1) seconds, added remove_older_than to lower now() calls",
          "timestamp": "2026-08-08T14:02:59+01:00",
          "tree_id": "278eb16d66a08943613107fb284569d96c85deee",
          "url": "https://github.com/gxnda/rudis/commit/d6b6f8db75cfd0fbc42d6331bf3ceb1ab874a020"
        },
        "date": 1786194266062,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE",
            "value": 59206.63,
            "unit": "undefined"
          },
          {
            "name": "PING_MBULK",
            "value": 59347.18,
            "unit": "undefined"
          },
          {
            "name": "SET",
            "value": 58411.21,
            "unit": "undefined"
          },
          {
            "name": "GET",
            "value": 59453.03,
            "unit": "undefined"
          },
          {
            "name": "INCR",
            "value": 59171.59,
            "unit": "undefined"
          },
          {
            "name": "LPUSH",
            "value": 58173.36,
            "unit": "undefined"
          },
          {
            "name": "RPUSH",
            "value": 59347.18,
            "unit": "undefined"
          },
          {
            "name": "LPOP",
            "value": 58719.91,
            "unit": "undefined"
          },
          {
            "name": "RPOP",
            "value": 58858.15,
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
          "id": "180dc866cb0fbee764c0fb18756d92827f482bb9",
          "message": "perf?: added vibe-coded benchmarks",
          "timestamp": "2026-08-08T15:13:08+01:00",
          "tree_id": "01ad6644e18f2bd072619e912031dd3d84e7c651",
          "url": "https://github.com/gxnda/rudis/commit/180dc866cb0fbee764c0fb18756d92827f482bb9"
        },
        "date": 1786198475060,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "PING_INLINE",
            "value": 59665.87,
            "unit": "undefined"
          },
          {
            "name": "PING_MBULK",
            "value": 59311.98,
            "unit": "undefined"
          },
          {
            "name": "SET",
            "value": 58788.95,
            "unit": "undefined"
          },
          {
            "name": "GET",
            "value": 57836.9,
            "unit": "undefined"
          },
          {
            "name": "INCR",
            "value": 59311.98,
            "unit": "undefined"
          },
          {
            "name": "LPUSH",
            "value": 57870.37,
            "unit": "undefined"
          },
          {
            "name": "RPUSH",
            "value": 58275.06,
            "unit": "undefined"
          },
          {
            "name": "LPOP",
            "value": 59206.63,
            "unit": "undefined"
          },
          {
            "name": "RPOP",
            "value": 58038.3,
            "unit": "undefined"
          }
        ]
      }
    ]
  }
}