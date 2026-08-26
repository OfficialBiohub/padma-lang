# Practical Padma Project Examples

This guide contains small **runnable** Termux-first Padma projects. Each project uses only currently implemented APIs, declares its required capability grants, and distinguishes a real result from a plan or a future adapter. Start with the release binary built from this repository.

```bash
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
```

> **Important:** A capability grant is permission for one limited Padma operation, not unrestricted phone, network, browser, or server access. Run `padma capabilities .` inside a project to review its grants before running it.

| Project | Directory | What it demonstrates | Current boundary |
|---|---|---|---|
| Authorized media download | `examples/authorized-media-download` | Interactive `media.download` with `yt-dlp` | Only content you own or are authorized to download; no platform bypass or shared-storage write in project mode. |
| Static website builder | `examples/static-website-builder` | Writing an HTML file safely inside a project | It creates a website file; Padma does not publish it or operate a public host. |
| Backend response pipeline | `examples/backend-response-pipeline` | Producing a validated JSON HTTP response envelope | It does not open a public server or receive requests. |
| Local backend routes | `examples/local-backend-routes` | Matching explicit education/e-commerce-style method/path requests to deterministic JSON response maps and serving them through fixed `127.0.0.1:8080` when `server:local` is granted | It does not bind publicly, read a database, authenticate users, process payments, or deploy a public website. |
| Local database routes | `examples/local-data-routes` | Seeds fixed student/product records into a project-local SQLite file, then serves only configured `GET` collection lists through fixed `127.0.0.1:8080` with `server:local` and `database:sqlite` | It does not accept request bodies or write methods, expose arbitrary SQL/query/filter/pagination, bind publicly, authenticate users, process payments, or deploy a public website. |
| Student SQLite records | `examples/student-records-sqlite` | Local structured persistence without raw SQL | It is not an ORM, remote database, or arbitrary SQL console. |
| Defensive URL inspector | `examples/defensive-url-inspector` | Input validation and manual-review report | It checks syntax only; it does not scan, attack, or prove a site is safe. |
| Local password check | `examples/local-password-check` | Local password hash and verification boundary | It is not a hosted login, user database, password reset, or authentication server. |
| Structured data inventory | `examples/structured-data-inventory` | Local CSV filtering, selection, count, and report export | It is not Excel/XLSX, cloud sync, shared-storage access, or a spreadsheet macro runner. |
| Filesystem productivity | `examples/filesystem-productivity` | Project-local listing, checksum, text search, and disabled copy planning | It is not file mutation, Android shared-storage access, shell execution, or a file-management daemon. |
| Local expense reporting | `examples/local-reporting-expense` | CSV-to-Markdown local expense report | It is not tax/accounting advice, invoice payment, email/upload, shared-storage, or public publishing. |
| Local profile validator | `examples/local-profile-validator` | Project-local JSON preferences with schema/default validation | It is not account/login state, game modification, anti-cheat bypass, network profile sync, or device settings control. |
| Freelancer quote draft | `examples/freelancer-quote-draft` | Local escaped quote Markdown with redacted review summary | It cannot contact a client, submit to a marketplace, sign a contract, make a payment, use a browser/network, or write shared storage. |
| Freelancer scope-of-work | `examples/freelancer-scope-of-work` | Reviewed local scope/exclusion Markdown draft | It cannot contact a client, sign/accept, submit, pay, use a marketplace/network/browser, or write shared storage. |
| Freelancer delivery checklist | `examples/freelancer-delivery-checklist` | Reviewed local deliverable/review/handover Markdown draft | It cannot contact, upload/download, submit delivery, sign/accept, pay, use a marketplace/network/browser, or write shared storage. |
| Freelancer portfolio and handoff | `examples/freelancer-portfolio-handoff` | Public case-study Markdown and user-mediated message/attachment review preparation | It cannot send, post, upload/download, submit delivery, sign, pay, log in, use a browser/network/account, or write shared storage. |
| Freelancer attachment review | `examples/freelancer-attachment-review` | Local checksum/byte-count attachment-review manifest with destination and ownership labels | It cannot send, upload/download, submit delivery, sign, pay, log in, use a browser/network/account, or write shared storage. |
| Freelancer delivery package | `examples/freelancer-delivery-package` | Local checksum/byte-count integrity manifest with manual folder and review checklist | It cannot copy files, render a PDF, send, upload/download, submit delivery, sign, pay, log in, use a browser/network/account, or write shared storage. |
| Freelancer client templates | `examples/freelancer-client-templates` | Bangla-English local proposal, brief, and copy-only message-template preparation | It cannot find recipients, send/post, upload/download, submit delivery, sign, pay, log in, use a browser/network/account, or write shared storage. |
| Local quantum circuit planning | `examples/local-quantum-planning` | Classical numeric values feed bounded `rx`/`ry`/`rz` circuit maps, deterministic OpenQASM 3.0, local exact probability data, single Pauli-product expectations, and explicit-seed bounded local counts | It cannot bind symbolic parameters, use a hidden seed/noise/collapse state, evaluate a Hamiltonian/algorithm, use a QPU/provider, read credentials, submit a job, access a network, or start a process. |
| Local OpenQASM interchange assessment | `examples/local-quantum-interchange-assessment` | Byte-exact assessment of a generated bounded OpenQASM 3.0 artifact and stable local metadata | It is not an arbitrary QASM parser/importer/compiler, source executor, file reader, QPU/provider submission path, credential reader, network client, or process runner. |
| Quantum provider readiness assessment | `examples/local-quantum-provider-readiness` | Redacted local controls required before a future provider adapter could be designed | It is not a provider login/token reader, account/backend/cost lookup, job submission/polling/cancellation path, provider SDK, QPU executor, network client, or process runner. |
| Termux CLI smoke test | `examples/termux-cli-smoke` | A temporary-copy release-binary project check with Bangla source and digits | It does not publish/install a package, invoke cloud services, or run optional external tools. |
| Local optimisation primitives | `examples/local-optimization-primitives` | Explicit quadratic value, centered finite-difference gradient, and one clamped projected proposal | It cannot mutate parameters, repeat/auto-run a loop, execute a callback, train a model, run VQE/QAOA/QML/Grover, contact a provider/QPU, read credentials, access a network, or start a process. |
| Local household records | `examples/local-records-household` | Attendance, expense, and inventory CSV validation plus a local Markdown expense report | It cannot cloud-sync, contact a school/shop, make a payment, start a process, or take stock/account action. |

## 1. Authorized media download

Before running this example, install `yt-dlp` in Termux:

```bash
pkg install python -y
python -m pip install --upgrade yt-dlp
cd ~/padma-lang/examples/authorized-media-download
padma .
```

The program is:

```padma
let url = input("Enter a video URL you are authorized to download: ")
let result = media.download(url, "video-%(id)s.%(ext)s")
print "yt-dlp output:"
print result
```

| Code | How it works |
|---|---|
| `input(...)` | Reads the URL only when the program is running. |
| `media.download(...)` | Validates an HTTP(S) URL, checks `media:download` and `filesystem:write`, then invokes the fixed `yt-dlp` program without a shell. |
| `video-%(id)s.%(ext)s` | Gives `yt-dlp` a safe project-relative output template. The downloaded file is created in the project directory. |
| `print result` | Shows `yt-dlp`'s own successful output text. Exact lines depend on the permitted media and installed `yt-dlp` version. |

An example interaction is:

```text
Enter a video URL you are authorized to download: https://example.org/my-permitted-video
yt-dlp output:
[download] Destination: video-example.mp4
```

The URL and output lines above are illustrative. Do not use this example to download content without the owner's permission or contrary to a service's terms.

## 2. Static website builder

```bash
cd ~/padma-lang/examples/static-website-builder
padma .
python -m http.server 8000 --directory site
```

The Padma program writes a simple HTML page:

```padma
let page = "<!doctype html>\n<html lang=\"en\">\n<head><meta charset=\"utf-8\"><title>Padma Site</title></head>\n<body><main><h1>Hello from Padma</h1><p>This page was written by a Padma program.</p></main></body>\n</html>\n"
file.write("site/index.html", page)
print "Created site/index.html"
```

The output is:

```text
Created site/index.html
Preview it with: python -m http.server 8000 --directory site
```

`file.write` may write only under the canonical project root because the manifest grants `filesystem = ["write"]`. The final Python command is optional local preview tooling; it is not a Padma public deployment command.

## 3. Backend response pipeline

```bash
cd ~/padma-lang/examples/backend-response-pipeline
padma .
cat out/health-response.json
```

```padma
let response = backend.response(
  200,
  {"Content-Type": "application/json"},
  {"ok": true, "message": "Padma backend response prepared"}
)

automation.write_json("out/health-response.json", response)
print json.stringify(response)
```

The generated output is equivalent to:

```json
{"body":{"message":"Padma backend response prepared","ok":true},"headers":{"Content-Type":"application/json"},"status":200.0}
```

`backend.response` validates the numeric status, text header map, and JSON-compatible body. `automation.write_json` stores the resulting response envelope under the project root. This is useful for a reviewed local adapter or job queue, but it does **not** expose a phone to the internet or create arbitrary HTTP routes.

## 4. Student SQLite records

Install the Termux SQLite command once, then run the project:

```bash
pkg install sqlite -y
cd ~/padma-lang/examples/student-records-sqlite
padma .
```

```padma
ধরি সংরক্ষণ = db.put("data/ছাত্র.sqlite", "শ্রেণি", "রিমা", {
  "নাম": "রিমা",
  "ক্লাস": 6,
  "বিষয়": ["গণিত", "বিজ্ঞান"]
})

দেখাও সংরক্ষণ
দেখাও json.stringify(db.get("data/ছাত্র.sqlite", "শ্রেণি", "রিমা"))
```

Expected output is:

```text
true
{"ক্লাস":6.0,"নাম":"রিমা","বিষয়":["গণিত","বিজ্ঞান"]}
```

`db.put` stores one JSON-compatible value under a namespace and key. `db.get` retrieves that value. The project grants only `database = ["sqlite"]`; Padma uses fixed operations and parameter binding instead of letting input become arbitrary SQL. [1]

## 5. Defensive URL inspector

```bash
cd ~/padma-lang/examples/defensive-url-inspector
padma .
```

```padma
let candidate = input("URL to inspect: ")
let valid = url.is_valid(candidate)

let report = {
  "candidate": candidate,
  "validSyntax": valid,
  "action": "manual-review-required"
}

if valid {
  print "URL syntax is valid. Review ownership and destination before opening it."
} else {
  print "Rejected: the URL syntax is invalid."
}

print json.stringify(report)
```

For `https://example.org`, the output is:

```text
URL syntax is valid. Review ownership and destination before opening it.
{"action":"manual-review-required","candidate":"https://example.org","validSyntax":true}
```

This is a defensive input-validation example. A syntactically valid URL is not proof that content is trustworthy, owned by the user, free of malware, or authorized to access. Padma does not implement port scanning, vulnerability exploitation, credential attacks, CAPTCHA bypass, or unauthorized browser automation.

## 6. Local password check

```bash
cd ~/padma-lang/examples/local-password-check
padma .
```

```padma
let password = input("Choose a password for this local demonstration: ")
let record = auth.password_hash(password)

print "Password record created without printing the password."
print auth.password_verify(record, password)
```

Expected output is:

```text
Choose a password for this local demonstration: ********
Password record created without printing the password.
true
```

The terminal may display typed characters depending on its input settings; the asterisks above are illustrative. Padma requires the password to come from a runtime variable rather than a hard-coded string literal. The resulting salted record can be verified locally, but this example does not create an online account system or a public login service. [2]

## 7. Structured data inventory

```bash
cd ~/padma-lang/examples/structured-data-inventory
padma .
cat out/food-inventory.csv
```

```padma
ধরি inventory = table.read("data/inventory.csv", "csv")
ধরি food = table.filter_equal(inventory, "category", "food")
ধরি report = table.select(food, ["name", "price"])
table.write_csv("out/food-inventory.csv", report)
```

This example reads the project-local CSV, matches the `food` category exactly, keeps two chosen columns, and writes a project-local CSV. It needs `filesystem = ["read", "write"]`. It cannot traverse to another directory, write Android Downloads, contact a cloud spreadsheet, read an Excel workbook, or execute a macro. See [`STRUCTURED-DATA.md`](STRUCTURED-DATA.md) for supported formats and limits.

## 8. Filesystem productivity

```bash
cd ~/padma-lang/examples/filesystem-productivity
padma .
```

```padma
ধরি checksum = fs.checksum("workspace/notes.txt")
ধরি found = fs.search_text("workspace/notes.txt", "review", 3)
ধরি plan = fs.copy_plan("workspace/notes.txt", "workspace/notes-copy.txt")
দেখাও plan["execution"]
```

The program reads only project-local regular files, reports a checksum and search match, then returns a copy descriptor whose execution state is `disabled`. It needs `filesystem = ["read"]`; it creates no destination file. It cannot copy, move, archive, delete, rename, scan other storage, invoke a shell, run in the background, or access Android Downloads. See [`FILESYSTEM-PRODUCTIVITY.md`](FILESYSTEM-PRODUCTIVITY.md) for full limits.

## 9. Local expense reporting

```bash
cd ~/padma-lang/examples/local-reporting-expense
padma .
cat out/january-expense.md
```

```padma
ধরি expenses = table.read("data/expenses.csv", "csv")
ধরি summary = report.summary("January Expense Report", expenses)
report.write_markdown("out/january-expense.md", "January Expense Report", expenses)
দেখাও summary["rowCount"]
```

This example validates project-local CSV rows, makes an in-memory report summary, then writes a project-local Markdown file. It requires `filesystem = ["read", "write"]`. It does not calculate tax, issue a financial recommendation, send/charge an invoice, upload/email/share the report, render scripts, start a background job, or write shared Android storage. See [`LOCAL-REPORTING.md`](LOCAL-REPORTING.md) for output limits and escaping rules.

## 10. Local profile validator

```bash
cd ~/padma-lang/examples/local-profile-validator
padma .
```

```padma
ধরি profile = json.parse(file.read("data/profile.json"))
ধরি checked = profile.validate(profile, schema)
ধরি summary = profile.summary(profile, schema)
দেখাও checked["theme"]
```

The example reads a project-local JSON preference file, validates only declared scalar fields, applies an explicit safe default, and emits a redacted summary. It needs `filesystem = ["read"]` only for `file.read`; profile validation itself has no capability. It cannot access a user account, browser, network, device setting, running game, game item/score/account, anti-cheat system, or process. See [`LOCAL-PROFILES.md`](LOCAL-PROFILES.md) for schema limits and the legal game-project boundary.

## 11. Freelancer quote draft

```bash
cd ~/padma-lang/examples/freelancer-quote-draft
padma .
cat out/portfolio-quote.md
```

```padma
ধরি summary = client.document_summary(draft)
ধরি saved = client.write_document("out/portfolio-quote.md", draft)
দেখাও summary["payment"]
দেখাও saved
```

The example prepares an explicit local quote draft, emits only redacted document metadata to the terminal, and writes one escaped project-local Markdown file. It needs `filesystem = ["write"]` only for the final file write. It is not a contract, tax or legal service, invoice transmission tool, marketplace client scraper, proposal sender, account/login system, e-signature service, payment/withdrawal tool, browser automation, network client, or process runner. See [`CLIENT-DOCUMENTS.md`](CLIENT-DOCUMENTS.md) for the exact fields, limits, and review boundary.

## 12. Local household records

```bash
cd ~/padma-lang/examples/local-records-household
padma .
cat out/family-expense.md
```

```padma
ধরি expenses = table.read("data/expenses.csv", "csv")
ধরি summary = record.summary("expense", expenses)
দেখাও text.format("Expense: {totalAmount} {currency}", summary)
```

The example validates separate attendance, expense, and inventory CSVs before emitting redacted counts/totals and writing one local Markdown expense report. It needs `filesystem = ["read", "write"]` only for project-local data/report files. It does not send a record, sync cloud data, calculate tax, start a background job, contact a school/client/shop, create payment, alter inventory, open a browser, or use an account. See [`LOCAL-RECORDS.md`](LOCAL-RECORDS.md) for exact record rules.

## Before you expand an example

Keep a project capability list as small as possible. A static website creator needs only filesystem write access; it does not need network, database, process, media, identity, server, Android, GUI, or deployment permission. The `padma capabilities .` command displays the active grants and their scope before you execute the program.

## References

[1]: ./SQLITE-PERSISTENCE.md "Padma Local SQLite Persistence"
[2]: ./IDENTITY-SESSION.md "Padma Identity and Session Foundation"
