# Browser Draft Example

এই Termux-friendly example একটি reviewed documentation URL-এর জন্য ছোট একটি message draft দেখায়। এটি browser automation নয়।

```bash
padma browser plan examples/browser-draft
padma browser draft inspect examples/browser-draft
padma browser draft plan examples/browser-draft
```

`padma-browser-draft.toml`-এর digest `padma-browser.toml`-এর একমাত্র reviewed URL-এর সঙ্গে bound। `attachment_path` কেবল metadata: example-এ ওই file না থাকলেও command সফল হবে, কারণ Padma file খোঁজে, পড়ে, hash করে, বা upload করে না।

Output দেখে মানুষ চাইলে পরে আলাদা `browser:handoff`-enabled project থেকে visible Android browser খুলতে পারে এবং draft text নিজে copy/type করতে পারে। এই example login, CAPTCHA, form fill/submit, message post, upload/download, account change, purchase/payment, cookie/profile/credential access, JavaScript injection, বা generated text execution করে না।
