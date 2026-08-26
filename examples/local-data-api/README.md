# Local Data API — M41

এই example-এ Padma একটি fixed local SQLite data model ব্যবহার করে school student record এবং ছোট product inventory সংরক্ষণ ও পড়বে। এটি public website বা cloud API নয়; database file project folder-এর ভেতরে থাকে এবং `database:sqlite` capability ছাড়া code চলবে না।

## Termux run

```sh
pkg install sqlite -y
cd ~/padma-lang
cargo build --release
cd examples/local-data-api
mkdir -p data
../../target/release/padma .
```

Expected output-এর shape:

```text
true
{"active": true, "class": 6, "guardian": নিলা, "name": রিমা, "school": Padma School}
true
[{"key": p-001, "value": {"category": শিক্ষা, "currency": BDT, "name": খাতা, "price": 55, "stock": 20}}]
```

`db.student_save`-এ student-এর পাঁচটি field এবং `db.product_save`-এ product-এর পাঁচটি field দিতে হয়। একই key আবার save করলে record replace হয়; `student_get`, `student_list`, এবং `product_get`, `product_list` read operation। Field-এর Bangla বা English নাম ব্যবহার করা যায়, কিন্তু একই field-এর দুই alias একসঙ্গে দেওয়া যাবে না।

এই example এখনও HTTP request dispatch করে না। Existing `server:local` route server fixed static `server-routes.json` response map ব্যবহার করে; M41 database helpers সেই local data layer প্রস্তুত করেছে। Authentication, authorization, arbitrary SQL/ORM, search, pagination, backup, cloud database, payment, browser/account automation, এবং remote deployment এখানে নেই।
