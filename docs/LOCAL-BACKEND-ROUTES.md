# Local Backend Route Responses

M39 Padma-তে beginner-friendly backend foundation-এর প্রথম route layer যোগ হয়েছে। `server.route_response(request, routes)` একটি explicit in-memory request map এবং route list মিলিয়ে deterministic JSON response envelope তৈরি করে। এটি কোনো callback চালায় না, কোনো socket খোলে না, কোনো file/network/process ব্যবহার করে না, এবং কোনো database বা provider-এর সঙ্গে যুক্ত হয় না।

## API

```padma
ধরি request = {"method": "GET", "path": "/students"}
ধরি routes = [{"method": "GET", "path": "/students", "status": 200, "body": {"ok": true, "items": ["Rafi", "Mitu"]}}]
ধরি response = server.route_response(request, routes)
দেখাও response
```

`request`-এ ঠিক `method` এবং `path` থাকতে হবে। Method কেবল `GET`, `POST`, `PUT`, `PATCH`, অথবা `DELETE`; path ASCII `/` দিয়ে শুরু করা, সর্বোচ্চ 128 bytes, এবং query, fragment, whitespace, বা `..` traversal ছাড়া হতে হবে।

প্রতিটি route-এ ঠিক `method`, `path`, `status`, এবং `body` থাকতে হবে। Route সংখ্যা 1–64, route identity unique, status integer 100–599, এবং JSON body সর্বোচ্চ 256 KiB। প্রথমে সব route validate হয়; তারপর exact method/path match খোঁজা হয়। Match না হলে deterministic 404 response ফেরত আসে।

সফল response-এ `status`, `statusText`, `headers`, JSON-string `body`, `matched`, `routeCount`, এবং `network: "disabled"` থাকে। Header সবসময় `content-type: application/json; charset=utf-8`। Response immutable proposal-এর মতো ব্যবহার করুন; API নিজে কোনো state বদলায় না।

## Capability boundary

এই helper-এর জন্য capability grant লাগে না, কারণ এটি pure local data transformation। Existing `padma serve .` আলাদা CLI capability: সেটি শুধু `127.0.0.1:8080/health` loopback health endpoint চালায় এবং project manifest-এ `server:local` grant চায়। M39 route helper এখনও custom routes-কে socket listener-এ attach করে না। সেই পরের increment-এর জন্য request-body limits, graceful shutdown, timeout, route dispatch lifecycle, observability, authentication, database transaction policy, এবং production deployment model আলাদা করে design/test করতে হবে।

এটি educational website, government portal, e-commerce API, বা public production server নয়। এগুলোর জন্য পরের ধাপে database schema, authentication/authorization, validation, audit, rate limits, deployment, backup, and operational review লাগবে। এই milestone কেবল একই route semantics beginner-friendlyভাবে practice করার নিরাপদ building block।

## Explicit exclusions

`server.route_response` কোনো URL, credential, cookie, token, account, endpoint, file path, shell command, callback, generated-code execution, browser, device, cloud provider, QPU, payment, marketplace, or remote deployment action গ্রহণ করে না। Unknown fields, duplicate routes, unsafe paths, invalid methods/statuses, oversized bodies, non-finite numbers, এবং wrong types bilingual `P1091` diagnostic-এ reject হয়।

## Termux

```sh
cd ~/padma-lang
cargo build --release
cd examples/local-backend-routes
../../target/release/padma .
```

এই example local response maps print করে; এটি network server চালায় না।

## Related references

- [`STANDARD-LIBRARY.md`](STANDARD-LIBRARY.md)
- [`DIAGNOSTICS.md`](DIAGNOSTICS.md)
- [`CAPABILITY-STATUS.md`](CAPABILITY-STATUS.md)
- [`TERMUX-FIRST-GUIDE-BN.md`](TERMUX-FIRST-GUIDE-BN.md)
- [`PRACTICAL-PROJECT-EXAMPLES.md`](PRACTICAL-PROJECT-EXAMPLES.md)
