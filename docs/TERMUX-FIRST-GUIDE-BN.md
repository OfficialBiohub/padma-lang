# Termux-এ Padma শুরু করার সংক্ষিপ্ত গাইড

এই গাইডটি **কম্পিউটার ছাড়া Android ফোনে Termux ব্যবহারকারী** শিক্ষার্থীর জন্য। Padma এখন পরীক্ষামূলক `0.1.0` release, কিন্তু আপনি ফোন থেকেই `.pd` file তৈরি, run, check, format এবং lint করতে পারবেন।

## ১. একবার Padma install করুন

Termux খুলে শুধু এই command দুটি চালান:

```bash
curl -fsSL https://raw.githubusercontent.com/OfficialBiohub/padma-lang/main/install-termux.sh | bash
padma --version
```

দ্বিতীয় command-এ `Padma 0.1.0` দেখালে install হয়েছে। `pkg install padma -y` এখনো কাজ করবে না, কারণ Termux upstream personal GitHub project package করে না।

## ২. প্রথম file লিখুন ও চালান

নিচের command চালান:

```bash
nano hello.pd
```

তারপর এই code লিখুন:

```padma
ধরি নাম = "বাংলাদেশ"
দেখাও "হ্যালো, {নাম}!"
```

`Ctrl` ধরে `O`, তারপর `Enter` চাপলে file save হবে। `Ctrl` ধরে `X` চাপলে nano থেকে বের হবেন। তারপর run করুন:

```bash
padma hello.pd
```

English syntax চাইলে একই কাজ হবে:

```padma
let name = "Bangladesh"
print "Hello, {name}!"
```

## ৩. Padma interactive shell

File ছাড়া দ্রুত practice করতে লিখুন:

```bash
padma
```

তারপর prompt-এ লিখুন:

```text
padma> ১ + ১
2
padma> দেখাও ২ + ৩
5
padma> বের হও
```

Python-এর `>>> 1 + 1`-এর মতো Padma-তেও এক লাইনের bare expression লিখলে তার non-null result দেখা যাবে। `দেখাও`/`print` এখনও explicit output statement; আর `ধরি x = ...`-এর মতো declaration অথবা `null` কোনো output দেখায় না। Multi-line `যদি`, `যতক্ষণ`, বা `ফাংশন` লিখলে closing `}` পর্যন্ত `...` prompt দেখা যাবে।

## ৪. Run করার আগে ভুল ধরুন

Padma code execute না করেই error দেখাতে পারে:

```bash
padma check hello.pd
padma check --json hello.pd
```

প্রথম command মানুষ পড়ার মতো Bangla বা English error দেখায়। দ্বিতীয়টি editor বা CI-এর জন্য JSON দেয়। `padma fmt hello.pd` layout ঠিক করে; file না বদলে check করতে `padma fmt --check hello.pd` ব্যবহার করুন। Style warning দেখতে:

```bash
padma lint hello.pd
```

## ৫. নিরাপদ project তৈরি করুন

একটি বড় program-এর জন্য folder ব্যবহার করুন:

```bash
padma init আমার-প্রকল্প
cd আমার-প্রকল্প
padma capabilities .
padma .
```

`padma.toml` হলো project-এর নিয়মের file। Project mode-এ file লেখা, web request, process run, এবং media download **default-এ বন্ধ**। দরকার হলে manifest-এ সবচেয়ে ছোট permission দিন:

```toml
[capabilities]
filesystem = ["write"]
```

তারপর `padma capabilities .` চালিয়ে run করার আগে permission review করুন। `padma .` mode শুধু project folder-এর ভিতরে declared file access দিতে পারে; `..`, absolute path, এবং project-mode `@downloads` নিরাপত্তার জন্য rejected হয়।

## ৬. Android Downloads ও video download

Android Downloads folder access দেওয়ার আগে Termux-এর দৃশ্যমান consent চালান:

```bash
termux-setup-storage
```

এটি Android permission dialog দেখাবে; Padma এটি স্বয়ংক্রিয়ভাবে চালায় না। Direct single-file compatibility scripts existing safe `@downloads` alias ব্যবহার করতে পারে যদি Termux storage permission আগে থেকেই পায়। Project mode-এ shared-storage permission এখনও intentionally unavailable—প্রথমে project folder-এ output তৈরি করুন, পরে নিজে review করে copy করুন।

`media.download` শুধু আপনার নিজের অথবা download করার অনুমতি আছে এমন content-এর জন্য ব্যবহার করুন এবং service-এর terms মানুন।

## ৭. ফোনে editor tooling

Termux-এ `nano` সবচেয়ে সহজ এবং supported editor। Android-এর অন্য কোনো text editor দিয়ে `.pd` file লেখা যায়, কিন্তু Padma চালানো, error দেখা, format করা, এবং lint করার জন্য আবার Termux-এ ফিরে আসতে হবে। তাই phone-only workflow-তে editor-এর বদলে **Termux + nano + Padma CLI**-ই প্রধান পথ:

```bash
nano hello.pd
padma check hello.pd
padma fmt hello.pd
padma lint hello.pd
padma hello.pd
```

Desktop VS Code একটি optional দ্বিতীয় পথ; এটি ফোনে Padma চালানোর প্রয়োজনীয় শর্ত নয়। Computer বা remote desktop থাকলে repository-এর `tooling/vscode-padma` extension install করে `.pd` highlighting, explicit run/check/format/lint command, Problems panel diagnostics, এবং opt-in language server পাওয়া যায়। Language server কখনও file খোলার সময় নিজে চালু হয় না: Command Palette থেকে **Padma: Start Language Server** নির্বাচন করতে হয়।

VS Code যদি `padma` বা `padma-lsp` না পায়, Settings-এ `padma.command` ও `padma.languageServer.command`-এ ওই executable-এর পূর্ণ path দিন। Example workspace `tooling/vscode-padma/sample-workspace/` কোনো sensitive capability চায় না এবং Bangla shadowingসহ local rename পরীক্ষা করার জন্য ব্যবহার করা যায়। Termux-এ mobile-only হলে এই optional setup বাদ দিন; CLI commandগুলো একই Padma diagnostics দেয়।

## ৮. Optional Python বা JavaScript bridge

Padma-এর `bridge.call` দিয়ে existing Python বা Node.js library-এর ছোট, review করা helper ব্যবহার করা যায়। এটি shell নয়: Padma শুধু fixed `python` বা `node` executable এবং project-এর ভিতরের একটি `.py` বা `.js` file চালায়। Foreign script Android/Termux user-এর permission-এ চলে, তাই অচেনা script বা dependency চালাবেন না।

Python bridge চাইলে একবার install করুন:

```bash
pkg install python -y
```

তারপর project-এ grant দিন এবং bridge file বানান:

```toml
[capabilities]
process = ["python"]
```

```bash
mkdir -p bridges
nano bridges/average.py
```

`bridges/average.py`-এ লিখুন:

```python
import json
import sys

data = json.load(sys.stdin)
json.dump({"average": sum(data["scores"]) / len(data["scores"])}, sys.stdout)
```

তারপর `src/main.pd`-এ লিখুন:

```padma
ধরি ফল = bridge.call("python", "bridges/average.py", {"scores": [৪, ৫, ৫]})
দেখাও ফল["average"]
```

`padma capabilities .` দিয়ে `process:python` দেখুন, তারপর `padma .` চালান। JavaScript-এর জন্য `pkg install nodejs -y`, `process = ["node"]`, এবং `bridge.call("javascript", "bridges/helper.js", data)` ব্যবহার করুন। Bridge input ও output JSON data হতে হবে; 256 KiB-এর বেশি data বা 10 সেকেন্ডের বেশি সময় নিলে Padma error দেখাবে।

## ৯. HTTP API এবং AI service

HTTP JSON API ব্যবহার করতে project manifest-এ শুধু দরকারি permission দিন:

```toml
[capabilities]
network = ["http"]
```

```padma
ধরি উত্তর = http.json("https://api.example.com/tasks", {"name": "Padma"})
দেখাও উত্তর
```

AI provider-এর key কখনও `.pd` file, `padma.toml`, screenshot, বা GitHub commit-এ লিখবেন না। Termux shell-এ temporary environment variable দিন:

```bash
export PADMA_AI_KEY='your-provider-key'
```

তারপর manifest-এ `network = ["ai"]` দিন এবং explicit endpoint ব্যবহার করুন:

```padma
ধরি উত্তর = ai.request(
  "https://provider.example/v1/chat/completions",
  "PADMA_AI_KEY",
  {"model": "chosen-model", "messages": [{"role": "user", "content": "বাংলায় উত্তর দাও"}]}
)
দেখাও উত্তর
```

AI output data হিসেবে নিন; সেটি shell command, Padma code, বা Python code হিসেবে automatic চালাবেন না। `backend.response(status, headers, body)` দিয়ে validated response data বানানো এবং `automation.write_json("output/job.json", data)` দিয়ে project-এর ভেতরে JSON result file লেখা যায়। এই milestone-এ Padma নিজে public web server port খোলে না।

## সাহায্য দরকার হলে

এই command-গুলোর outputসহ issue report করুন:

```bash
padma --version
termux-info
padma check আপনার-file.pd
```

ব্যক্তিগত token, password, বা private URL issue report-এ দেবেন না। আরও বিস্তারিত নিয়মের জন্য [`PROJECTS.md`](PROJECTS.md), [`CAPABILITY-SECURITY.md`](CAPABILITY-SECURITY.md), এবং [`DIAGNOSTICS.md`](DIAGNOSTICS.md) পড়ুন।
