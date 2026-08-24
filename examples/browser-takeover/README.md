# Visible Browser Takeover Example

এই Termux-first example একটি reviewed documentation URL-এর জন্য `payment` label-সহ user-takeover checklist দেখায়। Labelটি কোনো payment চালায় না; এটি শুধু মনে করিয়ে দেয় যে sensitive step ব্যবহারকারী নিজে visible browser-এ করবেন।

```bash
padma browser plan examples/browser-takeover
padma browser takeover inspect examples/browser-takeover
padma browser takeover plan examples/browser-takeover
```

Expected output-এ `takeover.status: "user-takeover-required"`, `completion: "not-collected"`, এবং `payment: "disabled"` থাকবে। কোনো Android browser open হবে না, network/DNS হবে না, login/CAPTCHA/form/post/upload/download/account/purchase/payment action হবে না, এবং cookie/credential/profile/page data পড়া হবে না।

বাস্তবে URL খুলতে হলে আলাদা `browser:handoff`-enabled project ও `OPEN` foreground confirmation প্রয়োজন। Browser খোলার পরে action করার সিদ্ধান্ত, cancellation, এবং সব sensitive interaction আপনার নিজের হাতে থাকবে।
