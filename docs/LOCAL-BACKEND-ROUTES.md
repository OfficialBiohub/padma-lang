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

এই helper-এর জন্য capability grant লাগে না, কারণ এটি pure local data transformation। M40-এ existing `padma serve .`-এর সঙ্গে project-root-এর `server-routes.json` যুক্ত হয়েছে। Manifest-এ `server:local` grant থাকলে server fixed `127.0.0.1:8080` loopback-এ bind করে, route file validate করে, bounded HTTP request নেয়, এবং `server.route_response` semantics ব্যবহার করে JSON response দেয়। `server-routes.json` না থাকলে কেবল `/health` endpoint থাকে। Ctrl-C দিয়ে process বন্ধ করা যায়; server public interface-এ bind করে না।

এটি educational website, government portal, e-commerce API, বা public production server নয়। এগুলোর জন্য পরের ধাপে database schema, authentication/authorization, validation, audit, rate limits, deployment, backup, and operational review লাগবে। এই milestone কেবল একই route semantics beginner-friendlyভাবে practice করার নিরাপদ building block।

## Explicit exclusions

`server.route_response` কোনো URL, credential, cookie, token, account, endpoint, file path, shell command, callback, generated-code execution, browser, device, cloud provider, QPU, payment, marketplace, or remote deployment action গ্রহণ করে না। Unknown fields, duplicate routes, unsafe paths, invalid methods/statuses, oversized bodies, non-finite numbers, এবং wrong types bilingual `P1091` diagnostic-এ reject হয়।

## Termux

প্রথমে pure route helper দেখুন:

```sh
cd ~/padma-lang/examples/local-backend-routes
../../target/release/padma .
```

তারপর actual loopback server চালান:

```sh
../../target/release/padma serve .
```

অন্য Termux session থেকে পরীক্ষা করুন:

```sh
curl --noproxy '*' -i http://127.0.0.1:8080/students
curl --noproxy '*' -i http://127.0.0.1:8080/products
curl --noproxy '*' -i http://127.0.0.1:8080/missing
```

Server বন্ধ করতে server session-এ `Ctrl-C` চাপুন। `server-routes.json` project root-এর বাইরে পড়া হয় না; এটি public network server নয়।

## Related references

- [`STANDARD-LIBRARY.md`](STANDARD-LIBRARY.md)
- [`DIAGNOSTICS.md`](DIAGNOSTICS.md)
- [`CAPABILITY-STATUS.md`](CAPABILITY-STATUS.md)
- [`TERMUX-FIRST-GUIDE-BN.md`](TERMUX-FIRST-GUIDE-BN.md)
- [`PRACTICAL-PROJECT-EXAMPLES.md`](PRACTICAL-PROJECT-EXAMPLES.md)
