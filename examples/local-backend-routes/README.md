# Local Backend Routes

এই `server:local` example-এ Padma একটি education-style `/students` route এবং e-commerce-style `/products` route চালায়। `server.route_response` pure route semantics দেখায়, আর `padma serve .` একই project-local `server-routes.json` দিয়ে fixed loopback server চালায়।

```sh
cd ~/padma-lang
cargo build --release
cd examples/local-backend-routes
../../target/release/padma .
../../target/release/padma serve .
```

Expected output:

```text
200
{"items":["Rafi","Mitu"],"ok":true}
404
false
disabled
```

`server.route_response` এবং loopback server exact method/path match করে। Match হলে configured status এবং JSON body ফেরত দেয়; match না হলে 404 দেয়। Server fixed `127.0.0.1:8080`-এ bind করে এবং `curl --noproxy '*' -i http://127.0.0.1:8080/students` দিয়ে পরীক্ষা করা যায়। বন্ধ করতে `Ctrl-C` চাপুন। Unknown fields, duplicate route, unsafe path, invalid status, oversized body, network URL, shell command, callback, token, credential, database, বা remote deployment field ব্যবহার করা যাবে না।

বাস্তব website backend বানাতে পরের ধাপে database schema, authentication, authorization, request-body policy, CSRF/rate-limit policy, audit logging, tests, deployment, backup, and operations design লাগবে। এই example-কে governmental website, school portal, e-commerce checkout, payment service, বা public internet deployment হিসেবে ব্যবহার করবেন না। Full production backend-এর জন্য database, authentication/authorization, request validation, CSRF/rate limits, audit, observability, deployment, backup, and operations review আলাদা করে দরকার।
