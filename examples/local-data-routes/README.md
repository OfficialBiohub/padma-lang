# Local Data Routes — M42

এই example M42-এর **read-only local database route dispatch** দেখায়। প্রথম command fixed student এবং product schema দিয়ে project-local SQLite file তৈরি/আপডেট করে। দ্বিতীয় command একই project-এর fixed `127.0.0.1:8080` loopback server চালায়। `server-data-routes.json` শুধু exact `GET` path-কে `student` অথবা `product` collection-এর সঙ্গে map করতে পারে।

## Termux run

```sh
pkg install sqlite -y
cd ~/padma-lang
cargo build --release
cd examples/local-data-routes
mkdir -p data
../../target/release/padma .
../../target/release/padma serve .
```

অন্য একটি Termux session-এ, অথবা server চলার সময় একই terminal থেকে আলাদা tab-এ পরীক্ষা করুন:

```sh
curl --noproxy '*' http://127.0.0.1:8080/health
curl --noproxy '*' http://127.0.0.1:8080/students
curl --noproxy '*' http://127.0.0.1:8080/products
curl --noproxy '*' -X POST http://127.0.0.1:8080/students
```

Expected behavior হলো `/health`, `/students`, এবং `/products`-এ JSON `200`; `POST /students`-এ `405 Method Not Allowed`; এবং unknown route-এ `404 Not Found`। Server বন্ধ করতে `Ctrl-C` দিন।

এই API public internet server নয়। এটি কেবল loopback, only `GET`, and fixed collection listing। Query string, request body, POST/PUT/PATCH/DELETE write operation, record-by-key lookup, filter/search/pagination, arbitrary SQL, user signup/login, authorization, cookie/session, CSRF, file upload, payment, cloud database, backup, browser/account automation, এবং remote deployment এখানে নেই। `sqlite3 -readonly` দিয়ে existing regular project-local `.sqlite` file read করা হয়; M42 server database file তৈরি বা পরিবর্তন করে না।

