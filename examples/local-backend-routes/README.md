# Local Backend Routes

এই no-capability example-এ Padma একটি education-style `/students` route এবং e-commerce-style `/products` route-এর deterministic JSON response map তৈরি করে। এটি beginner-friendly backend semantics শেখায়, কিন্তু socket/network server চালায় না।

```sh
cd ~/padma-lang
cargo build --release
cd examples/local-backend-routes
../../target/release/padma .
```

Expected output:

```text
200
{"items":["Rafi","Mitu"],"ok":true}
404
false
disabled
```

`server.route_response` exact method/path match করে। Match হলে configured status এবং JSON body ফেরত দেয়; match না হলে 404 দেয়। Unknown fields, duplicate route, unsafe path, invalid status, oversized body, network URL, shell command, callback, token, credential, database, বা remote deployment field ব্যবহার করা যাবে না।

বাস্তব website backend বানাতে পরের ধাপে database schema, authentication, authorization, request-body policy, CSRF/rate-limit policy, audit logging, tests, deployment, backup, and operations design লাগবে। এই example-কে governmental website, school portal, e-commerce checkout, payment service, বা public internet deployment হিসেবে ব্যবহার করবেন না।
