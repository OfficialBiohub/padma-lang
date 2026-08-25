# Padma দৈনন্দিন ব্যবহার Tool Roadmap

এই roadmap-এর লক্ষ্য হলো Padma-কে শুধু syntax শেখার ভাষা না রেখে **Termux/Android-এ দৈনন্দিন কাজের ব্যবহারযোগ্য ভাষা** করা। প্রতিটি item বাস্তব runtime, narrow capability, Bangla-English error, example, negative security test, এবং release verification ছাড়া “complete” বলা হবে না।

> **নীতি:** Padma সহজ হবে, কিন্তু hidden authority পাবে না। Data/process/network/device/browser/account action সবসময় explicit capability এবং visible user control-এর অধীন থাকবে।

## এখনই ব্যবহারযোগ্য capability

| ক্ষেত্র | এখন কী করা যায় | সীমা |
|---|---|---|
| ভাষার মূল অংশ | Bangla-English variables, functions, list/map, module, REPL, `padma file.pd`, project mode | Static types ও package registry এখনও নেই |
| ফাইল ও text | Project-root-scoped read/write, path/text utilities, JSON conversion | Project mode-এ arbitrary absolute/shared-storage write নেই |
| HTTP/API | Bounded HTTP(S), JSON request/response, provider-neutral AI request | Explicit `network` grant লাগে; credentials source-এ লেখা যায় না |
| Process/interop | Fixed Python/Node bridge এবং allowlisted executable integration | Shell, arbitrary command, hidden background process নেই |
| Local data | SQLite fixed operations, validated JSON values | Raw SQL console, remote DB, unrestricted ORM নেই |
| Web content | Static HTML writing, backend response envelope, loopback-oriented server contracts | Automatic public hosting/deploy নেই |
| Media | Authorized `yt-dlp` wrapper and prerequisite checks | Unauthorized download/platform bypass নেই |
| Browser | URL planning, visible Android URL handoff, draft/takeover review | Login/CAPTCHA/form/post/payment/browser control নেই |
| AI | Bounded structured workflow and local planning contracts | Autonomous agents, model-output execution, training runtime নেই |

## অগ্রাধিকারভিত্তিক বড় tool list

| Priority | Tool family | First practical result | Safety/quality gate |
|---|---|---|---|
| P0 | **Structured data** | CSV/TSV/JSON table read, validate, filter, map, aggregate, bounded export | `filesystem:read/write`, project-root scope, row/column/byte limits |
| P0 | **Filesystem productivity** | Recursive listing, checksum, safe copy/move/archive *plans*, text search | Traversal/symlink denial, dry-run before mutation, no shared storage by default |
| P0 | **Developer workspace** | Task aliases for format/test/build/check, exit summaries, safe tool prerequisites | Fixed allowlist, no shell parsing, no hidden daemon |
| P1 | **HTTP API productivity** | Reusable JSON request templates, response field extraction, retry/backoff limits | `network:http`, host/input validation, bounded response, secret names only |
| P1 | **Local web/backend** | Validated local routes, JSON response helpers, static-site build workflow | Loopback-only server, request limits, remote deploy remains separate |
| P1 | **SQLite application maintenance** | Migration descriptors, backup/export/import plans, parameterized record APIs | Project-local path, fixed operations, no arbitrary SQL evaluation |
| P1 | **Documents and reporting** | CSV/JSON summaries, text/HTML reports, locally generated data tables | Bounded output path/size and content-specific ownership checks |
| P2 | **AI productivity** | Prompt templates, JSON schema extraction, reviewed local artifacts | Explicit provider capability, output inert, no tool/browser auto-run |
| P2 | **Media productivity** | Metadata inspection, authorized transcode workflows, report generation | User-owned/authorized material, prerequisite validation, no bypass |
| P2 | **Package ecosystem** | Package provenance, offline cache inspection, lock verification | No lifecycle scripts, no automatic install/publish or trust upgrade |
| P3 | **Release and distribution** | SemVer, reproducible artifacts, SBOM, signed releases, Termux package recipe | Official upstream publication before any `pkg install` claim |
| P3 | **Static analysis** | Conservative semantic lint, scope-aware checks, bilingual fixes | Never guess dynamic/imported values or execute source during analysis |

## প্রথম implementation sequence

প্রথম বাস্তব increment হবে **Structured Data Toolkit v1**। বাংলাদেশে freelancing, school records, small business accounting, survey cleaning, inventory, API export, and report generation-এর জন্য CSV/TSV/JSON table processing উচ্চমূল্যের এবং local-first। এটি browser/account/payment authority চায় না, Termux-এ extra service ছাড়া কাজ করতে পারে, এবং existing project-root file boundary পুনর্ব্যবহার করা যায়।

| Increment | Planned public surface | Done হওয়ার প্রমাণ |
|---|---|---|
| M13.1 | `table.read`, `table.headers`, `table.rows`, `table.filter_equal`, `table.select`, `table.count_by`, `table.write_csv` | **Complete.** Typed bounded values, missing-grant/path/malformed-data tests, Bangla-English diagnostics, exact Termux example |
| M13.2 | `fs.list`, `fs.checksum`, `fs.search_text`, and disabled copy/move/archive plans | **Complete.** Project-only non-symlink inspection, no-mutation/security negative tests, exact Termux example |
| M15.1 | Local reporting toolkit | Validated table-to-Markdown/text rendering, bounded project-local `.md` export, injection/path/write denial tests |
| M15.2 | Writing and study-note toolkit | Bangla-English text cleanup, word/line statistics, title/outline helpers, deterministic Markdown notes |
| M15.3 | Household, student, and small-business record schemas | Attendance, expense, inventory, and task summary validation over local table data |
| M15.4 | Freelancer/office document drafts | User-reviewed quote, invoice, portfolio, and client-report templates; no payment/account automation |
| M15.5 | HTTP API request templates | Host/secret/timeout/retry/redaction regression tests |
| M15.6 | Developer workspace task manifests | Fixed executable allowlist, argument vector and exit-code tests |

## M15: কোন user category-র জন্য কী আসবে

| ব্যবহারকারী | সবচেয়ে দরকারি tool path | বর্তমান ভিত্তি | পরবর্তী বাস্তব ফল |
|---|---|---|---|
| School/college student | Note, attendance, marks, simple report | Bangla REPL, JSON/CSV tables, local file write | Markdown study/attendance report |
| Family/personal user | Expense, shopping, task list, document summary | Local tables, checksum, safe path boundary | Printable expense/task report |
| Shop/small business | Inventory, sale record, category summary | CSV filter/select/count/export | Stock and daily summary report |
| Freelancer/office worker | Client data, quote/invoice draft, portfolio report | Local tables, text format, Markdown export | User-reviewed client document draft |
| Teacher/researcher | Survey/marks cleaning, result table, source report | CSV/TSV/JSON validation and aggregation | Deterministic Markdown research/class report |
| Developer/maker | Project data, test/build plan, local API response | Project manifests, fixed bridge, local server contracts | Task manifest and API template increments |
| Creator/media worker | Authorized local metadata, script/text report | Local files, text/JSON, authorized media boundary | Media/document metadata report plan |
| Privacy/security learner | Hash, config inspection, local defensive report | Checksum, safe URL/file inspection, capability system | Redacted configuration/asset report |

The immediate M15.1 increment is **local reporting**, because all of these categories need a readable local result after collecting or validating data. It will consume only an already-validated Padma table value, render inert Markdown/text, and write a report only when a project explicitly grants `filesystem:write`. It will not upload a report, send an email, create an invoice payment, access an account, run a macro, render raw HTML, or start a background process.

## Deliberately not automatic

The following are useful concepts but will not be made silent or unrestricted tools: browser login/session automation, CAPTCHA bypass, credential/cookie/profile collection, JavaScript injection, automatic form/post/upload/download/account/purchase/payment action, generated-output execution, Android permission elevation, ADB/device control, arbitrary shell commands, remote deployment without provider confirmation, or package publication without explicit review.

## How each increment ships

Every tool family follows the same release order: narrow manifest/capability design; strict parser/runtime; Bangla-English diagnostics; allowed and denied security tests; standalone Termux example; public boundary documentation; formatter/root/LSP/release verification; then focused GitHub commit and push. A roadmap row is not a claim that a capability already exists.
