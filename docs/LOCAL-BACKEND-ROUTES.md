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

## M42 read-only local data routes

M42-এ `padma serve .` একটি দ্বিতীয়, mutually-exclusive configuration file গ্রহণ করে: `server-data-routes.json`। এটি `server-routes.json`-এর সঙ্গে এক project-এ ব্যবহার করা যাবে না। Data route চালাতে manifest-এ **দুইটি** explicit grant প্রয়োজন: `server = ["local"]` এবং `database = ["sqlite"]`।

```json
{
  "version": 1,
  "database": "data/app.sqlite",
  "routes": [
    {"path": "/students", "collection": "student"},
    {"path": "/products", "collection": "product"}
  ]
}
```

| Rule | M42 contract |
|---|---|
| File | One regular, UTF-8, project-root `server-data-routes.json`, maximum 64 KiB |
| Exact root fields | `version`, `database`, `routes` only; version must be `1` |
| Database | Existing regular project-local `.sqlite` file only; absolute paths, traversal, symlinks, and missing files are rejected |
| Route fields | `path`, `collection` only; up to 32 unique ASCII paths without query, fragment, whitespace, or traversal |
| Collections | Only managed typed `student` and `product` collections |
| Methods | Endpoint listing is **GET only**; write methods return `405` and bodies remain unsupported |
| Responses | `/health` is fixed; configured `GET` endpoints return deterministic JSON lists; missing path returns `404` |
| Database mode | Padma invokes the existing `sqlite3` CLI with `-readonly`, bounded 5-second execution, cleared child environment, and bounded output |

Stored rows are revalidated against M41’s strict student/product schemas before a response is sent. A malformed, unavailable, missing, or unreadable local database returns a generic `500 {"error":"local_data_unavailable"}` body; filesystem details, SQL text, and SQLite stderr are not disclosed. `P1093` reports invalid local data-route configuration at server startup.

The complete example is [`examples/local-data-routes`](../examples/local-data-routes). Run its seed program once, then start the server and request `/students` or `/products`.

এটি educational website, government portal, e-commerce API, বা public production server নয়। M42 database **read listing** যোগ করেছে, কিন্তু request body, create/update/delete, record lookup by user input, filtering/search/pagination, SQL/ORM/migration, authentication/authorization, audit, CSRF, rate limits, deployment, backup, and operational review এখনও আলাদা কাজ। এই milestone কেবল একই route semantics beginner-friendlyভাবে practice করার নিরাপদ building block।

## Explicit exclusions

`server.route_response` কোনো URL, credential, cookie, token, account, endpoint, file path, shell command, callback, generated-code execution, browser, device, cloud provider, QPU, payment, marketplace, or remote deployment action গ্রহণ করে না। Unknown fields, duplicate routes, unsafe paths, invalid methods/statuses, oversized bodies, non-finite numbers, এবং wrong types bilingual `P1091` diagnostic-এ reject হয়।

`server-data-routes.json` arbitrary SQL, database URL, database path override, write method, request body, query parameter, secret/credential/token, callback, file action, provider, browser/account, payment, deployment, or remote network field গ্রহণ করে না। Unknown/inexact fields, unsafe path/file, unsupported collection, duplicate path, missing database, or both route-file types present হলে M42 startup reject করে অথবা `P1093` error দেয়.

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
