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
padma> দেখাও ২ + ৩
5
padma> বের হও
```

বা `exit()` লিখুন। Multi-line `যদি`, `যতক্ষণ`, বা `ফাংশন` লিখলে closing `}` পর্যন্ত `...` prompt দেখা যাবে।

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

## সাহায্য দরকার হলে

এই command-গুলোর outputসহ issue report করুন:

```bash
padma --version
termux-info
padma check আপনার-file.pd
```

ব্যক্তিগত token, password, বা private URL issue report-এ দেবেন না। আরও বিস্তারিত নিয়মের জন্য [`PROJECTS.md`](PROJECTS.md), [`CAPABILITY-SECURITY.md`](CAPABILITY-SECURITY.md), এবং [`DIAGNOSTICS.md`](DIAGNOSTICS.md) পড়ুন।
